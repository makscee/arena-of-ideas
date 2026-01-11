use super::*;

#[spacetimedb::table(name = creation_parts, public)]
pub struct TCreationParts {
    #[primary_key]
    pub node_id: u64,
    pub kinds: Vec<String>,
}

pub trait CreationPartHelper {
    fn base_id(self, ctx: &ServerContext) -> NodeResult<u64>;
    fn complete_kind(self, ctx: &ServerContext, kind: ContentNodeKind) -> NodeResult<()>;
    fn uncomplete_kind(self, ctx: &ServerContext, kind: ContentNodeKind) -> NodeResult<()>;
    fn check_kind(self, ctx: &ServerContext, kind: ContentNodeKind) -> NodeResult<bool>;
    fn get_completed_kinds(self, ctx: &ServerContext) -> NodeResult<Vec<ContentNodeKind>>;
}

impl CreationPartHelper for u64 {
    fn base_id(self, ctx: &ServerContext) -> NodeResult<u64> {
        ctx.first_parent_recursive(self, NodeKind::NUnit)
            .or_else(|_| ctx.first_parent_recursive(self, NodeKind::NHouse))
    }

    fn complete_kind(self, ctx: &ServerContext, kind: ContentNodeKind) -> NodeResult<()> {
        let mut cp = ctx
            .rctx()
            .db
            .creation_parts()
            .node_id()
            .find(self)
            .unwrap_or_else(|| {
                ctx.rctx().db.creation_parts().insert(TCreationParts {
                    node_id: self,
                    kinds: default(),
                })
            });
        let kind_str = kind.as_ref().to_string();
        if !cp.kinds.contains(&kind_str) {
            cp.kinds.push(kind_str);
            ctx.rctx().db.creation_parts().node_id().update(cp);
        }
        Ok(())
    }

    fn uncomplete_kind(self, ctx: &ServerContext, kind: ContentNodeKind) -> NodeResult<()> {
        if let Some(mut cp) = ctx.rctx().db.creation_parts().node_id().find(self) {
            let kind_str = kind.as_ref().to_string();
            cp.kinds.retain(|k| k != &kind_str);
            ctx.rctx().db.creation_parts().node_id().update(cp);
        }
        Ok(())
    }

    fn get_completed_kinds(self, ctx: &ServerContext) -> NodeResult<Vec<ContentNodeKind>> {
        Ok(ctx
            .rctx()
            .db
            .creation_parts()
            .node_id()
            .find(self)
            .map(|cp| {
                cp.kinds
                    .into_iter()
                    .filter_map(|k| ContentNodeKind::from_str(&k).ok())
                    .collect_vec()
            })
            .unwrap_or_default())
    }

    fn check_kind(self, ctx: &ServerContext, kind: ContentNodeKind) -> NodeResult<bool> {
        Ok(self.get_completed_kinds(ctx)?.contains(&kind))
    }
}

impl TNode {
    pub fn get_content_kind(&self) -> NodeResult<ContentNodeKind> {
        let kind = self.kind();
        ContentNodeKind::try_from(kind)
            .map_err(|_| NodeError::custom(format!("Invalid content node kind: {}", self.kind())))
    }
}

impl TCreationParts {
    pub fn complete_node_part(ctx: &ServerContext, node: &TNode) -> NodeResult<()> {
        let kind = node.kind();
        let base_id = node.id.base_id(ctx)?;

        let content_kind = node.get_content_kind()?;
        if base_id.check_kind(ctx, content_kind)? {
            return Ok(());
        }

        // Get all parents and check if any have this node as a component child
        let parents = ctx
            .rctx()
            .db
            .node_links()
            .child()
            .filter(&node.id)
            .collect_vec();
        let mut parent_id = None;

        for link in &parents {
            if link
                .child_kind
                .to_kind()
                .is_component_child(link.parent_kind.to_kind())
            {
                parent_id = Some(link.parent);
            }
        }

        let Some(parent_id) = parent_id else {
            base_id.complete_kind(ctx, content_kind)?;
            return Ok(());
        };

        let other_links: Vec<_> = ctx
            .rctx()
            .db
            .node_links()
            .parent_child_kind()
            .filter((&parent_id, &kind.to_string()))
            .filter(|l| l.child != node.id)
            .collect();

        for link in other_links {
            if let Some(alt_node) = ctx.rctx().db.nodes_world().id().find(link.child) {
                if alt_node.owner == ID_INCUBATOR {
                    TNode::delete_by_id_recursive(ctx.rctx(), link.child);
                }
            }
        }
        base_id.complete_kind(ctx, content_kind)?;
        // TCreationParts::check_base_completion(ctx, node)?;
        Ok(())
    }

    pub fn uncomplete_node_part(ctx: &ServerContext, node: &TNode) -> NodeResult<()> {
        let base_id = node.id.base_id(ctx)?;
        let content_kind = node.get_content_kind()?;
        base_id.uncomplete_kind(ctx, content_kind)?;
        Ok(())
    }

    pub fn check_base_completion(ctx: &ServerContext, node: &TNode) -> NodeResult<()> {
        let base_id = node.id.base_id(ctx)?;
        let base_kind = base_id.kind(ctx.rctx()).to_not_found()?;

        let completed_kinds = base_id.get_completed_kinds(ctx)?;
        let is_complete = Self::is_complete(&completed_kinds, base_kind);

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

    fn is_complete(kinds: &Vec<ContentNodeKind>, base_kind: NodeKind) -> bool {
        match base_kind {
            NodeKind::NUnit => {
                kinds.contains(&ContentNodeKind::NUnit)
                    && kinds.contains(&ContentNodeKind::NUnitBehavior)
                    && kinds.contains(&ContentNodeKind::NUnitStats)
            }
            NodeKind::NHouse => {
                kinds.contains(&ContentNodeKind::NHouse)
                    && kinds.contains(&ContentNodeKind::NHouseColor)
                    && (kinds.contains(&ContentNodeKind::NAbilityMagic)
                        && kinds.contains(&ContentNodeKind::NAbilityEffect)
                        || kinds.contains(&ContentNodeKind::NStatusMagic)
                            && kinds.contains(&ContentNodeKind::NStatusBehavior))
            }
            _ => false,
        }
    }
}
