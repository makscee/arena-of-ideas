use bevy::utils::Instant;

use super::*;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Update, Self::update_units);
        app.add_systems(OnEnter(GameState::Next), Self::spawn)
            .add_systems(OnEnter(GameState::Restart), Self::despawn);
    }
}

impl UnitPlugin {
    pub fn get_slot_position(faction: Faction, slot: usize) -> Vec2 {
        vec2(
            slot as f32
                * 3.0
                * match faction {
                    Faction::Left => -1.0,
                    Faction::Right => 1.0,
                },
            0.0,
        )
    }

    pub fn collect_factions(
        factions: &HashSet<Faction>,
        world: &mut World,
    ) -> Vec<(Entity, Faction)> {
        world
            .query_filtered::<(Entity, &VarState), With<Unit>>()
            .iter(world)
            .filter_map(|(e, state)| {
                let faction = state.get_faction(VarName::Faction).unwrap();
                if factions.contains(&faction) {
                    Some((e, faction))
                } else {
                    None
                }
            })
            .collect_vec()
    }

    fn spawn(world: &mut World) {
        for (i, (_, unit)) in world
            .get_resource::<Pools>()
            .unwrap()
            .heroes
            .clone()
            .into_iter()
            .enumerate()
        {
            let units = world.get_resource::<Assets<PackedUnit>>().unwrap();
            let unit = units.get(&unit).unwrap().clone();
            unit.unpack(Faction::Left, Some(i + 1), world);
        }
        dbg!(Options::get_custom_battle(world));
    }

    fn despawn(
        query: Query<Entity, With<Unit>>,
        mut commands: Commands,
        mut state: ResMut<NextState<GameState>>,
        mut time: ResMut<Time>,
    ) {
        for unit in query.iter() {
            commands.entity(unit).despawn_recursive();
        }
        state.set(GameState::Battle);
        *time = Time::new(Instant::now());
    }
}

#[derive(Resource, Debug)]
pub struct UnitHandle(pub Handle<PackedUnit>);

#[derive(Component)]
pub struct Unit;

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, PartialOrd, Ord,
)]
pub enum Faction {
    Left,
    Right,
}
