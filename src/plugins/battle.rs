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
        GameTimer::get_mut(world).start_batch();
        for (caster, dir) in units {
            GameTimer::get_mut(world).head_to_batch_start();
            Options::get_animations(world)
                .get(AnimationType::BeforeStrike)
                .clone()
                .apply(
                    &Context::from_owner(caster).set_var(VarName::Direction, VarValue::Float(dir)),
                    world,
                )
                .unwrap();
        }
        GameTimer::get_mut(world).end_batch();
    }

    fn strike(left: Entity, right: Entity, world: &mut World) {
        debug!("Strike {left:?} {right:?}");
        let units = vec![(left, right), (right, left)];
        for (caster, target) in units {
            let context = Context::from_caster(caster)
                .set_target(target)
                .set_owner(caster);
            let effect = Effect::Damage {
                value: Some(Expression::Int(1)),
            };
            ActionPlugin::queue_effect(effect, context, world);
            ActionPlugin::spin(world);
        }
    }

    fn after_strike(left: Entity, right: Entity, world: &mut World) {
        debug!("After strike {left:?} {right:?}");
        let units = vec![left, right];
        GameTimer::get_mut(world).start_batch();
        for caster in units {
            GameTimer::get_mut(world).head_to_batch_start();
            Options::get_animations(world)
                .get(AnimationType::AfterStrike)
                .clone()
                .apply(&Context::from_owner(caster), world)
                .unwrap();
        }
        GameTimer::get_mut(world).end_batch();
    }

    fn translate_to_slots(world: &mut World) {
        let units =
            UnitPlugin::collect_factions(&HashSet::from([Faction::Left, Faction::Right]), world);
        GameTimer::get_mut(world).start_batch();
        for (unit, faction) in units.into_iter() {
            let slot = VarState::get(unit, world).get_int(VarName::Slot).unwrap() as usize;
            GameTimer::get_mut(world).head_to_batch_start();
            UnitPlugin::translate_unit(unit, UnitPlugin::get_slot_position(faction, slot), world);
        }
        GameTimer::get_mut(world).end_batch();
    }
}
