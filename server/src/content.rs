use super::*;

#[reducer]
fn content_publish_node(ctx: &ReducerContext, pack: String) -> Result<(), String> {
    let ctx = ctx.as_context();
    let player = ctx.player()?;
    let mut pack = ron::from_str::<PackedNodes>(&pack).map_err(|e| e.to_string())?;
    let mut next_id = ctx.next_id();
    pack.reassign_ids(&mut next_id);
    GlobalData::set_next_id(ctx.rctx(), next_id);
    let mut remap: HashMap<u64, u64> = default();
    for (id, NodeData { kind, data }) in &pack.nodes {
        let filter = ctx.rctx().db.nodes_world().kind_data_owner();
        if let Some(n) = filter
            .filter((kind, data, ID_CORE))
            .next()
            .or_else(|| filter.filter((kind, data, 0u64)).next())
        {
            remap.insert(*id, n.id);
            continue;
        }
        let kind = kind.to_kind();
        if !kind.is_content() {
            continue;
        }
        let tnode = TNode::new(*id, ID_INCUBATOR, kind, data.clone());
        tnode.insert(ctx.rctx());
        // Record who created this node
        TCreators::record_creation(ctx.rctx(), player.id, *id);
    }
    for NodeLink {
        mut parent,
        mut child,
        parent_kind,
        child_kind,
    } in pack.links
    {
        if !parent_kind.to_kind().is_content() || !child_kind.to_kind().is_content() {
            continue;
        }
        if let Some(id) = remap.get(&parent) {
            parent = *id;
        }
        if let Some(id) = remap.get(&child) {
            child = *id;
        }
        TNodeLink::add_by_id(ctx.rctx(), parent, child, parent_kind, child_kind)?;
    }
    Ok(())
}

#[reducer]
fn content_upvote_node(ctx: &ReducerContext, node_id: u64) -> Result<(), String> {
    let player = ctx.as_context().player()?;
    TVotes::upvote_node(ctx, player.id, node_id).map_err(|e| e.to_string())
}

#[reducer]
fn content_downvote_node(ctx: &ReducerContext, node_id: u64) -> Result<(), String> {
    let player = ctx.as_context().player()?;
    TVotes::downvote_node(ctx, player.id, node_id).map_err(|e| e.to_string())
}

#[reducer]
fn content_suggest_node(ctx: &ReducerContext, kind: String, name: String) -> Result<(), String> {
    let ctx = ctx.as_context();
    let player = ctx.player()?;
    let named_kind = kind
        .parse::<NamedNodeKind>()
        .map_err(|_| format!("Invalid node kind: {}", kind))?;

    let node_id = ctx.next_id();

    let tnode = match named_kind {
        NamedNodeKind::NUnit => {
            let unit = NUnit {
                id: node_id,
                owner: ID_INCUBATOR,
                unit_name: name,
                description: Component::none(),
                stats: Component::none(),
                state: Component::none(),
                is_dirty: false,
            };
            unit.to_tnode()
        }
        NamedNodeKind::NHouse => {
            let house = NHouse {
                id: node_id,
                owner: ID_INCUBATOR,
                house_name: name,
                color: Component::none(),
                ability: Component::none(),
                status: Component::none(),
                state: Component::none(),
                units: OwnedMultiple::none(),
                is_dirty: false,
            };
            house.to_tnode()
        }
        NamedNodeKind::NAbilityMagic => {
            let ability = NAbilityMagic {
                id: node_id,
                owner: ID_INCUBATOR,
                ability_name: name,
                description: Component::none(),
                is_dirty: false,
            };
            ability.to_tnode()
        }
        NamedNodeKind::NStatusMagic => {
            let status = NStatusMagic {
                id: node_id,
                owner: ID_INCUBATOR,
                status_name: name,
                description: Component::none(),
                representation: Component::none(),
                state: Component::none(),
                is_dirty: false,
            };
            status.to_tnode()
        }
    };

    tnode.insert(ctx.rctx());
    TCreators::record_creation(ctx.rctx(), player.id, node_id);
    GlobalData::set_next_id(ctx.rctx(), node_id + 1);

    Ok(())
}

