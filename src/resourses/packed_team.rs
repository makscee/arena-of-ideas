use super::*;

#[derive(Deserialize, TypeUuid, TypePath, Debug, Clone, Default)]
#[uuid = "cb5457bc-b429-4af8-8d92-bf141a80020b"]
pub struct PackedTeam {
    pub units: Vec<PackedUnit>,
}

impl PackedTeam {
    pub fn unpack(self, faction: Faction, world: &mut World) {
        for (i, unit) in self.units.into_iter().enumerate() {
            unit.unpack(faction, Some(i + 1), world);
        }
    }
    pub fn pack(faction: Faction, world: &mut World) -> Self {
        let mut team = PackedTeam::default();
        for (entity, _) in UnitPlugin::collect_factions(&HashSet::from([faction]), world) {
            team.units.push(PackedUnit::pack(entity, world));
        }
        team
    }
}
