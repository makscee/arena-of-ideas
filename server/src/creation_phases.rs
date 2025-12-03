use super::*;
use crate::nodes_table::creation_phases;

pub trait CreationPhasesHelper {
    fn base_id(self, kind: NodeKind, ctx: &ServerContext) -> NodeResult<u64>;
    fn fix_kind(self, ctx: &ServerContext, kind: NodeKind) -> NodeResult<()>;
    fn unfix_kind(self, ctx: &ServerContext, kind: NodeKind) -> NodeResult<()>;
    fn is_kind_fixed(self, ctx: &ServerContext, kind: NodeKind) -> bool;
}

impl CreationPhasesHelper for u64 {
    fn base_id(self, kind: NodeKind, ctx: &ServerContext) -> NodeResult<u64> {
        let base_kind = kind.base_kind();
        if kind == base_kind {
            return Ok(self);
        }
        ctx.first_parent_recursive(self, base_kind)
    }

    fn fix_kind(self, ctx: &ServerContext, kind: NodeKind) -> NodeResult<()> {
        let kind_str = kind.to_string();
        let mut cp = ctx
            .rctx()
            .db
            .creation_phases()
            .node_id()
            .find(self)
            .unwrap_or_else(|| {
                ctx.rctx().db.creation_phases().insert(TCreationPhases {
                    node_id: self,
                    fixed_kinds: default(),
                })
            });

        if !cp.fixed_kinds.contains(&kind_str) {
            cp.fixed_kinds.push(kind_str);
            ctx.rctx().db.creation_phases().node_id().update(cp);
        }
        Ok(())
    }

    fn unfix_kind(self, ctx: &ServerContext, kind: NodeKind) -> NodeResult<()> {
        if let Some(mut cp) = ctx.rctx().db.creation_phases().node_id().find(self) {
            let kind_str = kind.to_string();
            cp.fixed_kinds.retain(|k| k != &kind_str);
            ctx.rctx().db.creation_phases().node_id().update(cp);
        }
        Ok(())
    }

    fn is_kind_fixed(self, ctx: &ServerContext, kind: NodeKind) -> bool {
        ctx.rctx()
            .db
            .creation_phases()
            .node_id()
            .find(self)
            .map(|cp| cp.fixed_kinds.contains(&kind.to_string()))
            .unwrap_or(false)
    }
}

pub struct ComponentFixer;

impl ComponentFixer {
    pub fn fix_component(ctx: &ServerContext, node: &TNode) -> NodeResult<()> {
        let kind = node.kind();
        let base_kind = kind.base_kind();
        let base_id = if kind == base_kind {
            node.id
        } else {
            ctx.first_parent_recursive(node.id, base_kind)?
        };
        let fixed = base_id.fixed_kinds(ctx.rctx());
        if fixed.contains(&node.kind()) {
            return Ok(());
        }
        if kind == base_kind {
            return node.id.fix_kind(ctx, kind);
        }
        if let Some(parent_kind) = kind.component_parent() {
            if let Some(parent_link) = ctx
                .rctx()
                .db
                .node_links()
                .child_parent_kind()
                .filter((&node.id, &parent_kind.to_string()))
                .next()
            {
                let other_links: Vec<_> = ctx
                    .rctx()
                    .db
                    .node_links()
                    .parent_child_kind()
                    .filter((&parent_link.parent, &parent_link.child_kind))
                    .filter(|l| l.child != node.id)
                    .collect();

                for link in other_links {
                    if let Some(alt_node) = ctx.rctx().db.nodes_world().id().find(link.child) {
                        if alt_node.owner == ID_INCUBATOR {
                            TNode::delete_by_id_recursive(ctx.rctx(), link.child);
                        }
                    }
                }

                let base_id = ctx.first_parent_recursive(node.id, base_kind)?;

                base_id.fix_kind(ctx, kind)?;
            }
        }

        Ok(())
    }

    pub fn unfix_component(ctx: &ServerContext, node: &TNode) -> NodeResult<()> {
        let kind = node.kind();
        let base_id = node.id.base_id(kind, ctx);

        if let Some(parent_kind) = kind.component_parent() {
            if let Some(_parent_link) = ctx
                .rctx()
                .db
                .node_links()
                .child_parent_kind()
                .filter((&node.id, &parent_kind.to_string()))
                .next()
            {
                let base_id = ctx.first_parent_recursive(node.id, kind.base_kind())?;

                base_id.unfix_kind(ctx, kind)?;

                if node.owner == ID_CORE {
                    let mut unfixed_node = node.clone();
                    unfixed_node.owner = ID_INCUBATOR;
                    ctx.rctx().db.nodes_world().id().update(unfixed_node);
                }
            }
        }

        Ok(())
    }

    pub fn check_base_completion(ctx: &ServerContext, node: &TNode) -> NodeResult<()> {
        let kind = node.kind();

        let is_complete = match kind {
            NodeKind::NUnit => {
                let fixed = node.id.fixed_kinds(ctx.rctx());
                fixed.contains(&NodeKind::NUnitBehavior)
            }
            NodeKind::NHouse => {
                let fixed = node.id.fixed_kinds(ctx.rctx());
                let has_ability = fixed.contains(&NodeKind::NAbilityMagic);
                let has_status = fixed.contains(&NodeKind::NStatusMagic);
                fixed.contains(&NodeKind::NHouseColor) && (has_ability || has_status)
            }
            _ => false,
        };

        if is_complete {
            let mut node_mut = node.clone();
            node_mut.owner = ID_CORE;
            node_mut.update(ctx.rctx());

            for child_id in node.id.collect_children_recursive(ctx.rctx()) {
                if let Some(mut child_node) = ctx.rctx().db.nodes_world().id().find(child_id) {
                    if child_node.owner == ID_INCUBATOR {
                        child_node.owner = ID_CORE;
                        child_node.update(ctx.rctx());
                    }
                }
            }
        }

        Ok(())
    }
}
