use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NodeRelation {
    Parent,
    Child,
}

#[derive(Clone, Copy, Debug)]
pub struct NodeLink {
    pub from: NodeKind,
    pub to: NodeKind,
    pub relation: NodeRelation,
}

impl NodeLink {
    const fn new(from: NodeKind, to: NodeKind, relation: NodeRelation) -> Self {
        Self { from, to, relation }
    }
}

pub fn get_node_relationships(kind: NodeKind) -> Vec<NodeLink> {
    match kind {
        NodeKind::NHouse => vec![
            NodeLink::new(
                NodeKind::NHouse,
                NodeKind::NHouseColor,
                NodeRelation::Parent,
            ),
            NodeLink::new(
                NodeKind::NHouse,
                NodeKind::NAbilityMagic,
                NodeRelation::Parent,
            ),
            NodeLink::new(
                NodeKind::NHouse,
                NodeKind::NStatusMagic,
                NodeRelation::Parent,
            ),
            NodeLink::new(NodeKind::NHouse, NodeKind::NUnit, NodeRelation::Child),
        ],
        NodeKind::NUnit => vec![
            NodeLink::new(
                NodeKind::NUnit,
                NodeKind::NUnitDescription,
                NodeRelation::Parent,
            ),
            NodeLink::new(NodeKind::NUnit, NodeKind::NUnitStats, NodeRelation::Parent),
            NodeLink::new(NodeKind::NUnit, NodeKind::NUnitState, NodeRelation::Parent),
            NodeLink::new(NodeKind::NUnit, NodeKind::NHouse, NodeRelation::Child),
        ],
        NodeKind::NUnitDescription => vec![
            NodeLink::new(
                NodeKind::NUnitDescription,
                NodeKind::NUnitRepresentation,
                NodeRelation::Parent,
            ),
            NodeLink::new(
                NodeKind::NUnitDescription,
                NodeKind::NUnitBehavior,
                NodeRelation::Parent,
            ),
            NodeLink::new(
                NodeKind::NUnitDescription,
                NodeKind::NUnit,
                NodeRelation::Child,
            ),
        ],
        NodeKind::NAbilityMagic => vec![
            NodeLink::new(
                NodeKind::NAbilityMagic,
                NodeKind::NAbilityDescription,
                NodeRelation::Parent,
            ),
            NodeLink::new(
                NodeKind::NAbilityMagic,
                NodeKind::NHouse,
                NodeRelation::Child,
            ),
        ],
        NodeKind::NAbilityDescription => vec![
            NodeLink::new(
                NodeKind::NAbilityDescription,
                NodeKind::NAbilityEffect,
                NodeRelation::Parent,
            ),
            NodeLink::new(
                NodeKind::NAbilityDescription,
                NodeKind::NAbilityMagic,
                NodeRelation::Child,
            ),
        ],
        NodeKind::NStatusMagic => vec![
            NodeLink::new(
                NodeKind::NStatusMagic,
                NodeKind::NStatusDescription,
                NodeRelation::Parent,
            ),
            NodeLink::new(
                NodeKind::NStatusMagic,
                NodeKind::NStatusRepresentation,
                NodeRelation::Parent,
            ),
            NodeLink::new(
                NodeKind::NStatusMagic,
                NodeKind::NHouse,
                NodeRelation::Child,
            ),
        ],
        NodeKind::NStatusDescription => vec![
            NodeLink::new(
                NodeKind::NStatusDescription,
                NodeKind::NStatusBehavior,
                NodeRelation::Parent,
            ),
            NodeLink::new(
                NodeKind::NStatusDescription,
                NodeKind::NStatusMagic,
                NodeRelation::Child,
            ),
        ],
        NodeKind::NHouseColor => vec![NodeLink::new(
            NodeKind::NHouseColor,
            NodeKind::NHouse,
            NodeRelation::Child,
        )],
        NodeKind::NUnitStats => vec![NodeLink::new(
            NodeKind::NUnitStats,
            NodeKind::NUnit,
            NodeRelation::Child,
        )],
        NodeKind::NUnitState => vec![NodeLink::new(
            NodeKind::NUnitState,
            NodeKind::NUnit,
            NodeRelation::Child,
        )],
        NodeKind::NUnitBehavior => vec![NodeLink::new(
            NodeKind::NUnitBehavior,
            NodeKind::NUnitDescription,
            NodeRelation::Child,
        )],
        NodeKind::NUnitRepresentation => vec![NodeLink::new(
            NodeKind::NUnitRepresentation,
            NodeKind::NUnitDescription,
            NodeRelation::Child,
        )],
        NodeKind::NAbilityEffect => vec![NodeLink::new(
            NodeKind::NAbilityEffect,
            NodeKind::NAbilityDescription,
            NodeRelation::Child,
        )],
        NodeKind::NStatusBehavior => vec![NodeLink::new(
            NodeKind::NStatusBehavior,
            NodeKind::NStatusDescription,
            NodeRelation::Child,
        )],
        NodeKind::NStatusRepresentation => vec![NodeLink::new(
            NodeKind::NStatusRepresentation,
            NodeKind::NStatusMagic,
            NodeRelation::Child,
        )],
        _ => vec![],
    }
}

pub fn get_named_parent(kind: NodeKind) -> Option<NamedNodeKind> {
    let relationships = get_node_relationships(kind);

    for link in relationships {
        if link.relation == NodeRelation::Child {
            if let Ok(named) = link.to.try_into() {
                return Some(named);
            }
            return get_named_parent(link.to);
        }
    }

    None
}

pub fn get_related_nodes(inspected: NamedNodeKind) -> Vec<NodeKind> {
    let mut related = Vec::new();

    match inspected {
        NamedNodeKind::NUnit => {
            related.push(NodeKind::NUnit);
            related.push(NodeKind::NUnitStats);
            related.push(NodeKind::NUnitDescription);
            related.push(NodeKind::NUnitBehavior);
            related.push(NodeKind::NUnitRepresentation);
            related.push(NodeKind::NUnitState);
        }
        NamedNodeKind::NHouse => {
            related.push(NodeKind::NHouse);
            related.push(NodeKind::NHouseColor);
            related.push(NodeKind::NAbilityMagic);
            related.push(NodeKind::NStatusMagic);
        }
        NamedNodeKind::NAbilityMagic => {
            related.push(NodeKind::NAbilityMagic);
            related.push(NodeKind::NAbilityDescription);
            related.push(NodeKind::NAbilityEffect);
        }
        NamedNodeKind::NStatusMagic => {
            related.push(NodeKind::NStatusMagic);
            related.push(NodeKind::NStatusDescription);
            related.push(NodeKind::NStatusBehavior);
            related.push(NodeKind::NStatusRepresentation);
        }
    }

    related
}
