use super::*;
use itertools::Itertools;
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

// Content admin reducers
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
            description.behavior = Some(behavior);
        } else {
            return false;
        }
        if let Some(representation) = description.mutual_top_parent::<NUnitRepresentation>(ctx) {
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
            info!("color: {}", color.color.0);
            house.color = Some(color);
        } else {
            error!("color failed");
            continue;
        }
        if let Some(mut action) = house.mutual_top_parent::<NActionAbility>(ctx) {
            if let Some(mut description) = action.mutual_top_parent::<NActionDescription>(ctx) {
                if let Some(effect) = description.mutual_top_parent::<NActionEffect>(ctx) {
                    description.effect = Some(effect);
                    action.description = Some(description);
                    house.action = Some(action);
                } else {
                    error!("ability effect failed");
                }
            } else {
                error!("ability description failed");
            }
        };
        if house.action.is_none() {
            if let Some(mut status) = house.mutual_top_parent::<NStatusAbility>(ctx) {
                if let Some(mut description) = status.mutual_top_parent::<NStatusDescription>(ctx) {
                    if let Some(behavior) = description.mutual_top_parent::<NStatusBehavior>(ctx) {
                        description.behavior = Some(behavior);
                        status.description = Some(description);
                        house.status = Some(status);
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
                error!("failed to get ability for house");
                continue;
            }
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
fn admin_delete_node(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    ctx.is_admin()?;
    let kind = id.kind(ctx).to_custom_e_s("Failed to get kind")?;
    kind.delete_with_components(ctx, id)
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
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    m.g += 10;
    player.save(ctx);
    Ok(())
}
