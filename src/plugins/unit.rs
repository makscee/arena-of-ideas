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
    fn animate_unit(world: &mut World) {
        let input = world.get_resource::<Input<KeyCode>>().unwrap();
        if !input.just_pressed(KeyCode::A) {
            return;
        }
        let units = world
            .query_filtered::<Entity, With<Unit>>()
            .iter(world)
            .collect_vec();
        let (left, right) = (units[0], units[1]);
        Self::before_strike(left, right, world);
        Self::strike(left, right, world);
        Self::after_strike(left, right, world);
    }

    fn before_strike(left: Entity, right: Entity, world: &mut World) {
        let elapsed = world.get_resource::<Time>().unwrap().elapsed_seconds();
        let units = vec![(left, -1.0), (right, 1.0)];
        for (caster, dir) in units {
            let mut state = world.get_mut::<VarState>(caster).unwrap();
            let pos =
                UnitPlugin::get_slot_position(state.get_faction(VarName::Faction).unwrap(), 1);

            let t = elapsed - state.duration();
            let c1 = Change::new(VarValue::Vec2(pos + vec2(2.0, 0.0) * dir))
                .set_duration(1.0)
                .set_t(t)
                .set_tween(Tween::QuartInOut);
            let c2 = Change::new(VarValue::Vec2(vec2(1.0, 0.0) * dir))
                .set_duration(0.1)
                .set_t(0.3);
            state.push_back(VarName::Position, c1);
            state.push_back(VarName::Position, c2);
        }
    }

    fn strike(left: Entity, right: Entity, world: &mut World) {
        let units = vec![(left, right), (right, left)];
        for (caster, target) in units {
            let action = Action {
                context: Context::from_caster(caster).set_target(target),
                effect: Effect::Damage {
                    value: Some(Expression::Int(1)),
                },
            };
            world
                .get_resource_mut::<ActionQueue>()
                .unwrap()
                .push(action);
            ActionPlugin::spin(world);
        }
    }

    fn after_strike(left: Entity, right: Entity, world: &mut World) {
        let units = vec![left, right];
        for caster in units {
            let mut state = world.get_mut::<VarState>(caster).unwrap();
            let faction = state.get_faction(VarName::Faction).unwrap();
            let slot = state.get_int(VarName::Slot).unwrap();
            let pos = UnitPlugin::get_slot_position(faction, slot as usize);
            let change = Change::new(VarValue::Vec2(pos))
                .set_duration(0.5)
                .set_tween(Tween::QuartOut)
                .set_t(0.5);
            state.push_back(VarName::Position, change);
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum Faction {
    Left,
    Right,
}
