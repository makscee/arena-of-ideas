use super::*;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct ExplorerCache {
    pub units: HashMap<String, (NUnit, NUnit)>,
    pub houses: HashMap<String, (NHouse, NHouse)>,
    pub abilities: HashMap<String, (NAbilityMagic, NAbilityMagic)>,
    pub statuses: HashMap<String, (NStatusMagic, NStatusMagic)>,

    pub unit_parents: HashMap<String, Vec<String>>,
    pub ability_parents: HashMap<String, Vec<String>>,
    pub status_parents: HashMap<String, Vec<String>>,

    pub unit_links: HashMap<String, (String, String)>,
    pub ability_links: HashMap<String, (String, String)>,
    pub status_links: HashMap<String, (String, String)>,
}

impl ExplorerCache {
    pub fn rebuild(&mut self) -> NodeResult<()> {
        *self = ExplorerCache::default();
        let ctx = &cn().db().as_context();
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
                    let mut unit = dbg!(ctx.load::<NUnit>(node.id)?.clone());
                    unit.load_all(ctx)?;
                    dbg!(&unit);
                    let unit = node.to_node::<NUnit>()?;
                    self.units
                        .insert(unit.name().to_string(), (unit.clone(), unit));
                }
            }
        }

        let mut unit_current_links: HashMap<String, TNodeLink> = HashMap::new();
        let mut unit_selected_links: HashMap<String, TNodeLink> = HashMap::new();
        let mut ability_current_links: HashMap<String, TNodeLink> = HashMap::new();
        let mut ability_selected_links: HashMap<String, TNodeLink> = HashMap::new();
        let mut status_current_links: HashMap<String, TNodeLink> = HashMap::new();
        let mut status_selected_links: HashMap<String, TNodeLink> = HashMap::new();

        for link in cn().db.node_links().iter() {
            if link.parent_kind == "NHouse" {
                match link.child_kind.as_str() {
                    "NUnit" => {
                        if let Some(child_node) = cn().db.nodes_world().id().find(&link.child) {
                            if let Ok(child_unit) = child_node.to_node::<NUnit>() {
                                let child_name = child_unit.name().to_string();

                                if link.solid {
                                    unit_selected_links
                                        .entry(child_name.clone())
                                        .and_modify(|existing| {
                                            if link.rating > existing.rating
                                                || (link.rating == existing.rating
                                                    && link.parent < existing.parent)
                                            {
                                                *existing = link.clone();
                                            }
                                        })
                                        .or_insert(link.clone());
                                } else {
                                    unit_current_links
                                        .entry(child_name.clone())
                                        .and_modify(|existing| {
                                            if link.rating > existing.rating
                                                || (link.rating == existing.rating
                                                    && link.parent < existing.parent)
                                            {
                                                *existing = link.clone();
                                            }
                                        })
                                        .or_insert(link.clone());
                                }
                            }
                        }
                    }
                    "NAbilityMagic" => {
                        if let Some(child_node) = cn().db.nodes_world().id().find(&link.child) {
                            if let Ok(child_ability) = child_node.to_node::<NAbilityMagic>() {
                                let child_name = child_ability.name().to_string();

                                if link.solid {
                                    ability_selected_links
                                        .entry(child_name.clone())
                                        .and_modify(|existing| {
                                            if link.rating > existing.rating
                                                || (link.rating == existing.rating
                                                    && link.parent < existing.parent)
                                            {
                                                *existing = link.clone();
                                            }
                                        })
                                        .or_insert(link.clone());
                                } else {
                                    ability_current_links
                                        .entry(child_name.clone())
                                        .and_modify(|existing| {
                                            if link.rating > existing.rating
                                                || (link.rating == existing.rating
                                                    && link.parent < existing.parent)
                                            {
                                                *existing = link.clone();
                                            }
                                        })
                                        .or_insert(link.clone());
                                }
                            }
                        }
                    }
                    "NStatusMagic" => {
                        if let Some(child_node) = cn().db.nodes_world().id().find(&link.child) {
                            if let Ok(child_status) = child_node.to_node::<NStatusMagic>() {
                                let child_name = child_status.name().to_string();

                                if link.solid {
                                    status_selected_links
                                        .entry(child_name.clone())
                                        .and_modify(|existing| {
                                            if link.rating > existing.rating
                                                || (link.rating == existing.rating
                                                    && link.parent < existing.parent)
                                            {
                                                *existing = link.clone();
                                            }
                                        })
                                        .or_insert(link.clone());
                                } else {
                                    status_current_links
                                        .entry(child_name.clone())
                                        .and_modify(|existing| {
                                            if link.rating > existing.rating
                                                || (link.rating == existing.rating
                                                    && link.parent < existing.parent)
                                            {
                                                *existing = link.clone();
                                            }
                                        })
                                        .or_insert(link.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

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

        for (child_name, current_link) in unit_current_links {
            if let Some(parent_node) = cn().db.nodes_world().id().find(&current_link.parent) {
                if let Ok(parent_house) = parent_node.to_node::<NHouse>() {
                    let current_parent = parent_house.name().to_string();
                    let selected_parent = unit_selected_links
                        .get(&child_name)
                        .and_then(|selected_link| {
                            cn().db
                                .nodes_world()
                                .id()
                                .find(&selected_link.parent)
                                .and_then(|parent_node| parent_node.to_node::<NHouse>().ok())
                                .map(|house| house.name().to_string())
                        })
                        .unwrap_or_else(|| current_parent.clone());

                    self.unit_links
                        .insert(child_name, (current_parent, selected_parent));
                }
            }
        }

        for (child_name, current_link) in ability_current_links {
            if let Some(parent_node) = cn().db.nodes_world().id().find(&current_link.parent) {
                if let Ok(parent_house) = parent_node.to_node::<NHouse>() {
                    let current_parent = parent_house.name().to_string();
                    let selected_parent = ability_selected_links
                        .get(&child_name)
                        .and_then(|selected_link| {
                            cn().db
                                .nodes_world()
                                .id()
                                .find(&selected_link.parent)
                                .and_then(|parent_node| parent_node.to_node::<NHouse>().ok())
                                .map(|house| house.name().to_string())
                        })
                        .unwrap_or_else(|| current_parent.clone());

                    self.ability_links
                        .insert(child_name, (current_parent, selected_parent));
                }
            }
        }

        for (child_name, current_link) in status_current_links {
            if let Some(parent_node) = cn().db.nodes_world().id().find(&current_link.parent) {
                if let Ok(parent_house) = parent_node.to_node::<NHouse>() {
                    let current_parent = parent_house.name().to_string();
                    let selected_parent = status_selected_links
                        .get(&child_name)
                        .and_then(|selected_link| {
                            cn().db
                                .nodes_world()
                                .id()
                                .find(&selected_link.parent)
                                .and_then(|parent_node| parent_node.to_node::<NHouse>().ok())
                                .map(|house| house.name().to_string())
                        })
                        .unwrap_or_else(|| current_parent.clone());

                    self.status_links
                        .insert(child_name, (current_parent, selected_parent));
                }
            }
        }

        Ok(())
    }
}
