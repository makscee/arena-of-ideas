use super::*;

use log::{error, info};
use spacetimedb::{Identity, ReducerContext, reducer};
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;

const ADMIN_IDENTITY_HEX: &str = "c200ef827a902b03c6c49d77435af70cad0003fc322d23cf338142fa883b2dcf";

pub fn is_admin(identity: &Identity) -> Result<bool, String> {
    Ok(Identity::from_str(ADMIN_IDENTITY_HEX)
        .map_err(|e| e.to_string())?
        .eq(identity))
}

pub trait AdminCheck {
    fn is_admin(self) -> Result<(), String>;
}

impl AdminCheck for &ReducerContext {
    fn is_admin(self) -> Result<(), String> {
        if is_admin(&self.sender)? {
            Ok(())
        } else {
            Err("Need admin access".to_owned())
        }
    }
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
    let ctx = &ctx.as_context();
    let mut units = NUnit::collect_owner(ctx, 0);
    info!("initial units {}", units.len());
    units.retain_mut(|unit| {
        let Some(mut description) = unit
            .id
            .top_parent(ctx.rctx(), NodeKind::NUnitDescription)
            .and_then(|id| ctx.load::<NUnitDescription>(id).ok())
        else {
            return false;
        };
        if let Some(behavior) = description
            .id
            .mutual_top_parent(ctx.rctx(), NodeKind::NUnitBehavior)
            .and_then(|id| ctx.load::<NUnitBehavior>(id).ok())
        {
            description.behavior.state_mut().set(behavior);
        } else {
            return false;
        }
        if let Some(representation) = description
            .id
            .mutual_top_parent(ctx.rctx(), NodeKind::NUnitRepresentation)
            .and_then(|id| ctx.load::<NUnitRepresentation>(id).ok())
        {
            description.representation.state_mut().set(representation);
        } else {
            return false;
        }
        unit.description.state_mut().set(description);
        if let Some(stats) = unit
            .id
            .top_parent(ctx.rctx(), NodeKind::NUnitStats)
            .and_then(|id| ctx.load::<NUnitStats>(id).ok())
        {
            unit.stats.state_mut().set(stats);
        } else {
            return false;
        }
        true
    });
    info!("retained units {}", units.len());
    // Group units by their magic type (ability or status)
    let mut ability_units: HashMap<u64, Vec<NUnit>> = HashMap::new();
    let mut status_units: HashMap<u64, Vec<NUnit>> = HashMap::new();

    for unit in units {
        if let Some(description) = unit.description.get() {
            match description.magic_type {
                MagicType::Ability => {
                    // Find the ability magic this unit should belong to
                    if let Some(ability_id) =
                        unit.id.top_parent(ctx.rctx(), NodeKind::NAbilityMagic)
                    {
                        ability_units.entry(ability_id).or_default().push(unit);
                    }
                }
                MagicType::Status => {
                    // Find the status magic this unit should belong to
                    if let Some(status_id) = unit.id.top_parent(ctx.rctx(), NodeKind::NStatusMagic)
                    {
                        status_units.entry(status_id).or_default().push(unit);
                    }
                }
            }
        }
    }
    info!(
        "ability units: {}, status units: {}",
        ability_units.len(),
        status_units.len()
    );

