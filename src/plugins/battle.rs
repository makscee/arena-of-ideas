use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Battle), Self::enter);
    }
}

impl BattlePlugin {
    pub fn enter(world: &mut World) {
        let bs = Options::get_custom_battle(world).clone();
        bs.unpack(world);
        // bs.right.unpack(Faction::Right, world);
        // ShopPlugin::unpack_active_team(Faction::Left, world);
        UnitPlugin::translate_to_slots(world);
        Self::run_battle(world);
        world
            .get_resource_mut::<GameTimer>()
            .unwrap()
            .head_to_save();
    }

    pub fn run_battle(world: &mut World) {
        Event::BattleStart.send(world);
        ActionPlugin::spin(world);
        while let Some((left, right)) = Self::get_strikers(world) {
            Self::run_strike(left, right, world);
        }
    }

    pub fn get_strikers(world: &mut World) -> Option<(Entity, Entity)> {
        if let Some(left) = UnitPlugin::find_unit(Faction::Left, 1, world) {
            if let Some(right) = UnitPlugin::find_unit(Faction::Right, 1, world) {
                return Some((left, right));
            }
        }
        None
    }

    pub fn run_strike(left: Entity, right: Entity, world: &mut World) {
        Self::before_strike(left, right, world);
        Self::strike(left, right, world);
        UnitPlugin::run_death_check(world);
        Self::after_strike(left, right, world);
        UnitPlugin::fill_slot_gaps(Faction::Left, world);
        UnitPlugin::fill_slot_gaps(Faction::Right, world);
        UnitPlugin::translate_to_slots(world);
    }

    fn before_strike(left: Entity, right: Entity, world: &mut World) {
        Event::TurnStart.send(world);
        Event::BeforeStrike(left).send(world);
        Event::BeforeStrike(right).send(world);
        ActionPlugin::spin(world);
        let units = vec![(left, -1.0), (right, 1.0)];
        GameTimer::get_mut(world).start_batch();
        for (caster, dir) in units {
            GameTimer::get_mut(world).head_to_batch_start();
            Options::get_animations(world)
                .get(AnimationType::BeforeStrike)
                .clone()
                .apply(
                    &Context::from_owner(caster, world)
                        .set_var(VarName::Direction, VarValue::Float(dir)),
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
            let context = mem::take(
                Context::from_caster(caster, world)
                    .set_target(target, world)
                    .set_owner(caster, world),
            );
            let effect = Effect::Damage(None).wrap();
            ActionPlugin::push_back(effect, context, world);
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
                .apply(&Context::from_owner(caster, world), world)
                .unwrap();
        }
        GameTimer::get_mut(world).end_batch();
    }
}
