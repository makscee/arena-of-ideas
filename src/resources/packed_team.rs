use super::*;

#[derive(Asset, Deserialize, Serialize, TypePath, Debug, Clone, Default)]
pub struct PackedTeam {
    pub units: Vec<PackedUnit>,
    #[serde(default)]
    pub ability_states: AbilityStates,
}

#[derive(Component, Default, Clone, Debug, Serialize, Deserialize)]
pub struct AbilityStates(pub HashMap<String, VarState>);

impl PackedTeam {
    pub fn from_id(id: u64) -> Self {
        if id == 0 {
            return default();
        }
        let team = id.get_team();
        Self {
            units: team.units.into_iter().map(|u| u.into()).collect_vec(),
            ..default()
        }
    }
    pub fn unpack(self, faction: Faction, world: &mut World) {
        debug!("unpack team: {self:?}");
        let entity = TeamPlugin::entity(faction, world);
        for (slot, unit) in self.units.into_iter().enumerate() {
            unit.unpack(entity, Some(slot as i32), None, world);
        }
        world.entity_mut(entity).insert(self.ability_states);
    }
    pub fn apply_limit(mut self) -> Self {
        self.units
            .resize(global_settings().arena.team_slots as usize, default());
        self
    }
}
