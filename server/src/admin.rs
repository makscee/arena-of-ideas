use super::*;

use log::{error, info};
use spacetimedb::{Identity, ReducerContext, reducer};
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;

const ADMIN_IDENTITY_HEX: &str = "c200fd59bb7bf2e9069a5db968bea2659d6826ab39f4e07fa96b0756fae2b16a";

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
    info!("Content rotation start");
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
            .top_child(ctx.rctx(), NodeKind::NUnitDescription)
            .and_then(|id| ctx.load::<NUnitDescription>(id).ok())
        else {
            info!("Skip unit {}: no description", unit.name());
            return false;
        };
        if let Some(behavior) = description
            .id
            .mutual_top_child(ctx.rctx(), NodeKind::NUnitBehavior)
            .and_then(|id| ctx.load::<NUnitBehavior>(id).ok())
        {
            description.behavior.set_loaded(behavior).ok();
        } else {
            info!("Skip unit {}: no behavior", unit.name());
            return false;
        }
        if let Some(representation) = description
            .id
            .mutual_top_child(ctx.rctx(), NodeKind::NUnitRepresentation)
            .and_then(|id| ctx.load::<NUnitRepresentation>(id).ok())
        {
            description.representation.set_loaded(representation).ok();
        } else {
            info!("Skip unit {}: no representation", unit.name());
            return false;
        }
        unit.description.set_loaded(description).ok();
        if let Some(stats) = unit
            .id
            .top_child(ctx.rctx(), NodeKind::NUnitStats)
            .and_then(|id| ctx.load::<NUnitStats>(id).ok())
        {
            unit.stats.set_loaded(stats).ok();
        } else {
            info!("Skip unit {}: no stats", unit.name());
            return false;
        }
        true
    });
    info!("retained units {}", units.len());
    let mut house_units: HashMap<u64, Vec<NUnit>> = HashMap::new();
    for unit in units {
        if let Some(house) = unit.id.top_parent(ctx.rctx(), NodeKind::NHouse) {
            house_units.entry(house).or_default().push(unit);
        }
    }
    info!("house units: {}", house_units.len());
    let mut abilities: Vec<NAbilityMagic> = NAbilityMagic::collect_owner(ctx, 0);
    abilities.retain_mut(|ability| {
        let Some(mut description) = ability
            .id
            .top_child(ctx.rctx(), NodeKind::NAbilityDescription)
            .and_then(|id| ctx.load::<NAbilityDescription>(id).ok())
        else {
            info!("Skip ability {}: no description", ability.name());
            return false;
        };
        if let Some(effect) = description
            .id
            .mutual_top_child(ctx.rctx(), NodeKind::NAbilityEffect)
            .and_then(|id| ctx.load::<NAbilityEffect>(id).ok())
        {
            description.effect.set_loaded(effect).unwrap();
        } else {
            info!("Skip ability {}: no effect", ability.name());
            return false;
        }
        ability.description_set(description).unwrap();
        info!("Keep ability {}", ability.name());
        true
    });
    info!("abilities: {}", abilities.len());
    let mut abilities: HashMap<u64, NAbilityMagic> =
        HashMap::from_iter(abilities.into_iter().map(|s| (s.id, s)));

    let mut statuses: Vec<NStatusMagic> = NStatusMagic::collect_owner(ctx, 0);
    statuses.retain_mut(|status| {
        let Some(mut description) = status
            .id
            .top_child(ctx.rctx(), NodeKind::NStatusDescription)
            .and_then(|id| ctx.load::<NStatusDescription>(id).ok())
        else {
            info!("Skip status {}: no description", status.name());
            return false;
        };
        if let Some(behavior) = description
            .id
            .mutual_top_child(ctx.rctx(), NodeKind::NStatusBehavior)
            .and_then(|id| ctx.load::<NStatusBehavior>(id).ok())
        {
            description.behavior.set_loaded(behavior).unwrap();
        } else {
            info!("Skip status {}: no behavior", status.name());
            return false;
        }
        if let Some(rep) = status
            .id
            .mutual_top_child(ctx.rctx(), NodeKind::NStatusRepresentation)
            .and_then(|id| ctx.load::<NStatusRepresentation>(id).ok())
        {
            status.representation.set_loaded(rep).unwrap();
        }
        status.description_set(description).unwrap();
        info!("Keep status {}", status.name());
        true
    });
    info!("statuses: {}", statuses.len());
    let mut statuses: HashMap<u64, NStatusMagic> =
        HashMap::from_iter(statuses.into_iter().map(|s| (s.id, s)));

    let mut houses: VecDeque<NHouse> = VecDeque::from_iter(NHouse::collect_owner(ctx, 0));
    while let Some(mut house) = houses.pop_front() {
        info!("\n===\nstart house {}", house.house_name);
        if let Some(color) = house
            .id
            .mutual_top_child(ctx.rctx(), NodeKind::NHouseColor)
            .and_then(|id| ctx.load::<NHouseColor>(id).ok())
        {
            info!("color: {}", color.color.0);
            house.color.set_loaded(color).ok();
        } else {
            info!("Skip house {}: no color", house.house_name);
            continue;
        }
        if let Some(ability) = house
            .id
            .mutual_top_child(ctx.rctx(), NodeKind::NAbilityMagic)
            .and_then(|id| abilities.remove(&id))
        {
            info!("ability: {}", ability.name());
            house.ability_set(ability).unwrap();
        }
        if let Some(status) = house
            .id
            .mutual_top_child(ctx.rctx(), NodeKind::NStatusMagic)
            .and_then(|id| statuses.remove(&id))
        {
            info!("status: {}", status.name());
            house.status_set(status).unwrap();
        }
        if house.status.is_none() && house.ability.is_none() {
            info!("Skip house {}: no ability or status", house.name());
            continue;
        }
        for unit in house_units.remove(&house.id).unwrap_or_default() {
            if !match unit.description().unwrap().magic_type {
                MagicType::Ability => house.ability.is_loaded(),
                MagicType::Status => house.status.is_loaded(),
            } {
                error!(
                    "Skip unit {} house {}: required {}",
                    unit.name(),
                    house.name(),
                    unit.description().unwrap().magic_type
                );
                continue;
            }
            info!("unit: {}", unit.name());
            house.units_push(unit).unwrap();
        }

        info!("Solidifying house {}\n===\n", house.house_name);
        for id in house.collect_owned_ids() {
            let mut node = id.load_tnode(ctx.rctx()).unwrap();
            node.owner = ID_CORE;
            node.update(ctx.rctx());
        }
        for (parent_id, child_id) in house.collect_owned_links() {
            let _ = TNodeLink::solidify(ctx.rctx(), parent_id, child_id);
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
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    m.g += 10;
    player.save(ctx)?;
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
