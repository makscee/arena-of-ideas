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
    fn update_units(time: Res<Time>, mut query: Query<&mut Transform, With<Unit>>) {
        for mut t in query.iter_mut() {
            t.translation.y = time.elapsed_seconds().sin() * 2.0;
        }
    }

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
    pub hp: i32,
    pub atk: i32,
    pub name: String,
    pub representation: Representation,
    pub state: VarState,
}

impl PackedUnit {
    pub fn unpack(mut self, world: &mut World) {
        let entity = world
            .get_resource::<Assets<Representation>>()
            .unwrap()
            .get(&world.get_resource::<Options>().unwrap().unit)
            .unwrap()
            .clone()
            .unpack(None, world);
        self.representation.unpack(Some(entity), world);
        self.state
            .insert(VarName::Hp, VarValue::Int(self.hp))
            .insert(VarName::Atk, VarValue::Int(self.atk))
            .insert(VarName::Name, VarValue::String(self.name))
            .insert(VarName::Position, VarValue::Vec2(default()));
        world.entity_mut(entity).insert(Unit).insert(self.state);
    }
}

#[derive(Resource, Debug)]
pub struct UnitHandle(pub Handle<PackedUnit>);

#[derive(Component)]
pub struct Unit;