    let mut houses: VecDeque<NHouse> = VecDeque::from_iter(NHouse::collect_owner(ctx, 0));
    while let Some(mut house) = houses.pop_front() {
        info!("start house {}", house.house_name);
        if let Some(color) = house
            .id
            .mutual_top_parent(ctx.rctx(), NodeKind::NHouseColor)
            .and_then(|id| ctx.load::<NHouseColor>(id).ok())
        {
            info!("color: {}", color.color.0);
            house.color.state_mut().set(color);
        } else {
            error!("color failed");
            continue;
        }
        if let Some(mut ability) = house
            .id
            .mutual_top_child(ctx.rctx(), NodeKind::NAbilityMagic)
            .and_then(|id| ctx.load::<NAbilityMagic>(id).ok())
        {
            if let Some(mut description) = ability
                .id
                .mutual_top_parent(ctx.rctx(), NodeKind::NAbilityDescription)
                .and_then(|id| ctx.load::<NAbilityDescription>(id).ok())
            {
                if let Some(effect) = description
                    .id
                    .mutual_top_parent(ctx.rctx(), NodeKind::NAbilityEffect)
                    .and_then(|id| ctx.load::<NAbilityEffect>(id).ok())
                {
                    description.effect.state_mut().set(effect);
                    ability.description.state_mut().set(description);
                    let ability_id = ability.id;
                    house.ability.state_mut().set(ability);
                    if let Some(units) = ability_units.remove(&ability_id) {
                        // TODO: Handle units assignment
                    }
                } else {
                    error!("ability effect failed");
                    continue;
                }
            } else {
                error!("ability description failed");
                continue;
            }
        } else {
            error!("ability child failed");
            continue;
        }
        if let Some(mut status) = house
            .id
            .mutual_top_child(ctx.rctx(), NodeKind::NStatusMagic)
            .and_then(|id| ctx.load::<NStatusMagic>(id).ok())
        {
            if let Some(mut description) = status
                .id
                .mutual_top_parent(ctx.rctx(), NodeKind::NStatusDescription)
                .and_then(|id| ctx.load::<NStatusDescription>(id).ok())
            {
                if let Some(behavior) = description
                    .id
                    .mutual_top_parent(ctx.rctx(), NodeKind::NStatusBehavior)
                    .and_then(|id| ctx.load::<NStatusBehavior>(id).ok())
                {
                    description.behavior.state_mut().set(behavior);
                    status.description.state_mut().set(description);
                    let status_id = status.id;
                    house.status.state_mut().set(status);
                    if let Some(units) = status_units.remove(&status_id) {
                        // TODO: Handle units assignment
                    }
                } else {
                    error!("status behavior failed");
                    continue;
                }
            } else {
                error!("status description failed");
                continue;
            }
        } else {
            error!("status magic failed");
            continue;
        }
        if house.status.is_none() {
            error!("failed to get status for house");
            continue;
        }
        if house.ability.is_none() {
            error!("failed to get ability for house");
            continue;
        }

        info!("solidifying house {}", house.house_name);
        for id in house.collect_owned_ids() {
            let mut node = id.load_tnode(ctx.rctx()).unwrap();
            node.owner = ID_CORE;
            node.update(ctx.rctx());
        }
        for (parent_id, child_id) in house.collect_owned_links() {
            TNodeLink::solidify(ctx.rctx(), parent_id, child_id)?;
        }
    }

    Ok(())
}

#[reducer]
fn admin_daily_update(ctx: &ReducerContext) -> Result<(), String> {
    ctx.is_admin()?;
    crate::daily_updater::daily_update(ctx)
}

#[reducer]
fn admin_delete_node_recursive(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    ctx.is_admin()?;
    TNode::delete_by_id_recursive(ctx, id);
    Ok(())
}

#[reducer]
fn admin_upload_world(
    ctx: &ReducerContext,
    global_settings: GlobalSettings,
    nodes: Vec<String>,
    links: Vec<String>,
) -> Result<(), String> {
    GlobalData::init(ctx);
    ctx.is_admin()?;
    global_settings.replace(ctx);
    for node in ctx.db.nodes_world().iter() {
        ctx.db.nodes_world().id().delete(node.id);
    }
    for link in ctx.db.node_links().iter() {
        ctx.db.node_links().id().delete(link.id);
    }
    for node in nodes {
        let (id, kind, (data, owner, rating)) =
            ron::from_str::<(u64, String, NodeAsset)>(&node).map_err(|e| e.to_string())?;
        TNode {
            id,
            owner,
            kind,
            data,
            rating,
        }
        .insert(ctx);
    }
    for link in links {
        let (parent, child, parent_kind, child_kind, rating, solid) =
            ron::from_str::<LinkAsset>(&link).map_err(|e| e.to_string())?;
        let solid = solid != 0;
        TNodeLink {
            id: ctx.as_context().next_id(),
            parent,
            child,
            parent_kind,
            child_kind,
            rating,
            solid,
        }
        .insert(ctx);
    }
    Ok(())
}

#[reducer]
fn admin_add_gold(ctx: &ReducerContext) -> Result<(), String> {
    ctx.is_admin()?;
    let ctx = &ctx.as_context();
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    m.g += 10;
    player.save(ctx.source());
    Ok(())
}

#[reducer]
fn admin_sync_link_ratings(ctx: &ReducerContext) -> Result<(), String> {
    ctx.is_admin()?;
    info!("syncing link ratings with selection counts");

    for link in ctx.db.node_links().iter() {
        link.sync_rating_with_selections(ctx);
    }

    info!("link ratings sync complete");
    Ok(())
}
