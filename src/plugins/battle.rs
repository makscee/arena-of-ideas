use bevy::ecs::system::RunSystemOnce;
use itertools::EitherOrBoth;

use super::*;

pub struct BattlePlugin;

pub struct Battle {
    pub left: Vec<Unit>,
    pub right: Vec<Unit>,
}
pub struct BattleSimulation {
    pub world: World,
    pub left: Vec<Entity>,
    pub right: Vec<Entity>,
}
#[derive(Default, Debug)]
pub struct BattleLog {
    pub states: HashMap<Entity, NodeState>,
    pub actions: Vec<BattleAction>,
}
#[derive(Default, Debug)]
pub struct BattleHistory {
    pub t: f32,
    pub reps: HashMap<Entity, Representation>,
    pub history: HashMap<Entity, StateHistory>,
}
#[derive(Default, Debug)]
pub struct StateHistory {
    pub vars: HashMap<VarName, VarHistory>,
}
#[derive(Default, Debug)]
pub struct VarHistory {
    changes: Vec<(f32, VarValue)>,
}
#[derive(Clone, Debug)]
pub enum BattleAction {
    VarSet(Entity, NodeKind, VarName, VarValue),
    Strike(Entity, Entity),
    Damage(Entity, Entity, i32),
    Death(Entity),
    Spawn(Entity),
}
impl BattleAction {
    fn apply(
        &self,
        battle: &mut BattleSimulation,
        log: &mut BattleLog,
        history: &mut BattleHistory,
    ) -> Vec<Self> {
        match self {
            BattleAction::Strike(a, b) => {
                let pwr = battle.world.get::<UnitStats>(*a).unwrap().pwr;
                let action_a = Self::Damage(*a, *b, pwr);
                let pwr = battle.world.get::<UnitStats>(*b).unwrap().pwr;
                let action_b = Self::Damage(*b, *a, pwr);
                [action_a, action_b].into()
            }
            BattleAction::Death(a) => {
                battle.die(*a);
                default()
            }
            BattleAction::Damage(_, b, x) => {
                let hp = battle.world.get::<UnitStats>(*b).unwrap().hp - x;
                [Self::VarSet(
                    *b,
                    NodeKind::UnitStats,
                    VarName::hp,
                    hp.into(),
                )]
                .into()
            }
            BattleAction::VarSet(entity, kind, var, value) => {
                kind.set_var(*entity, *var, value.clone(), &mut battle.world);
                default()
            }
            BattleAction::Spawn(entity) => {
                let reps = <Unit as Node>::collect_children_entity::<Representation>(
                    *entity,
                    &Context::new_world(&battle.world),
                )
                .into_iter()
                .map(|(e, r)| (e, r.clone()))
                .collect_vec();
                for (e, r) in reps {
                    let state = battle
                        .world
                        .run_system_once_with(e, NodeStatePlugin::collect_full_state);
                    history.reps.insert(e, r.clone());
                    history.history.insert(e, StateHistory::from_state(&state));
                }
                let state = battle
                    .world
                    .run_system_once_with(*entity, NodeStatePlugin::collect_full_state);
                log.states.insert(*entity, state);
                default()
            }
        }
    }
    fn apply_hist(&self, history: &mut BattleHistory) {
        const ANIMATION: f32 = 0.1;
        match self {
            BattleAction::Strike(a, b) => {}
            BattleAction::VarSet(entity, _, var, value) => {
                history.set_var(*entity, *var, value.clone());
                history.t += ANIMATION;
            }
            BattleAction::Death(entity) => {
                history.set_var(*entity, VarName::visible, false.into());
            }
            BattleAction::Damage(a, b, x) => {}
            BattleAction::Spawn(entity) => history.set_var(*entity, VarName::visible, true.into()),
        }
    }
}

impl VarHistory {
    fn new(value: VarValue) -> Self {
        Self {
            changes: vec![(0.0, value)],
        }
    }
}
impl StateHistory {
    fn from_state(state: &NodeState) -> Self {
        Self {
            vars: HashMap::from_iter(
                state
                    .vars
                    .iter()
                    .map(|(k, v)| (*k, VarHistory::new(v.clone()))),
            ),
        }
    }
    fn set(&mut self, t: f32, var: VarName, value: VarValue) {
        let h = &mut self.vars.entry(var).or_default().changes;
        h.push((t, value));
    }
}
impl BattleHistory {
    fn set_var(&mut self, entity: Entity, var: VarName, value: VarValue) {
        self.history
            .get_mut(&entity)
            .unwrap()
            .set(self.t, var, value);
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
        for (_, u) in battle.left.iter().enumerate() {
            let entity = world.spawn_empty().id();
            u.clone().unpack(entity, &mut world.commands());
            left.push(entity);
        }
        for (_, u) in battle.right.iter().enumerate() {
            let entity = world.spawn_empty().id();
            u.clone().unpack(entity, &mut world.commands());
            right.push(entity);
        }
        Self { world, left, right }
    }
    fn process_actions(
        &mut self,
        log: &mut BattleLog,
        history: &mut BattleHistory,
        mut actions: Vec<BattleAction>,
    ) {
        while let Some(a) = actions.pop() {
            log.actions.push(a.clone());
            actions.extend(a.apply(self, log, history));
        }
    }
    pub fn run(mut self) -> (BattleLog, BattleHistory) {
        let log = &mut BattleLog::default();
        let history = &mut BattleHistory::default();
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
            .collect_vec();
        self.process_actions(log, history, spawn_actions);
        while !self.left.is_empty() && !self.right.is_empty() {
            let a = BattleAction::Strike(self.left[0], self.right[0]);
            self.process_actions(log, history, [a].into());
            let a = self.death_check();
            self.process_actions(log, history, a);
        }
        (mem::take(log), mem::take(history))
    }
    fn death_check(&mut self) -> Vec<BattleAction> {
        let mut actions: Vec<BattleAction> = default();
        for (entity, stats) in self.world.query::<(Entity, &UnitStats)>().iter(&self.world) {
            if stats.hp <= 0 {
                actions.push(BattleAction::Death(entity));
            }
        }
        actions
    }
    fn die(&mut self, entity: Entity) {
        self.world.entity_mut(entity).despawn_recursive();
        if let Some(p) = self.left.iter().position(|u| *u == entity) {
            self.left.remove(p);
        }
        if let Some(p) = self.right.iter().position(|u| *u == entity) {
            self.right.remove(p);
        }
    }
}

impl ToCstr for BattleAction {
    fn cstr(&self) -> Cstr {
        match self {
            BattleAction::Strike(a, b) => format!("{a}|{b}"),
            BattleAction::Damage(a, b, x) => format!("{a}>{b}-{x}"),
            BattleAction::Death(a) => format!("{a}x"),
            BattleAction::VarSet(a, b, var, value) => format!("{a}>{b}${var}>{value}"),
            BattleAction::Spawn(a) => format!("*{a}"),
        }
    }
}
