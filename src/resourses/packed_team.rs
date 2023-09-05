use bevy::ecs::world::EntityMut;

use super::*;

#[derive(Deserialize, TypeUuid, TypePath, Debug, Clone, Default)]
#[uuid = "cb5457bc-b429-4af8-8d92-bf141a80020b"]
pub struct PackedTeam {
    pub units: Vec<PackedUnit>,
    #[serde(default)]
    pub state: VarState,
}

impl PackedTeam {
    pub fn unpack(mut self, faction: Faction, world: &mut World) {
        Self::despawn(faction, world);
        self.state
            .insert(VarName::Faction, VarValue::Faction(faction));
        let team = Self::spawn(faction, world).insert(self.state).id();
        for (i, unit) in self.units.into_iter().enumerate() {
            unit.unpack(team, Some(i + 1), world);
        }
    }
    pub fn pack(faction: Faction, world: &mut World) -> Self {
        let mut team = PackedTeam::default();
        for (entity, _) in UnitPlugin::collect_factions(HashSet::from([faction]), world) {
            team.units.push(PackedUnit::pack(entity, world));
        }
        team
    }
    pub fn spawn(faction: Faction, world: &mut World) -> EntityMut {
        Self::despawn(faction, world);

        world.spawn((
            VarState::new_with(VarName::Faction, VarValue::Faction(faction)),
            Team,
            Transform::default(),
            GlobalTransform::default(),
            VisibilityBundle::default(),
        ))
    }
    pub fn despawn(faction: Faction, world: &mut World) {
        if let Some(team) = Self::entity(faction, world) {
            world.entity_mut(team).despawn_recursive();
        }
    }
    pub fn entity(faction: Faction, world: &mut World) -> Option<Entity> {
        world
            .query_filtered::<(Entity, &VarState), With<Team>>()
            .iter(world)
            .find_map(
                |(e, s)| match s.get_faction(VarName::Faction).unwrap().eq(&faction) {
                    true => Some(e),
                    false => None,
                },
            )
    }
    pub fn state(faction: Faction, world: &mut World) -> Option<&VarState> {
        Self::entity(faction, world).and_then(|e| Some(VarState::get(e, world)))
    }
    pub fn state_mut(faction: Faction, world: &mut World) -> Option<Mut<VarState>> {
        Self::entity(faction, world).and_then(|e| Some(VarState::get_mut(e, world)))
    }
}

#[derive(Component)]
pub struct Team;
