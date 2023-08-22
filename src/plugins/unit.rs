use bevy::utils::Instant;

use super::*;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Update, Self::update_units);
        app.add_systems(OnEnter(GameState::Next), Self::spawn)
            .add_systems(OnEnter(GameState::Restart), Self::despawn)
            .add_systems(Update, Self::animate_unit);
    }
}

impl UnitPlugin {
    fn animate_unit(
        input: Res<Input<KeyCode>>,
        mut query: Query<&mut VarState, With<Unit>>,
        time: Res<Time>,
    ) {
        if !input.just_pressed(KeyCode::A) {
            return;
        }
        for mut state in query.iter_mut() {
            let t = time.elapsed_seconds() - state.duration();
            let c1 = Change::new(VarValue::Vec2(vec2(-1.0, 0.0)))
                .set_duration(1.0)
                .set_t(t)
                .set_tween(Tween::QuartInOut);
            let c2 = Change::new(VarValue::Vec2(vec2(2.0, 0.0)))
                .set_duration(0.1)
                .set_t(0.3);
            let c3 = Change::new(VarValue::Vec2(vec2(0.0, 0.0)))
                .set_duration(0.5)
                .set_tween(Tween::QuartOut)
                .set_t(0.5);
            state.push_back(VarName::Position, c1);
            state.push_back(VarName::Position, c2);
            state.push_back(VarName::Position, c3);
        }
    }

    fn spawn(world: &mut World) {
        for (_, unit) in world.get_resource::<Pools>().unwrap().heroes.clone() {
            let units = world.get_resource::<Assets<PackedUnit>>().unwrap();
            let unit = units.get(&unit).unwrap().clone();
            unit.unpack(world);
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
        state.set(GameState::Next);
        *time = Time::new(Instant::now());
    }
}

#[derive(Resource, Debug)]
pub struct UnitHandle(pub Handle<PackedUnit>);

#[derive(Component)]
pub struct Unit;
