use super::*;
use std::collections::{HashMap, HashSet};

#[derive(Default, Debug)]
pub struct ExplorerCache {
    pub units: HashMap<String, (NUnit, NUnit)>,
    pub houses: HashMap<String, (NHouse, NHouse)>,
    pub abilities: HashMap<String, (NAbilityMagic, NAbilityMagic)>,
    pub statuses: HashMap<String, (NStatusMagic, NStatusMagic)>,

    pub unit_parents: HashMap<String, (String, String)>,
    pub ability_parents: HashMap<String, (String, String)>,
    pub status_parents: HashMap<String, (String, String)>,

    pub house_ability_children: HashMap<String, String>,
    pub house_status_children: HashMap<String, String>,
    pub house_units_children: HashMap<String, HashSet<String>>,
}

impl ExplorerCache {
    pub fn rebuild(&mut self) -> NodeResult<()> {
        *self = ExplorerCache::default();

        cn().db()
            .with_context_strategy(DbLinkStrategy::TopRating, |top_ctx| {
                cn().db()
                    .with_context_strategy(DbLinkStrategy::PlayerSelection, |player_ctx| {
                        for node in cn().db.nodes_world().iter() {
                            if node.owner != 0 && node.owner != ID_CORE {
                                continue;
                            }
                            let Ok(kind) = node.kind().to_named() else {
                                continue;
                            };
                            match kind {
                                NamedNodeKind::NHouse => {
                                    let top_house = top_ctx
                                        .load::<NHouse>(node.id)?
                                        .load_components(top_ctx)?
                                        .take();
                                    let player_house = player_ctx
                                        .load::<NHouse>(node.id)?
                                        .load_components(player_ctx)?
                                        .take();
                                    self.houses.insert(
                                        top_house.name().to_string(),
                                        (top_house, player_house),
                                    );
                                }
                                NamedNodeKind::NUnit => {
                                    let top_unit = top_ctx
                                        .load::<NUnit>(node.id)?
                                        .load_components(top_ctx)?
                                        .take();
                                    let player_unit = player_ctx
                                        .load::<NUnit>(node.id)?
                                        .load_components(player_ctx)?
                                        .take();
                                    let unit_name = top_unit.name().to_string();

                                    let top_parent = top_ctx
                                        .load_first_parent::<NHouse>(top_unit.id)
                                        .map(|h| h.name().to_string())
                                        .unwrap_or_default();
                                    let player_parent = player_ctx
                                        .load_first_parent::<NHouse>(player_unit.id)
                                        .map(|h| h.name().to_string())
                                        .unwrap_or_else(|_| top_parent.clone());

                                    self.units
                                        .insert(unit_name.clone(), (top_unit, player_unit));
                                    self.unit_parents.insert(
                                        unit_name.clone(),
                                        (top_parent.clone(), player_parent.clone()),
                                    );

                                    self.house_units_children
                                        .entry(top_parent.clone())
                                        .or_insert_with(HashSet::new)
                                        .insert(unit_name.clone());
                                    if player_parent != top_parent {
                                        self.house_units_children
                                            .entry(player_parent)
                                            .or_insert_with(HashSet::new)
                                            .insert(unit_name);
                                    }
                                }
                                NamedNodeKind::NAbilityMagic => {
                                    let top_ability = top_ctx
                                        .load::<NAbilityMagic>(node.id)?
                                        .load_components(top_ctx)?
                                        .take();
                                    let player_ability = player_ctx
                                        .load::<NAbilityMagic>(node.id)?
                                        .load_components(player_ctx)?
                                        .take();
                                    let ability_name = top_ability.name().to_string();

                                    let top_parent = top_ctx
                                        .load_first_parent::<NHouse>(top_ability.id)
                                        .map(|h| h.name().to_string())
                                        .unwrap_or_default();
                                    let player_parent = player_ctx
                                        .load_first_parent::<NHouse>(player_ability.id)
                                        .map(|h| h.name().to_string())
                                        .unwrap_or_else(|_| top_parent.clone());

                                    self.abilities.insert(
                                        ability_name.clone(),
                                        (top_ability, player_ability),
                                    );
                                    self.ability_parents.insert(
                                        ability_name.clone(),
                                        (top_parent.clone(), player_parent.clone()),
                                    );

                                    if !top_parent.is_empty() {
                                        self.house_ability_children
                                            .insert(top_parent.clone(), ability_name.clone());
                                    }
                                    if !player_parent.is_empty() && player_parent != top_parent {
                                        self.house_ability_children
                                            .insert(player_parent, ability_name);
                                    }
                                }
                                NamedNodeKind::NStatusMagic => {
                                    let top_status = top_ctx
                                        .load::<NStatusMagic>(node.id)?
                                        .load_components(top_ctx)?
                                        .take();
                                    let player_status = player_ctx
                                        .load::<NStatusMagic>(node.id)?
                                        .load_components(player_ctx)?
                                        .take();
                                    let status_name = top_status.name().to_string();

                                    let top_parent = top_ctx
                                        .load_first_parent::<NHouse>(top_status.id)
                                        .map(|h| h.name().to_string())
                                        .unwrap_or_default();
                                    let player_parent = player_ctx
                                        .load_first_parent::<NHouse>(player_status.id)
                                        .map(|h| h.name().to_string())
                                        .unwrap_or_else(|_| top_parent.clone());

                                    self.statuses
                                        .insert(status_name.clone(), (top_status, player_status));
                                    self.status_parents.insert(
                                        status_name.clone(),
                                        (top_parent.clone(), player_parent.clone()),
                                    );

                                    if !top_parent.is_empty() {
                                        self.house_status_children
                                            .insert(top_parent.clone(), status_name.clone());
                                    }
                                    if !player_parent.is_empty() && player_parent != top_parent {
                                        self.house_status_children
                                            .insert(player_parent, status_name);
                                    }
                                }
                            }
                        }
                        Ok(())
                    })
            })?;

        Ok(())
    }
}
