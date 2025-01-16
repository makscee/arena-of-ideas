use std::collections::VecDeque;

use assets::animations;
use bevy::{ecs::system::RunSystemOnce, prelude::Without};
use colored::Colorize;

use itertools::EitherOrBoth;

use super::*;

const ANIMATION: f32 = 0.2;

pub struct Battle {
    pub left: Vec<Unit>,
    pub right: Vec<Unit>,
}
#[derive(Debug)]
pub struct BattleSimulation {
    pub t: f32,
    pub world: World,
    pub left: Vec<Entity>,
    pub right: Vec<Entity>,
    pub log: BattleLog,
    pub slots: usize,
}
#[derive(Default, Debug)]
pub struct BattleLog {
    pub states: HashMap<Entity, NodeState>,
    pub actions: Vec<BattleAction>,
}

#[derive(Component)]
pub struct Corpse;

impl BattleAction {
    fn apply(&self, battle: &mut BattleSimulation) -> Vec<Self> {
        let mut add_actions = Vec::default();
        let applied = match self {
            BattleAction::Strike(a, b) => {
                let strike_anim = animations().get("strike").unwrap();
                match battle.apply_animation(
                    Context::default()
                        .set_owner(*a)
                        .set_target(*b)
                        .set_var(VarName::position, vec2(0.0, 0.0).into())
                        .take(),
                    strike_anim,
                ) {
                    Ok(_) => {}
                    Err(e) => error!("Animation error: {e}"),
                }
                let strike_vfx = animations().get("strike_vfx").unwrap();
                match battle.apply_animation(Context::default(), strike_vfx) {
                    Ok(_) => {}
                    Err(e) => error!("Animation error: {e}"),
                }
                let pwr = battle.world.get::<UnitStats>(*a).unwrap().pwr;
                let action_a = Self::Damage(*a, *b, pwr);
                let pwr = battle.world.get::<UnitStats>(*b).unwrap().pwr;
                let action_b = Self::Damage(*b, *a, pwr);
                add_actions.extend_from_slice(&[action_a, action_b]);
                add_actions.extend(battle.slots_sync());
                true
            }
            BattleAction::Death(a) => {
                add_actions.extend(battle.die(*a));
                true
            }
            BattleAction::Damage(_, b, x) => {
                let text = animations().get("text").unwrap();
                let pos = Context::new_battle_simulation(&battle)
                    .set_owner(*b)
                    .get_var(VarName::position)
                    .unwrap();
                match battle.apply_animation(
                    Context::default()
                        .set_var(VarName::text, (-*x).to_string().into())
                        .set_var(VarName::color, RED.into())
                        .set_var(VarName::position, pos)
                        .take(),
                    text,
                ) {
                    Ok(_) => {}
                    Err(e) => error!("Animation error: {e}"),
                }
                let hp = battle.world.get::<UnitStats>(*b).unwrap().hp - x;
                add_actions.push(Self::VarSet(
                    *b,
                    NodeKind::UnitStats,
                    VarName::hp,
                    hp.into(),
                ));
                true
            }
            BattleAction::VarSet(entity, kind, var, value) => {
                if battle.world.get_mut::<NodeState>(*entity).unwrap().insert(
                    battle.t,
                    0.1,
                    *var,
                    value.clone(),
                    *kind,
                ) {
                    kind.set_var(*entity, *var, value.clone(), &mut battle.world);
                    true
                } else {
                    false
                }
            }
            BattleAction::Spawn(entity) => {
                battle
                    .world
                    .run_system_once_with((*entity, battle.t), NodeStatePlugin::inject_entity_vars);
                battle.log.add_state(*entity, &mut battle.world);
                add_actions.extend_from_slice(&[BattleAction::VarSet(
                    *entity,
                    NodeKind::None,
                    VarName::visible,
                    true.into(),
                )]);
                true
            }
            BattleAction::Wait(t) => {
                battle.t += *t;
                false
            }
        };
        if applied {
            info!("{} {self}", "+".green().dimmed());
            battle.log.actions.push(self.clone());
        } else {
            info!("{} {self}", "-".dimmed());
        }
        add_actions
    }
}

