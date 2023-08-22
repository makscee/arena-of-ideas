use super::*;

#[derive(Deserialize, TypeUuid, TypePath, Debug, Clone)]
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
}
