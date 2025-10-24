use super::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct ExplorerCache {
    pub units: HashMap<String, (NUnit, NUnit)>,
    pub houses: HashMap<String, (NHouse, NHouse)>,
    pub abilities: HashMap<String, (NAbilityMagic, NAbilityMagic)>,
    pub statuses: HashMap<String, (NStatusMagic, NStatusMagic)>,

    pub unit_parents: HashMap<String, Vec<String>>,
    pub ability_parents: HashMap<String, Vec<String>>,
    pub status_parents: HashMap<String, Vec<String>>,
}

impl ExplorerCache {
    pub fn rebuild(&mut self) -> NodeResult<()> {
        *self = ExplorerCache::default();

        for node in cn().db.nodes_world().iter() {
            if node.owner != 0 && node.owner != ID_CORE {
                continue;
            }
            let Ok(kind) = node.kind().to_named() else {
                continue;
            };
            match kind {
                NamedNodeKind::NHouse => {
                    let house = node.to_node::<NHouse>()?;
                    self.houses
                        .insert(house.name().to_string(), (house.clone(), house));
                }
                NamedNodeKind::NAbilityMagic => {
                    let ability = node.to_node::<NAbilityMagic>()?;
                    self.abilities
                        .insert(ability.name().to_string(), (ability.clone(), ability));
                }
                NamedNodeKind::NStatusMagic => {
                    let status = node.to_node::<NStatusMagic>()?;
                    self.statuses
                        .insert(status.name().to_string(), (status.clone(), status));
                }
                NamedNodeKind::NUnit => {
                    let unit = node.to_node::<NUnit>()?;
                    self.units
                        .insert(unit.name().to_string(), (unit.clone(), unit));
                }
            }
        }

        for link in cn().db.node_links().iter() {
            if let (Some(child_node), Some(parent_node)) = (
                cn().db.nodes_world().id().find(&link.child),
                cn().db.nodes_world().id().find(&link.parent),
            ) {
                if let (Ok(child_kind), Ok(parent_kind)) = (
                    NodeKind::try_from(child_node.kind.as_str()),
                    NodeKind::try_from(parent_node.kind.as_str()),
                ) {
                    if parent_kind == NodeKind::NHouse {
                        if let Ok(parent_house) = parent_node.to_node::<NHouse>() {
                            let parent_name = parent_house.name().to_string();
                            match child_kind {
                                NodeKind::NUnit => {
                                    if let Ok(child_unit) = child_node.to_node::<NUnit>() {
                                        let child_name = child_unit.name().to_string();
                                        self.unit_parents
                                            .entry(child_name)
                                            .or_default()
                                            .push(parent_name);
                                    }
                                }
                                NodeKind::NAbilityMagic => {
                                    if let Ok(child_ability) = child_node.to_node::<NAbilityMagic>()
                                    {
                                        let child_name = child_ability.name().to_string();
                                        self.ability_parents
                                            .entry(child_name)
                                            .or_default()
                                            .push(parent_name);
                                    }
                                }
                                NodeKind::NStatusMagic => {
                                    if let Ok(child_status) = child_node.to_node::<NStatusMagic>() {
                                        let child_name = child_status.name().to_string();
                                        self.status_parents
                                            .entry(child_name)
                                            .or_default()
                                            .push(parent_name);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
