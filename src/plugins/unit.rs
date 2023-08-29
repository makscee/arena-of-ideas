use bevy::utils::Instant;

use super::*;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Update, Self::update_units);
        app.add_systems(OnEnter(GameState::Next), Self::spawn)
            .add_systems(OnEnter(GameState::Restart), Self::despawn)
            .add_systems(Update, (Self::animate_unit, Self::units_strike));
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

    fn units_strike(
        input: Res<Input<KeyCode>>,
        mut queue: ResMut<ActionQueue>,
        query: Query<Entity, With<Unit>>,
    ) {
        if !input.just_pressed(KeyCode::S) {
            return;
        }
        let units = query.iter().collect_vec();
        let action = Action {
            context: Context::from_caster(units[0]).set_target(units[1]),
            effect: Effect::Damage {
                value: Some(Expression::Int(1)),
            },
        };
        queue.push(action);
    }

    pub fn translate_unit(entity: Entity, position: Vec2, world: &mut World) {
        world.get_mut::<VarState>(entity).unwrap().push_back(
            VarName::Position,
            Change::new(VarValue::Vec2(position))
                .set_duration(0.3)
                .set_tween(Tween::QuartOut),
        );
    }

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
            .query::<(Entity, &Unit)>()
            .iter(world)
            .filter_map(|(e, u)| {
                if factions.contains(&u.faction) {
                    Some((e, u.faction))
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
pub struct Unit {
    pub faction: Faction,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Faction {
    Left,
    Right,
}
