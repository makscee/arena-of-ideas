use super::*;

use spacetimedb::{Identity, ReducerContext, reducer};
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
        let (parent, child, parent_kind, child_kind) =
            ron::from_str::<LinkAsset>(&link).map_err(|e| e.to_string())?;
        TNodeLink {
            id: ctx.as_context().next_id(),
            parent,
            child,
            parent_kind,
            child_kind,
        }
        .insert(ctx);
    }
    init(ctx)?;
    Ok(())
}

#[reducer]
fn admin_add_gold(ctx: &ReducerContext) -> Result<(), String> {
    ctx.is_admin()?;
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let mut m = player.active_match.load_node(ctx)?;
    m.g += 10;
    ctx.source_mut().commit(m)?;
    Ok(())
}

#[reducer]
fn admin_add_votes(ctx: &ReducerContext, amount: i32) -> Result<(), String> {
    ctx.is_admin()?;
    let player = ctx.as_context().player()?;
    TVotes::add_votes(ctx, player.id, amount);
    Ok(())
}
