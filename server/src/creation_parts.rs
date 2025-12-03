use super::*;

#[spacetimedb::table(name = creation_parts, public)]
pub struct TCreationParts {
    #[primary_key]
    pub node_id: u64,
    pub parts: Vec<String>,
}

pub trait CreationPartHelper {
    fn base_id(self, kind: NodeKind, ctx: &ServerContext) -> NodeResult<u64>;
    fn complete_part(self, ctx: &ServerContext, part: CreationPart) -> NodeResult<()>;
    fn uncomplete_part(self, ctx: &ServerContext, part: CreationPart) -> NodeResult<()>;
    fn check_part(self, ctx: &ServerContext, part: CreationPart) -> NodeResult<bool>;
    fn get_part(self, ctx: &ServerContext) -> NodeResult<Vec<CreationPart>>;
}

fn check_kind(ctx: &ServerContext, node_id: u64, part: CreationPart) -> NodeResult<()> {
    let expected = if part.is_unit() {
        NodeKind::NUnit
    } else {
        NodeKind::NHouse
    };
    let kind = node_id.kind(ctx.rctx()).to_not_found()?;
    if kind != expected {
        return Err(NodeError::custom(format!(
            "Wrong node kind, expected {expected} got {kind}"
        )));
    }
    Ok(())
}

impl CreationPartHelper for u64 {
    fn base_id(self, kind: NodeKind, ctx: &ServerContext) -> NodeResult<u64> {
        let base_kind = kind.base_kind();
        if kind == base_kind {
            return Ok(self);
        }
        ctx.first_parent_recursive(self, base_kind)
    }

    fn complete_part(self, ctx: &ServerContext, part: CreationPart) -> NodeResult<()> {
        check_kind(ctx, self, part)?;
        let mut cp = ctx
            .rctx()
            .db
            .creation_parts()
            .node_id()
            .find(self)
            .unwrap_or_else(|| {
                ctx.rctx().db.creation_parts().insert(TCreationParts {
                    node_id: self,
                    parts: default(),
                })
            });
        let part = part.as_ref().to_string();
        if !cp.parts.contains(&part) {
            cp.parts.push(part);
            ctx.rctx().db.creation_parts().node_id().update(cp);
        }
        Ok(())
    }

    fn uncomplete_part(self, ctx: &ServerContext, part: CreationPart) -> NodeResult<()> {
        check_kind(ctx, self, part)?;
        if let Some(mut cp) = ctx.rctx().db.creation_parts().node_id().find(self) {
            let part_str = part.as_ref().to_string();
            cp.parts.retain(|k| k != &part_str);
            ctx.rctx().db.creation_parts().node_id().update(cp);
        }
        Ok(())
    }

    fn get_part(self, ctx: &ServerContext) -> NodeResult<Vec<CreationPart>> {
        Ok(ctx
            .rctx()
            .db
            .creation_parts()
            .node_id()
            .find(self)
            .to_not_found()?
            .parts
            .into_iter()
            .map(|p| CreationPart::from_str(&p).unwrap())
            .collect_vec())
    }

    fn check_part(self, ctx: &ServerContext, part: CreationPart) -> NodeResult<bool> {
        check_kind(ctx, self, part)?;
        Ok(self.get_part(ctx)?.contains(&part))
    }
}

impl TNode {
    pub fn get_creation_part(&self) -> NodeResult<CreationPart> {
        let kind = self.kind();
        let part = match kind {
            NodeKind::NHouse => CreationPart::HouseName,
            NodeKind::NHouseColor => CreationPart::HouseColor,
            NodeKind::NAbilityMagic => CreationPart::AbilityName,
            NodeKind::NAbilityEffect => {
                let node = self.to_node::<NAbilityEffect>()?;
                if node.effect.actions.is_empty() {
                    CreationPart::AbilityDescription
                } else {
                    CreationPart::AbilityImplementation
                }
            }
            NodeKind::NStatusMagic => CreationPart::StatusName,
            NodeKind::NStatusBehavior => {
                let node = self.to_node::<NStatusBehavior>()?;
                if node.reactions.iter().all(|r| !r.effect.actions.is_empty()) {
                    CreationPart::StatusImplementation
                } else {
                    CreationPart::StatusDescription
                }
            }
            NodeKind::NUnit => CreationPart::UnitName,
            NodeKind::NUnitBehavior => {
                let node = self.to_node::<NUnitBehavior>()?;
                if node.reactions.iter().all(|r| !r.effect.actions.is_empty()) {
                    CreationPart::UnitImplementation
                } else {
                    CreationPart::UnitDescription
                }
            }
            NodeKind::NUnitStats => CreationPart::UnitStats,
            NodeKind::NUnitRepresentation => CreationPart::UnitRepresentation,
            _ => {
                return Err(NodeError::custom(format!(
                    "Invalid node kind for creation part: {}",
                    self.kind()
                )));
            }
        };
        Ok(part)
    }
}

impl TCreationParts {
    pub fn complete_node_part(
        ctx: &ServerContext,
        node: &TNode,
        part: CreationPart,
    ) -> NodeResult<()> {
        let kind = node.kind();
        let base_id = node.id.base_id(kind, ctx)?;
        if base_id.check_part(ctx, part)? {
            return Err(NodeError::custom(format!(
                "Part {part} already complete for {base_id}"
            )));
        }
        let Some(parent_kind) = kind.component_parent() else {
            return Err(NodeError::custom(format!(
                "No parent kind found for {kind}"
            )));
        };
        let parent_id = node
            .id
            .get_kind_parent(ctx.rctx(), parent_kind)
            .to_not_found()?;

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
        base_id.complete_part(ctx, part)?;
        Ok(())
    }

    pub fn uncomplete_node_part(ctx: &ServerContext, node: &TNode) -> NodeResult<()> {
        let kind = node.kind();
        let base_id = node.id.base_id(kind, ctx)?;
        let parts = match kind {
            NodeKind::NHouse => [CreationPart::HouseName].to_vec(),
            NodeKind::NHouseColor => [CreationPart::HouseColor].to_vec(),
            NodeKind::NAbilityMagic => [CreationPart::AbilityName].to_vec(),
            NodeKind::NAbilityEffect => [
                CreationPart::AbilityDescription,
                CreationPart::AbilityImplementation,
            ]
            .to_vec(),
            NodeKind::NStatusMagic => [CreationPart::StatusName].to_vec(),
            NodeKind::NStatusBehavior => [
                CreationPart::StatusDescription,
                CreationPart::StatusImplementation,
            ]
            .to_vec(),
            NodeKind::NUnit => [CreationPart::UnitName].to_vec(),
            NodeKind::NUnitBehavior => [
                CreationPart::UnitDescription,
                CreationPart::UnitImplementation,
            ]
            .to_vec(),
            NodeKind::NUnitStats => [CreationPart::UnitStats].to_vec(),
            NodeKind::NUnitRepresentation => [CreationPart::UnitRepresentation].to_vec(),
            _ => return Err(NodeError::custom(format!("Invalid part kind {kind}"))),
        };
        for part in parts {
            base_id.uncomplete_part(ctx, part)?;
        }
        Ok(())
    }

    pub fn check_base_completion(ctx: &ServerContext, node: &TNode) -> NodeResult<()> {
        let kind = node.kind();
        let base_id = node.id.base_id(kind, ctx)?;
        let base_kind = base_id.kind(ctx.rctx()).to_not_found()?;

        let is_complete =
            CreationPart::is_complete(&base_id.get_part(ctx)?, base_kind == NodeKind::NUnit);
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
