use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Team {
    pub name: String,
    pub units: Vec<SerializedUnit>,
    #[serde(default)]
    pub ability_state: AbilitiesState,
}

impl Team {
    pub fn unpack_entries(
        from: Faction,
        into: Faction,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        let units = resources.teams.get_mut(&from).unwrap().units.to_owned();
        units.iter().enumerate().for_each(|(slot, unit)| {
            unit.unpack(world, resources, slot + 1, into);
        });
        resources.teams.get_mut(&from).unwrap().units = units;
    }

    pub fn pack_entries(faction: &Faction, world: &legion::World, resources: &mut Resources) {
        let mut team = resources.teams.remove(faction).unwrap_or_default();
        team.units = UnitSystem::collect_factions(world, &hashset! {*faction})
            .iter()
            .sorted_by_key(|(_, unit)| unit.slot)
            .map(|(entity, _)| SerializedUnit::pack(*entity, world, resources))
            .collect_vec();
        resources.teams.insert(*faction, team);
    }
}
