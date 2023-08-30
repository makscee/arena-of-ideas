use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Battle), Self::setup);
    }
}

impl BattlePlugin {
    pub fn setup(world: &mut World) {
        let bs = Options::get_custom_battle(world).clone();
        bs.unpack(world);
        Self::translate_to_slots(world);
        Self::run_battle(world);
    }

    pub fn run_battle(world: &mut World) {
        while let Some((left, right)) = Self::get_strikers(world) {
            Self::run_strike(left, right, world);
        }
    }

    pub fn get_strikers(world: &mut World) -> Option<(Entity, Entity)> {
        let units = world
            .query_filtered::<(Entity, &VarState), With<Unit>>()
            .iter(world)
            .filter(|(_, s)| s.get_int(VarName::Slot).unwrap() == 1)
            .collect_vec();
        if units.len() == 2 {
            let (left, right) = units
                .iter()
                .sorted_by(|(_, a), (_, b)| {
                    a.get_faction(VarName::Faction)
                        .unwrap()
                        .cmp(&b.get_faction(VarName::Faction).unwrap())
                })
                .map(|(e, _)| *e)
                .collect_tuple()
                .unwrap();
            Some((left, right))
        } else {
            None
        }
    }

    pub fn run_strike(left: Entity, right: Entity, world: &mut World) {
        Self::before_strike(left, right, world);
        Self::strike(left, right, world);
        UnitPlugin::run_death_check(world);
        Self::after_strike(left, right, world);
        UnitPlugin::fill_slot_gaps(Faction::Left, world);
        UnitPlugin::fill_slot_gaps(Faction::Right, world);
        Self::translate_to_slots(world);
    }

    fn before_strike(left: Entity, right: Entity, world: &mut World) {
        debug!("Before strike {left:?} {right:?}");
        let units = vec![(left, -1.0), (right, 1.0)];
        world.get_resource_mut::<GameTimer>().unwrap().save_end();
        for (caster, dir) in units {
            let state = world.get_mut::<VarState>(caster).unwrap();
            let pos =
                UnitPlugin::get_slot_position(state.get_faction(VarName::Faction).unwrap(), 1);
            let c1 = Change::new(VarValue::Vec2(pos + vec2(2.0, 0.0) * dir))
                .set_duration(1.0)
                .set_tween(Tween::QuartInOut);
            let c2 = Change::new(VarValue::Vec2(vec2(1.0, 0.0) * dir))
                .set_duration(0.1)
                .set_t(0.3);
            VarState::push_back(caster, VarName::Position, c1, world);
            VarState::push_back(caster, VarName::Position, c2, world);
            world
                .get_resource_mut::<GameTimer>()
                .unwrap()
                .return_to_saved_end();
        }
        world
            .get_resource_mut::<GameTimer>()
            .unwrap()
            .return_to_max_end();
    }

    fn strike(left: Entity, right: Entity, world: &mut World) {
        debug!("Strike {left:?} {right:?}");
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
        debug!("After strike {left:?} {right:?}");
        let units = vec![left, right];
        world.get_resource_mut::<GameTimer>().unwrap().save_end();
        for caster in units {
            let state = world.get_mut::<VarState>(caster).unwrap();
            let faction = state.get_faction(VarName::Faction).unwrap();
            let slot = state.get_int(VarName::Slot).unwrap();
            let pos = UnitPlugin::get_slot_position(faction, slot as usize);
            let change = Change::new(VarValue::Vec2(pos))
                .set_duration(0.5)
                .set_tween(Tween::QuartOut)
                .set_t(0.5);
            VarState::push_back(caster, VarName::Position, change, world);
            world
                .get_resource_mut::<GameTimer>()
                .unwrap()
                .return_to_saved_end();
        }
        world
            .get_resource_mut::<GameTimer>()
            .unwrap()
            .return_to_max_end();
    }

    fn translate_to_slots(world: &mut World) {
        let units =
            UnitPlugin::collect_factions(&HashSet::from([Faction::Left, Faction::Right]), world);
        world.get_resource_mut::<GameTimer>().unwrap().save_end();
        for (unit, faction) in units.into_iter() {
            let slot = VarState::get_value_from_world(unit, VarName::Slot, world)
                .unwrap()
                .get_int()
                .unwrap() as usize;
            UnitPlugin::translate_unit(unit, UnitPlugin::get_slot_position(faction, slot), world);
            world
                .get_resource_mut::<GameTimer>()
                .unwrap()
                .return_to_saved_end();
        }
        world
            .get_resource_mut::<GameTimer>()
            .unwrap()
            .return_to_max_end();
    }
}
