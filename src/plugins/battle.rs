use std::fmt::Display;

use assets::animations;
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
    pub slots: usize,
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
                add_actions.extend_from_slice(&[BattleAction::VarSet(
                    *entity,
                    NodeKind::None,
                    VarName::visible,
                    true.into(),
                )]);
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
            slots: 5,
        }
    }
    fn apply_animation(&mut self, context: Context, anim: &Anim) -> Result<(), ExpressionError> {
        let context = context.clone().set_world(&self.world).take();
        let c = anim.get_changes(context)?;
        for c in c {
            c.apply(&mut self.t, &mut self.world);
        }
        Ok(())
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
            [BattleAction::VarSet(
                entity,
                NodeKind::None,
                VarName::visible,
                false.into(),
            )]
            .into()
        } else {
            default()
        }
    }
    fn slots_sync(&self) -> Vec<BattleAction> {
        let mut actions = Vec::default();
        for (i, (e, side)) in self
            .left
            .iter()
            .map(|e| (e, true))
            .enumerate()
            .chain(self.right.iter().map(|e| (e, false)).enumerate())
        {
            actions.push(BattleAction::VarSet(
                *e,
                NodeKind::None,
                VarName::slot,
                i.into(),
            ));
            actions.push(BattleAction::VarSet(
                *e,
                NodeKind::None,
                VarName::side,
                side.into(),
            ));
            let position = vec2((i + 1) as f32 * if side { -1.0 } else { 1.0 } * 2.0, 0.0);
            actions.push(BattleAction::VarSet(
                *e,
                NodeKind::None,
                VarName::position,
                position.into(),
            ));
        }
        actions
    }
    fn slot_rect(i: usize, side: bool, full_rect: Rect, team_slots: usize) -> Rect {
        let total_slots = team_slots * 2 + 1;
        let pos_i = if side {
            (team_slots - i) as i32
        } else {
            (team_slots + i) as i32
        } as f32;
        let size = (full_rect.width() / total_slots as f32).at_most(full_rect.height());
        let mut rect = full_rect;
        rect.set_height(size);
        rect.set_width(size);
        rect.translate(egui::vec2(size * pos_i, 0.0))
    }
    fn show_slot(&self, i: usize, side: bool, ui: &mut Ui) -> Response {
        let full_rect = ui.available_rect_before_wrap();
        const FRAME: Frame = Frame {
            inner_margin: Margin::same(0.0),
            outer_margin: Margin::same(0.0),
            rounding: Rounding::ZERO,
            shadow: Shadow::NONE,
            fill: TRANSPARENT,
            stroke: STROKE_DARK,
        };
        let rect = Self::slot_rect(i, side, full_rect, self.slots);
        ui.expand_to_include_rect(rect);
        let mut cui = ui.child_ui(rect, *ui.layout(), None);
        let r = cui.allocate_rect(rect, Sense::hover());
        let stroke = if r.hovered() {
            STROKE_YELLOW
        } else {
            STROKE_DARK
        };
        cui.painter().add(FRAME.stroke(stroke).paint(r.rect));
        r
    }
    pub fn show_at(&mut self, t: f32, ui: &mut Ui) {
        let center_rect = Self::slot_rect(0, true, ui.available_rect_before_wrap(), self.slots);
        let up = center_rect.width() * 0.5;
        for (slot, side) in (1..=self.slots).cartesian_product([true, false]) {
            self.show_slot(slot, side, ui);
        }
        let unit_size = center_rect.width() * UNIT_SIZE;
        let mut q = self.world.query::<(Entity, &Representation)>();
        let context = Context::new_world(&self.world).set_t(t).take();
        for (e, rep) in q.iter(&self.world) {
            let context = context.clone().set_owner(e).take();
            let position = context
                .get_var(VarName::position)
                .unwrap_or_default()
                .get_vec2()
                .unwrap()
                .to_evec2()
                * up;
            let rect = Rect::from_center_size(
                center_rect.center() + position,
                egui::Vec2::splat(unit_size),
            );
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
