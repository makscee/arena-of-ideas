use std::collections::VecDeque;

use super::*;

#[reducer]
fn content_publish_node(ctx: &ReducerContext, pack: String) -> Result<(), String> {
    let _ = ctx.player()?;
    let mut pack = ron::from_str::<PackedNodes>(&pack).map_err(|e| e.to_string())?;
    let mut next_id = ctx.next_id();
    pack.reassign_ids(&mut next_id);
    GlobalData::set_next_id(ctx, next_id);
    let mut remap: HashMap<u64, u64> = default();
    for (id, NodeData { kind, data }) in &pack.nodes {
        if let Some(n) = ctx.db.nodes_world().kind_data().filter((kind, data)).next() {
            remap.insert(*id, n.id);
            continue;
        }
        let tnode = TNode::new(*id, 0, kind.to_kind(), data.clone());
        tnode.insert(ctx);
    }
    for NodeLink {
        mut parent,
        mut child,
        parent_kind,
        child_kind,
    } in pack.links
    {
        if let Some(id) = remap.get(&parent) {
            parent = *id;
        }
        if let Some(id) = remap.get(&child) {
            child = *id;
        }
        let _ = TNodeLink::add_by_id(ctx, parent, child, parent_kind, child_kind, false);
    }
    Ok(())
}

#[reducer]
fn content_rotation(ctx: &ReducerContext) -> Result<(), String> {
    ctx.is_admin()?;
    info!("content rotation start");
    for mut n in ctx.db.nodes_world().owner().filter(ID_CORE) {
        n.owner = 0;
        for mut l in ctx
            .db
            .node_links()
            .parent()
            .filter(n.id)
            .chain(ctx.db.node_links().child().filter(n.id))
        {
            if !l.solid {
                continue;
            }
            l.solid = false;
            ctx.db.node_links().id().update(l);
        }
        n.update(ctx);
    }

    let mut units = NUnit::collect_owner(ctx, 0);
    info!("initial units {}", units.len());
    units.retain_mut(|unit| {
        let Some(mut description) = unit.top_parent::<NUnitDescription>(ctx) else {
            return false;
        };
        if let Some(behavior) = description.mutual_top_parent::<NBehavior>(ctx) {
            description.behavior = Some(behavior);
        } else {
            return false;
        }
        if let Some(representation) = description.mutual_top_parent::<NRepresentation>(ctx) {
            description.representation = Some(representation);
        } else {
            return false;
        }
        unit.description = Some(description);
        if let Some(stats) = unit.top_parent::<NUnitStats>(ctx) {
            unit.stats = Some(stats);
        } else {
            return false;
        }
        true
    });
    info!("retained units {}", units.len());
    let mut units: HashMap<u64, Vec<NUnit>> = units
        .into_iter()
        .filter_map(|n| {
            if let Some(house) = n.top_parent::<NHouse>(ctx) {
                Some((house.id, n))
            } else {
                None
            }
        })
        .into_group_map();
    info!("units with house {}", units.len());

    let mut houses: VecDeque<NHouse> = VecDeque::from_iter(NHouse::collect_owner(ctx, 0));
    while let Some(mut house) = houses.pop_front() {
        info!("start house {}", house.house_name);
        if let Some(color) = house.mutual_top_parent::<NHouseColor>(ctx) {
            house.color = Some(color);
        } else {
            error!("color failed");
            continue;
        }
        if let Some(mut ability_magic) = house.mutual_top_parent::<NAbilityMagic>(ctx) {
            if let Some(mut description) =
                ability_magic.mutual_top_parent::<NAbilityDescription>(ctx)
            {
                if let Some(effect) = description.mutual_top_parent::<NAbilityEffect>(ctx) {
                    description.effect = Some(effect);
                } else {
                    error!("ability effect failed");
                    continue;
                }
                ability_magic.description = Some(description);
            } else {
                error!("ability description failed");
                continue;
            }
            house.ability_magic = Some(ability_magic);
        } else {
            error!("ability magic failed");
            continue;
        }
        if let Some(mut status_magic) = house.mutual_top_parent::<NStatusMagic>(ctx) {
            if let Some(mut description) = status_magic.mutual_top_parent::<NStatusDescription>(ctx)
            {
                if let Some(behavior) = description.mutual_top_parent::<NBehavior>(ctx) {
                    description.behavior = Some(behavior);
                } else {
                    error!("status behavior failed");
                    continue;
                }
                status_magic.description = Some(description);
            } else {
                error!("status description failed");
                continue;
            }
            house.status_magic = Some(status_magic);
        } else {
            error!("status magic failed");
            continue;
        }
        if let Some(units) = units.remove(&house.id) {
            house.units = units;
        } else {
            error!("units failed");
            continue;
        }

        info!("solidifying house {}", house.house_name);
        for id in house.collect_ids() {
            let mut node = id.find(ctx).unwrap();
            node.owner = ID_CORE;
            node.update(ctx);
        }
        house.solidify_links(ctx)?;
    }

    Ok(())
}

#[reducer]
fn content_delete_node(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    ctx.is_admin()?;
    let kind = id.kind(ctx).to_custom_e_s("Failed to get kind")?;
    kind.delete_with_components(ctx, id)
}

#[reducer]
fn content_vote_node(ctx: &ReducerContext, id: u64, vote: bool) -> Result<(), String> {
    let player = ctx.player()?;
    let mut node = id.find_err(ctx)?;
    let vote = if vote { 1 } else { -1 };
    node.rating += vote;
    node.update(ctx);
    Ok(())
}
