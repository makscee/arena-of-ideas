use super::*;

use spacetimedb::Identity;
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

impl AdminCheck for &ServerContext<'_> {
    fn is_admin(self) -> Result<(), String> {
        if is_admin(&self.rctx().sender)? {
            Ok(())
        } else {
            Err("Need admin access".to_owned())
        }
    }
}

#[reducer]
fn admin_daily_update(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &ctx.as_context();
    ctx.is_admin()?;
    crate::daily_updater::daily_update(ctx)
}

#[reducer]
fn admin_delete_node_recursive(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let ctx = &ctx.as_context();
    ctx.is_admin()?;
    TNode::delete_by_id_recursive(ctx.rctx(), id);
    Ok(())
}

#[reducer]
fn admin_upload_world(
    ctx: &ReducerContext,
    global_settings: GlobalSettings,
    nodes: Vec<String>,
    links: Vec<String>,
) -> Result<(), String> {
    let ctx = &ctx.as_context();
    GlobalData::init(ctx);
    ctx.is_admin()?;
    global_settings.replace(ctx);
    let rctx = ctx.rctx();
    for node in rctx.db.nodes_world().iter() {
        rctx.db.nodes_world().id().delete(node.id);
    }
    for link in rctx.db.node_links().iter() {
        rctx.db.node_links().id().delete(link.id);
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
        .insert(ctx.rctx());
    }
    for link in links {
        let (parent, child, parent_kind, child_kind) =
            ron::from_str::<LinkAsset>(&link).map_err(|e| e.to_string())?;
        TNodeLink {
            id: ctx.next_id(),
            parent,
            child,
            parent_kind,
            child_kind,
        }
        .insert(ctx.rctx());
    }
    init(ctx.rctx())?;
    Ok(())
}

#[reducer]
fn admin_add_gold(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    ctx.is_admin()?;
    let player = ctx.player()?;
    let mut m = player.active_match.load_node(ctx)?;
    m.g += 10;
    ctx.source_mut().commit(m)?;
    Ok(())
}

#[reducer]
fn admin_add_votes(ctx: &ReducerContext, amount: i32) -> Result<(), String> {
    let ctx = &ctx.as_context();
    ctx.is_admin()?;
    let player = ctx.player()?;
    TVotes::add_votes(ctx, player.id, amount);
    Ok(())
}

#[reducer]
fn admin_edit_nodes(ctx: &ReducerContext, pack: String) -> Result<(), String> {
    let ctx = &ctx.as_context();
    ctx.is_admin()?;
    let pack = PackedNodes::from_string(&pack)?;
    for (id, data) in pack.nodes {
        let mut node = TNode::load(ctx.rctx(), id).to_not_found()?;
        node.data = data.data;
        node.update(ctx.rctx());
    }

    Ok(())
}

#[reducer]
fn admin_edit_owner(ctx: &ReducerContext, node_id: u64, owner_id: u64) -> Result<(), String> {
    let ctx = &ctx.as_context();
    ctx.is_admin()?;

    let mut node = ctx
        .rctx()
        .db
        .nodes_world()
        .id()
        .find(node_id)
        .ok_or_else(|| format!("Node {} not found", node_id))?;
    node.owner = owner_id;
    ctx.rctx().db.nodes_world().id().update(node);
    for child_id in node_id.collect_children_recursive(ctx.rctx()) {
        if let Some(mut child_node) = ctx.rctx().db.nodes_world().id().find(child_id) {
            child_node.owner = owner_id;
            ctx.rctx().db.nodes_world().id().update(child_node);
        }
    }

    Ok(())
}
