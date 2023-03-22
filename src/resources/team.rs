use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Team {
    pub name: String,
    pub units: Vec<PackedUnit>,
    #[serde(default)]
    pub ability_state: AbilitiesState,
}

impl Team {
    pub fn empty(name: String) -> Self {
        Self { name, ..default() }
    }

    pub fn new(name: String, units: Vec<PackedUnit>) -> Self {
        Self {
            name,
            units,
            ..default()
        }
    }

    pub fn unpack_entries(
        &self,
        faction: &Faction,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        self.units.iter().enumerate().for_each(|(slot, unit)| {
            let slot = slot + 1;
            let position = SlotSystem::get_position(slot, faction);
            unit.unpack(world, resources, slot + 1, *faction, Some(position));
        });
    }

    pub fn pack_entries(
        faction: &Faction,
        world: &legion::World,
        resources: &Resources,
    ) -> Vec<PackedUnit> {
        UnitSystem::collect_faction(world, *faction)
            .iter()
            .sorted_by_key(|(_, unit)| unit.slot)
            .map(|(entity, _)| PackedUnit::pack(*entity, world, resources))
            .collect_vec()
    }

    pub fn try_get_ability_var_int(
        &self,
        house: &HouseName,
        ability: &str,
        var: &VarName,
    ) -> Option<i32> {
        self.ability_state
            .get_vars(house, ability)
            .and_then(|vars| vars.try_get_int(var))
    }
}
