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
    fn update_units(time: Res<Time>, mut query: Query<&mut Transform, With<Unit>>) {
        for mut t in query.iter_mut() {
            t.translation.y = time.elapsed_seconds().sin() * 2.0;
        }
    }

    fn spawn(world: &mut World) {
        debug!("Spawn");
        for (_, unit) in world.get_resource::<Pools>().unwrap().heroes.clone() {
            let units = world.get_resource::<Assets<PackedUnit>>().unwrap();
            let unit = units.get(&unit).unwrap().clone();
            unit.unpack(world);
        }
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
        debug!("Despawn");
        state.set(GameState::Next);
        *time = Time::new(Instant::now());
    }
}

#[derive(Deserialize, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "028620be-3b01-4e20-b62e-a631f0db4777"]
pub struct PackedUnit {
    pub representation: Representation,
    pub state: VarState,
}

impl PackedUnit {
    pub fn unpack(self, world: &mut World) {
        let entity = world
            .get_resource::<Assets<Representation>>()
            .unwrap()
            .get(&world.get_resource::<Options>().unwrap().unit)
            .unwrap()
            .clone()
            .unpack(None, world);
        self.representation.unpack(Some(entity), world);
        world.entity_mut(entity).insert(Unit).insert(self.state);
    }
}

#[derive(Resource, Debug)]
pub struct UnitHandle(pub Handle<PackedUnit>);

#[derive(Component)]
pub struct Unit;
