use std::fmt::Display;

use bevy::{ecs::system::RunSystemOnce, prelude::Without};
use itertools::EitherOrBoth;

use super::*;

pub struct BattlePlugin;

const ANIMATION: f32 = 0.1;

pub struct Battle {
    pub left: Vec<Unit>,
    pub right: Vec<Unit>,
}
pub struct BattleSimulation {
    pub t: f32,
    pub world: World,
    pub left: Vec<Entity>,
    pub right: Vec<Entity>,
    pub log: BattleLog,
}
#[derive(Default, Debug)]
pub struct BattleLog {
    pub states: HashMap<Entity, NodeState>,
    pub actions: Vec<BattleAction>,
}
#[derive(Clone, Debug)]
pub enum BattleAction {
    VarSet(Entity, NodeKind, VarName, VarValue),
    Strike(Entity, Entity),
    Damage(Entity, Entity, i32),
    Death(Entity),
    Spawn(Entity),
}
#[derive(Component)]
struct Corpse;
impl BattleAction {
    fn apply(&self, battle: &mut BattleSimulation) -> Vec<Self> {
        let mut add_actions = Vec::default();
        let applied = match self {
            BattleAction::Strike(a, b) => {
                let pwr = battle.world.get::<UnitStats>(*a).unwrap().pwr;
                let action_a = Self::Damage(*a, *b, pwr);
                let pwr = battle.world.get::<UnitStats>(*b).unwrap().pwr;
                let action_b = Self::Damage(*b, *a, pwr);
                add_actions.extend_from_slice(&[action_a, action_b]);
                true
            }
            BattleAction::Death(a) => {
                battle.die(*a);
                true
            }
            BattleAction::Damage(_, b, x) => {
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
                true
            }
        };
        if applied {
            info!("{} {self}", "+".green().dimmed());
            battle.t += ANIMATION;
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
        }
    }
    fn process_actions(&mut self, mut actions: Vec<BattleAction>) {
        while let Some(a) = actions.pop() {
            actions.extend(a.apply(self));
        }
    }
    pub fn run(mut self) -> Self {
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
        self.process_actions(spawn_actions);
        self.process_actions(self.slots_sync());
        while !self.left.is_empty() && !self.right.is_empty() {
            let a = BattleAction::Strike(self.left[0], self.right[0]);
            self.process_actions([a].into());
            let a = self.death_check();
            self.process_actions(a);
            self.process_actions(self.slots_sync());
        }
        for i in self.world.query::<&NodeState>().iter(&self.world) {
            dbg!(i);
        }
        self
    }
    fn death_check(&mut self) -> Vec<BattleAction> {
        let mut actions: Vec<BattleAction> = default();
        for (entity, stats) in self
            .world
            .query_filtered::<(Entity, &UnitStats), Without<Corpse>>()
            .iter(&self.world)
        {
            if stats.hp <= 0 {
                actions.push(BattleAction::Death(entity));
            }
        }
        actions
    }
    fn die(&mut self, entity: Entity) {
        self.world.entity_mut(entity).insert(Corpse);
        if let Some(p) = self.left.iter().position(|u| *u == entity) {
            self.left.remove(p);
        }
        if let Some(p) = self.right.iter().position(|u| *u == entity) {
            self.right.remove(p);
        }
    }
    fn slots_sync(&self) -> Vec<BattleAction> {
        let mut actions = Vec::default();
        for (i, e) in self
            .left
            .iter()
            .enumerate()
            .chain(self.right.iter().enumerate())
        {
            actions.push(BattleAction::VarSet(
                *e,
                NodeKind::None,
                VarName::slot,
                i.into(),
            ));
        }
        actions
    }
    pub fn show_at(&mut self, t: f32, ui: &mut Ui) {
        for (e, rep, state) in self
            .world
            .query::<(Entity, &Representation, &NodeState)>()
            .iter(&self.world)
        {
            if !state
                .get_at(t, VarName::visible)
                .and_then(|v| v.get_bool().ok())
                .unwrap_or(true)
            {
                continue;
            }
            let context = Context::new_world(&self.world).set_owner(e).set_t(t).take();
            let slot = context
                .get_var(VarName::slot)
                .unwrap_or_default()
                .get_i32()
                .unwrap();
            let rect = ui.available_rect_before_wrap();
            let width = rect.width() / 5.0;
            let rect = rect
                .with_max_x(rect.min.x + width)
                .translate(egui::vec2(width * (slot as f32), 0.0));
            match RepresentationPlugin::paint_rect(rect, &context, &rep.material, ui) {
                Ok(_) => {}
                Err(e) => error!("Rep paint error: {e}"),
            }
        }
    }
}

impl ToCstr for BattleAction {
    fn cstr(&self) -> Cstr {
        match self {
            BattleAction::Strike(a, b) => format!("{a}|{b}"),
            BattleAction::Damage(a, b, x) => format!("{a}>{b}-{x}"),
            BattleAction::Death(a) => format!("x{a}"),
            BattleAction::VarSet(a, _, var, value) => format!("{a}>${var}>{value}"),
            BattleAction::Spawn(a) => format!("*{a}"),
        }
    }
}
impl Display for BattleAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.cstr().to_colored())
    }
}