impl BattleLog {
    fn add_state(&mut self, entity: Entity, world: &mut World) {
        self.states.insert(
            entity,
            world.run_system_once_with(entity, NodeStatePlugin::collect_full_state),
        );
    }
}
impl BattleSimulation {
    pub fn new(battle: &Battle) -> Self {
        let mut world = World::new();
        for k in NodeKind::iter() {
            k.register_world(&mut world);
        }
        let mut left: Vec<Entity> = default();
        let mut right: Vec<Entity> = default();
        let mut log = BattleLog::default();
        for (_, u) in battle.left.iter().enumerate() {
            let entity = world.spawn_empty().id();
            u.clone().unpack(entity, &mut world.commands());
            left.push(entity);
            log.add_state(entity, &mut world);
        }
        for (_, u) in battle.right.iter().enumerate() {
            let entity = world.spawn_empty().id();
            u.clone().unpack(entity, &mut world.commands());
            right.push(entity);
            log.add_state(entity, &mut world);
        }
        world.flush();
        Self {
            t: 0.0,
            world,
            left,
            right,
            log,
            slots: 5,
        }
    }
    fn apply_animation(&mut self, context: Context, anim: &Anim) -> Result<(), ExpressionError> {
        anim.apply(&mut self.t, context, &mut self.world)?;
        Ok(())
    }
    fn process_actions(&mut self, mut actions: VecDeque<BattleAction>) {
        while let Some(a) = actions.pop_front() {
            for a in a.apply(self) {
                actions.push_front(a);
            }
        }
    }
    fn send_event(&mut self, event: Event) {
        let mut actions = Vec::default();
        for (entity, ut) in self
            .world
            .query_filtered::<(Entity, &UnitTrigger), Without<Corpse>>()
            .iter(&self.world)
        {
            match event {
                Event::BattleStart => match ut.trigger {
                    Trigger::BattleStart => {
                        let mut context = Context::new_battle_simulation(self)
                            .set_owner(entity)
                            .take();
                        match ut.target.get_entity(&context) {
                            Ok(target) => {
                                context.set_target(target);
                            }
                            Err(e) => {
                                error!("Get target error: {e}")
                            }
                        }
                        match ut.effect.process(&context) {
                            Ok(a) => {
                                actions.extend(a);
                            }
                            Err(e) => {
                                error!("Effect process error: {e}")
                            }
                        }
                    }
                    Trigger::TurnEnd => todo!(),
                },
            }
        }
        self.process_actions(actions.into());
    }
    pub fn start(mut self) -> Self {
        let spawn_actions = self
            .left
            .iter()
            .zip_longest(self.right.iter())
            .flat_map(|e| match e {
                EitherOrBoth::Both(a, b) => {
                    vec![BattleAction::Spawn(*a), BattleAction::Spawn(*b)]
                }
                EitherOrBoth::Left(e) | EitherOrBoth::Right(e) => {
                    vec![BattleAction::Spawn(*e)]
                }
            })
            .collect();
        self.process_actions(spawn_actions);
        self.process_actions(self.slots_sync());
        self.send_event(Event::BattleStart);
        self
    }
    pub fn run(&mut self) {
        if self.ended() {
            return;
        }
        let a = BattleAction::Strike(self.left[0], self.right[0]);
        self.process_actions([a].into());
        let a = self.death_check();
        self.process_actions(a);
        self.process_actions(self.slots_sync());
    }
    pub fn ended(&self) -> bool {
        self.left.is_empty() || self.right.is_empty()
    }
    fn death_check(&mut self) -> VecDeque<BattleAction> {
        let mut actions: VecDeque<BattleAction> = default();
        for (entity, stats) in self
            .world
            .query_filtered::<(Entity, &UnitStats), Without<Corpse>>()
            .iter(&self.world)
        {
            if stats.hp <= 0 {
                actions.push_back(BattleAction::Death(entity));
            }
        }
        actions
    }
    fn die(&mut self, entity: Entity) -> Vec<BattleAction> {
        self.world.entity_mut(entity).insert(Corpse);
        let mut died = false;
        if let Some(p) = self.left.iter().position(|u| *u == entity) {
            self.left.remove(p);
            died = true;
        }
        if let Some(p) = self.right.iter().position(|u| *u == entity) {
            self.right.remove(p);
            died = true;
        }
        if died {
            [
                BattleAction::VarSet(entity, NodeKind::None, VarName::visible, false.into()),
                BattleAction::Wait(ANIMATION),
            ]
            .into()
        } else {
            default()
        }
    }
    fn slots_sync(&self) -> VecDeque<BattleAction> {
        let mut actions = VecDeque::default();
        for (i, (e, side)) in self
            .left
            .iter()
            .map(|e| (e, true))
            .enumerate()
            .chain(self.right.iter().map(|e| (e, false)).enumerate())
        {
            actions.push_back(BattleAction::VarSet(
                *e,
                NodeKind::None,
                VarName::slot,
                i.into(),
            ));
            actions.push_back(BattleAction::VarSet(
                *e,
                NodeKind::None,
                VarName::side,
                side.into(),
            ));
            let position = vec2((i + 1) as f32 * if side { -1.0 } else { 1.0 } * 2.0, 0.0);
            actions.push_back(BattleAction::VarSet(
                *e,
                NodeKind::None,
                VarName::position,
                position.into(),
            ));
        }
        actions.push_back(BattleAction::Wait(ANIMATION * 3.0));
        actions
    }
}
