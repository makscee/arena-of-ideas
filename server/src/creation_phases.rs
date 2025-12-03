use super::*;

#[spacetimedb::table(name = creation_phases, public)]
pub struct TCreationPhases {
    #[primary_key]
    pub node_id: u64,
    pub phases: Vec<String>,
}

pub trait CreationPhasesHelper {
    fn base_id(self, kind: NodeKind, ctx: &ServerContext) -> NodeResult<u64>;
    fn complete_phase(self, ctx: &ServerContext, phase: CreationPhase) -> NodeResult<()>;
    fn uncomplete_phase(self, ctx: &ServerContext, phase: CreationPhase) -> NodeResult<()>;
    fn check_phase(self, ctx: &ServerContext, phase: CreationPhase) -> NodeResult<bool>;
    fn get_phases(self, ctx: &ServerContext) -> NodeResult<Vec<CreationPhase>>;
}

fn check_kind(ctx: &ServerContext, node_id: u64, phase: CreationPhase) -> NodeResult<()> {
    let expected = if phase.is_unit() {
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

impl CreationPhasesHelper for u64 {
    fn base_id(self, kind: NodeKind, ctx: &ServerContext) -> NodeResult<u64> {
        let base_kind = kind.base_kind();
        if kind == base_kind {
            return Ok(self);
        }
        ctx.first_parent_recursive(self, base_kind)
    }

    fn complete_phase(self, ctx: &ServerContext, phase: CreationPhase) -> NodeResult<()> {
        check_kind(ctx, self, phase)?;
        let mut cp = ctx
            .rctx()
            .db
            .creation_phases()
            .node_id()
            .find(self)
            .unwrap_or_else(|| {
                ctx.rctx().db.creation_phases().insert(TCreationPhases {
                    node_id: self,
                    phases: default(),
                })
            });
        let phase = phase.as_ref().to_string();
        if !cp.phases.contains(&phase) {
            cp.phases.push(phase);
            ctx.rctx().db.creation_phases().node_id().update(cp);
        }
        Ok(())
    }

    fn uncomplete_phase(self, ctx: &ServerContext, phase: CreationPhase) -> NodeResult<()> {
        check_kind(ctx, self, phase)?;
        if let Some(mut cp) = ctx.rctx().db.creation_phases().node_id().find(self) {
            let phase_str = phase.as_ref().to_string();
            cp.phases.retain(|k| k != &phase_str);
            ctx.rctx().db.creation_phases().node_id().update(cp);
        }
        Ok(())
    }

    fn get_phases(self, ctx: &ServerContext) -> NodeResult<Vec<CreationPhase>> {
        Ok(ctx
            .rctx()
            .db
            .creation_phases()
            .node_id()
            .find(self)
            .to_not_found()?
            .phases
            .into_iter()
            .map(|p| CreationPhase::from_str(&p).unwrap())
            .collect_vec())
    }

    fn check_phase(self, ctx: &ServerContext, phase: CreationPhase) -> NodeResult<bool> {
        check_kind(ctx, self, phase)?;
        Ok(self.get_phases(ctx)?.contains(&phase))
    }
}

impl TNode {
    pub fn get_creation_phase(&self) -> NodeResult<CreationPhase> {
        let kind = self.kind();
        let phase = match kind {
            NodeKind::NHouse => CreationPhase::HouseName,
            NodeKind::NHouseColor => CreationPhase::HouseColor,
            NodeKind::NAbilityMagic => CreationPhase::AbilityName,
            NodeKind::NAbilityEffect => {
                let node = self.to_node::<NAbilityEffect>()?;
                if node.effect.actions.is_empty() {
                    CreationPhase::AbilityDescription
                } else {
                    CreationPhase::AbilityImplementation
                }
            }
            NodeKind::NStatusMagic => CreationPhase::StatusName,
            NodeKind::NStatusBehavior => {
                let node = self.to_node::<NStatusBehavior>()?;
                if node.reactions.iter().all(|r| !r.effect.actions.is_empty()) {
                    CreationPhase::StatusImplementation
                } else {
                    CreationPhase::StatusDescription
                }
            }
            NodeKind::NUnit => CreationPhase::UnitName,
            NodeKind::NUnitBehavior => {
                let node = self.to_node::<NUnitBehavior>()?;
                if node.reactions.iter().all(|r| !r.effect.actions.is_empty()) {
                    CreationPhase::UnitImplementation
                } else {
                    CreationPhase::UnitDescription
                }
            }
            NodeKind::NUnitStats => CreationPhase::UnitStats,
            NodeKind::NUnitRepresentation => CreationPhase::UnitRepresentation,
            _ => {
                return Err(NodeError::custom(format!(
                    "Invalid node kind for creation phase: {}",
                    self.kind()
                )));
            }
        };
        Ok(phase)
    }
}

impl TCreationPhases {
    pub fn complete_node_phase(
        ctx: &ServerContext,
        node: &TNode,
        phase: CreationPhase,
    ) -> NodeResult<()> {
        let kind = node.kind();
        let base_id = node.id.base_id(kind, ctx)?;
        if base_id.check_phase(ctx, phase)? {
            return Err(NodeError::custom(format!(
                "Phase {phase} already complete for {base_id}"
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
        base_id.complete_phase(ctx, phase)?;
        Ok(())
    }

    pub fn uncomplete_node_phase(ctx: &ServerContext, node: &TNode) -> NodeResult<()> {
        let kind = node.kind();
        let base_id = node.id.base_id(kind, ctx)?;
        let phases = match kind {
            NodeKind::NHouse => [CreationPhase::HouseName].to_vec(),
            NodeKind::NHouseColor => [CreationPhase::HouseColor].to_vec(),
            NodeKind::NAbilityMagic => [CreationPhase::AbilityName].to_vec(),
            NodeKind::NAbilityEffect => [
                CreationPhase::AbilityDescription,
                CreationPhase::AbilityImplementation,
            ]
            .to_vec(),
            NodeKind::NStatusMagic => [CreationPhase::StatusName].to_vec(),
            NodeKind::NStatusBehavior => [
                CreationPhase::StatusDescription,
                CreationPhase::StatusImplementation,
            ]
            .to_vec(),
            NodeKind::NUnit => [CreationPhase::UnitName].to_vec(),
            NodeKind::NUnitBehavior => [
                CreationPhase::UnitDescription,
                CreationPhase::UnitImplementation,
            ]
            .to_vec(),
            NodeKind::NUnitStats => [CreationPhase::UnitStats].to_vec(),
            NodeKind::NUnitRepresentation => [CreationPhase::UnitRepresentation].to_vec(),
            _ => return Err(NodeError::custom(format!("Invalid phase kind {kind}"))),
        };
        for phase in phases {
            base_id.uncomplete_phase(ctx, phase)?;
        }
        Ok(())
    }

    pub fn check_base_completion(ctx: &ServerContext, node: &TNode) -> NodeResult<()> {
        let kind = node.kind();
        let base_id = node.id.base_id(kind, ctx)?;
        let base_kind = base_id.kind(ctx.rctx()).to_not_found()?;

        let is_complete =
            CreationPhase::is_complete(&base_id.get_phases(ctx)?, base_kind == NodeKind::NUnit);
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
