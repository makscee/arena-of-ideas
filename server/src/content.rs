use std::collections::VecDeque;

use super::*;

#[reducer]
fn content_publish_node(ctx: &ReducerContext, pack: String) -> Result<(), String> {
    let _ = ctx.player()?;
    let mut pack = ron::from_str::<PackedNodes>(&pack).map_err(|e| e.to_string())?;
    let mut next_id = ctx.next_id();
    pack.reassign_ids(&mut next_id);
    GlobalData::set_next_id(ctx, next_id);
    for (id, NodeData { kind, data }) in &pack.nodes {
        let tnode = TNode::new(*id, 0, kind.to_kind(), data.clone());
        tnode.insert(ctx);
    }
    for NodeLink {
        parent,
        child,
        parent_kind,
        child_kind,
    } in pack.links
    {
        TNodeLink::add_by_id(ctx, parent, child, parent_kind, child_kind)?;
    }
    Ok(())
}

#[reducer]
fn content_rotation(ctx: &ReducerContext) -> Result<(), String> {
    ctx.is_admin()?;
    for mut n in ctx.db.nodes_world().owner().filter(ID_CORE) {
        n.owner = 0;
        n.update(ctx);
    }
    ctx.db.node_links().parent().delete(ID_CORE);

    let mut units = NUnit::collect_kind_by_owner(ctx, 0);
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

    let mut houses = VecDeque::from_iter(NHouse::collect_kind_by_owner(ctx, 0).into_iter());
    while let Some(mut house) = houses.pop_front() {
        if let Some(color) = house.mutual_top_parent::<NHouseColor>(ctx) {
            house.color = Some(color);
        } else {
            continue;
        }
        if let Some(mut ability_magic) = house.mutual_top_parent::<NAbilityMagic>(ctx) {
            if let Some(mut description) =
                ability_magic.mutual_top_parent::<NAbilityDescription>(ctx)
            {
                if let Some(effect) = description.mutual_top_parent::<NAbilityEffect>(ctx) {
                    description.effect = Some(effect);
                } else {
                    continue;
                }
                ability_magic.description = Some(description);
            } else {
                continue;
            }
            house.ability_magic = Some(ability_magic);
        } else {
            continue;
        }
        if let Some(mut status_magic) = house.mutual_top_parent::<NStatusMagic>(ctx) {
            if let Some(mut description) = status_magic.mutual_top_parent::<NStatusDescription>(ctx)
            {
                if let Some(behavior) = description.mutual_top_parent::<NBehavior>(ctx) {
                    description.behavior = Some(behavior);
                } else {
                    continue;
                }
                status_magic.description = Some(description);
            } else {
                continue;
            }
            house.status_magic = Some(status_magic);
        } else {
            continue;
        }
        if let Some(units) = units.remove(&house.id) {
            house.units = units;
        } else {
            continue;
        }

        for id in house.collect_ids() {
            let mut node = id.get(ctx).unwrap();
            node.owner = ID_CORE;
            node.update(ctx);
        }
    }

    Ok(())
}
