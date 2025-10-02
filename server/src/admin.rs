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

    let mut units = NUnit::collect_owner(ctx, 0);
    info!("initial units {}", units.len());
    units.retain_mut(|unit| {
        let Some(mut description) = unit.top_parent::<NUnitDescription>(ctx) else {
            return false;
        };
        if let Some(behavior) = description.mutual_top_parent::<NUnitBehavior>(ctx) {
            description.behavior.state_mut().set(behavior);
        } else {
            return false;
        }
        if let Some(representation) = description.mutual_top_parent::<NUnitRepresentation>(ctx) {
            description.representation.state_mut().set(representation);
        } else {
            return false;
        }
        unit.description.state_mut().set(description);
        if let Some(stats) = unit.top_parent::<NUnitStats>(ctx) {
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
                    if let Some(ability) = unit.top_parent::<NAbilityMagic>(ctx) {
                        ability_units.entry(ability.id).or_default().push(unit);
                    }
                }
                MagicType::Status => {
                    // Find the status magic this unit should belong to
                    if let Some(status) = unit.top_parent::<NStatusMagic>(ctx) {
                        status_units.entry(status.id).or_default().push(unit);
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
        if let Some(color) = house.mutual_top_parent::<NHouseColor>(ctx) {
            info!("color: {}", color.color.0);
            house.color.state_mut().set(color);
        } else {
            error!("color failed");
            continue;
        }
        if let Some(mut ability) = house.mutual_top_child::<NAbilityMagic>(ctx) {
            if let Some(mut description) = ability.mutual_top_parent::<NAbilityDescription>(ctx) {
                if let Some(effect) = description.mutual_top_parent::<NAbilityEffect>(ctx) {
                    description.effect.state_mut().set(effect);
                    ability.description.state_mut().set(description);
                    house.ability.state_mut().set(ability);
                    todo!();
                    // if let Some(units) = ability_units.remove(&ability.id) {
                    //     ability.units.state_mut().set(units);
                    // }
                } else {
                    error!("ability effect failed");
                }
            } else {
                error!("ability description failed");
            }
        };
        if let Some(mut status) = house.mutual_top_child::<NStatusMagic>(ctx) {
            if let Some(mut description) = status.mutual_top_parent::<NStatusDescription>(ctx) {
                if let Some(behavior) = description.mutual_top_parent::<NStatusBehavior>(ctx) {
                    description.behavior.state_mut().set(behavior);
                    status.description.state_mut().set(description);
                    house.status.state_mut().set(status);
                    todo!();
                    // if let Some(units) = status_units.remove(&status.id) {
                    //     status.units.set_data(units);
                    // }
                } else {
                    error!("status behavior failed");
                }
            } else {
                error!("status description failed");
            }
        } else {
            error!("status magic failed");
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
        for id in house.collect_ids() {
            let mut node = id.load_tnode(ctx).unwrap();
            node.owner = ID_CORE;
            node.update(ctx);
        }
        house.solidify_links(ctx)?;
    }

    Ok(())
}

#[reducer]
fn admin_delete_node(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    ctx.is_admin()?;
    let kind = id.kind(ctx).to_custom_e_s("Failed to get kind")?;
    kind.delete_with_parts(ctx, id)
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
            id: ctx.next_id(),
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
    let ctx = ctx.as_context();
    let mut player = ctx.player()?;

    let m = player.active_match_load(ctx)?;
    m.g += 10;
    player.save(ctx.source().reducer_context());

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