#[reducer]
fn content_check_phase_completion(ctx: &ReducerContext) -> Result<(), String> {
    let threshold = 5i32;

    // Check all incubator nodes to see if any have reached the threshold
    let incubator_nodes: Vec<_> = ctx
        .db
        .nodes_world()
        .owner()
        .filter(ID_INCUBATOR)
        .filter(|n| n.rating >= threshold)
        .collect();

    for node in incubator_nodes {
        let kind = node.kind();

        // Check if this is a component that has reached threshold
        match kind {
            NodeKind::NUnitDescription
            | NodeKind::NUnitRepresentation
            | NodeKind::NUnitBehavior
            | NodeKind::NUnitStats
            | NodeKind::NHouseColor
            | NodeKind::NAbilityMagic
            | NodeKind::NStatusMagic => {
                // Fix this component (keep it, delete alternatives)
                if let Some(parent_link) = ctx.db.node_links().child().filter(node.id).next() {
                    // Delete all other suggestions for the same parent and component type
                    let other_links: Vec<_> = ctx
                        .db
                        .node_links()
                        .parent_child_kind()
                        .filter((&parent_link.parent, &parent_link.child_kind))
                        .filter(|l| l.child != node.id)
                        .collect();

                    for link in other_links {
                        // Delete the alternative node
                        if let Some(alt_node) = ctx.db.nodes_world().id().find(link.child) {
                            if alt_node.owner == ID_INCUBATOR {
                                TNode::delete_by_id_recursive(ctx, link.child);
                            }
                        }
                    }
                }
            }
            NodeKind::NUnit | NodeKind::NHouse => {
                // Check if unit/house is complete and can enter the game
                let is_complete = match kind {
                    NodeKind::NUnit => {
                        let has_description = node
                            .id
                            .get_kind_child(ctx, NodeKind::NUnitDescription)
                            .and_then(|id| ctx.db.nodes_world().id().find(id))
                            .map(|n| n.rating >= threshold)
                            .unwrap_or(false);
                        let has_representation = node
                            .id
                            .get_kind_child(ctx, NodeKind::NUnitRepresentation)
                            .and_then(|id| ctx.db.nodes_world().id().find(id))
                            .map(|n| n.rating >= threshold)
                            .unwrap_or(false);
                        let has_behavior = node
                            .id
                            .get_kind_child(ctx, NodeKind::NUnitBehavior)
                            .and_then(|id| ctx.db.nodes_world().id().find(id))
                            .map(|n| n.rating >= threshold)
                            .unwrap_or(false);
                        let has_stats = node
                            .id
                            .get_kind_child(ctx, NodeKind::NUnitStats)
                            .and_then(|id| ctx.db.nodes_world().id().find(id))
                            .map(|n| n.rating >= threshold)
                            .unwrap_or(false);
                        has_description && has_representation && has_behavior && has_stats
                    }
                    NodeKind::NHouse => {
                        let has_color = node
                            .id
                            .get_kind_child(ctx, NodeKind::NHouseColor)
                            .and_then(|id| ctx.db.nodes_world().id().find(id))
                            .map(|n| n.rating >= threshold)
                            .unwrap_or(false);
                        let has_ability = node
                            .id
                            .get_kind_child(ctx, NodeKind::NAbilityMagic)
                            .and_then(|id| ctx.db.nodes_world().id().find(id))
                            .map(|n| n.rating >= threshold)
                            .unwrap_or(false);
                        let has_status = node
                            .id
                            .get_kind_child(ctx, NodeKind::NStatusMagic)
                            .and_then(|id| ctx.db.nodes_world().id().find(id))
                            .map(|n| n.rating >= threshold)
                            .unwrap_or(false);
                        has_color && (has_ability || has_status)
                    }
                    _ => false,
                };

                if is_complete {
                    // Move to ID_CORE
                    let mut node_mut = node.clone();
                    node_mut.owner = ID_CORE;
                    node_mut.update(ctx);

                    // Also update all child components
                    for child_id in node.id.collect_children(ctx) {
                        if let Some(mut child_node) = ctx.db.nodes_world().id().find(child_id) {
                            if child_node.owner == ID_INCUBATOR {
                                child_node.owner = ID_CORE;
                                child_node.update(ctx);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Clean up nodes with very negative ratings
    let delete_threshold = -5i32;
    let nodes_to_delete: Vec<_> = ctx
        .db
        .nodes_world()
        .owner()
        .filter(ID_INCUBATOR)
        .filter(|n| n.rating <= delete_threshold)
        .map(|n| n.id)
        .collect();

    for id in nodes_to_delete {
        TNode::delete_by_id_recursive(ctx, id);
    }

    Ok(())
}
