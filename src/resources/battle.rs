use super::*;

const ANIMATION: f32 = 0.2;

pub struct Battle {
    pub left: Team,
    pub right: Team,
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
#[derive(Clone, Debug)]
pub enum BattleAction {
    VarSet(Entity, NodeKind, VarName, VarValue),
    Strike(Entity, Entity),
    Damage(Entity, Entity, i32),
    Death(Entity),
    Spawn(Entity),
    ApplyStatus(Entity),
    Wait(f32),
}

impl ToCstr for BattleAction {
    fn cstr(&self) -> Cstr {
        match self {
            BattleAction::Strike(a, b) => format!("{a}|{b}"),
            BattleAction::Damage(a, b, x) => format!("{a}>{b}-{x}"),
            BattleAction::Death(a) => format!("x{a}"),
            BattleAction::VarSet(a, _, var, value) => format!("{a}>${var}>{value}"),
            BattleAction::Spawn(a) => format!("*{a}"),
            BattleAction::ApplyStatus(a) => format!("+{a}"),
            BattleAction::Wait(t) => format!("~{t}"),
        }
    }
}
impl std::fmt::Display for BattleAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.cstr().to_colored())
    }
}

impl Battle {
    pub fn open_window(&self, world: &mut World) {
        let mut bs = BattleSimulation::new(self).start();
        let mut t = 0.0;
        let mut playing = false;
        Window::new("Battle", move |ui, _| {
            ui.set_min_size(egui::vec2(800.0, 400.0));
            Slider::new("ts").full_width().ui(&mut t, 0.0..=bs.t, ui);
            Checkbox::new(&mut playing, "play").ui(ui);
            if "+1".cstr().button(ui).clicked() {
                bs.run();
            }
            if "+10".cstr().button(ui).clicked() {
                for _ in 0..10 {
                    bs.run();
                }
            }
            if "+100".cstr().button(ui).clicked() {
                for _ in 0..100 {
                    bs.run();
                }
            }
            if playing {
                t += gt().last_delta();
                t = t.at_most(bs.t);
            }
            bs.show_at(t, ui);
            if t >= bs.t && !bs.ended() {
                bs.run();
            }
        })
        .push(world);
    }
}

impl BattleAction {
    fn apply(&self, battle: &mut BattleSimulation) -> Vec<Self> {
        let mut add_actions = Vec::default();
        let applied = match self {
            BattleAction::Strike(a, b) => {
                let strike_anim = animations().get("strike").unwrap();
                battle.apply_animation(
                    Context::default()
                        .set_owner(*a)
                        .set_target(*b)
                        .set_var(VarName::position, vec2(0.0, 0.0).into())
                        .take(),
                    strike_anim,
                );
                let strike_vfx = animations().get("strike_vfx").unwrap();
                battle.apply_animation(Context::default(), strike_vfx);
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
                let pos = Context::new_battle_simulation(&battle)
                    .set_owner(*b)
                    .get_var(VarName::position)
                    .unwrap();
                let text = animations().get("text").unwrap();
                battle.apply_animation(
                    Context::default()
                        .set_var(VarName::text, (-*x).to_string().into())
                        .set_var(VarName::color, RED.into())
                        .set_var(VarName::position, pos.clone())
                        .take(),
                    text,
                );
                if *x > 0 {
                    let pain = animations().get("pain_vfx").unwrap();
                    battle.apply_animation(
                        Context::default().set_var(VarName::position, pos).take(),
                        pain,
                    );
                    let hp = battle.world.get::<UnitStats>(*b).unwrap().hp - x;
                    add_actions.push(Self::VarSet(
                        *b,
                        NodeKind::UnitStats,
                        VarName::hp,
                        hp.into(),
                    ));
                }
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
            BattleAction::ApplyStatus(entity) => {
                battle.apply_status(*entity);
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
        for (_, u) in battle.left.units.iter().enumerate() {
            let entity = world.spawn_empty().id();
            u.clone().unpack(entity, &mut world.commands());
            left.push(entity);
            log.add_state(entity, &mut world);
        }
        for (_, u) in battle.right.units.iter().enumerate() {
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
    fn apply_animation(&mut self, context: Context, anim: &Anim) {
        match anim.apply(&mut self.t, context, &mut self.world) {
            Ok(_) => {}
            Err(e) => error!("Animation error: {e}"),
        }
    }
    fn process_actions(&mut self, mut actions: VecDeque<BattleAction>) {
        while let Some(a) = actions.pop_front() {
            for a in a.apply(self) {
                actions.push_front(a);
            }
        }
    }
    fn apply_status(&mut self, target: Entity) {
        let status = Status {
            name: "Test Status".into(),
            description: Some(StatusDescription {
                description: "Test status desc".into(),
                trigger: Some(StatusTrigger {
                    trigger: Trigger::TurnEnd,
                    target: Expression::RandomUnit(Box::new(Expression::AllUnits)),
                    effect: Effect::Damage,
                    ..default()
                }),
                ..default()
            }),
            ..default()
        };
        let entity = self.world.spawn_empty().set_parent(target).id();
        status.unpack(entity, &mut self.world.commands());
        self.world.flush_commands();
        let mut state = NodeState::from_world_mut(entity, &mut self.world).unwrap();
        state.insert(0.0, 0.0, VarName::visible, false.into(), default());
        state.insert(self.t, 0.0, VarName::visible, true.into(), default());
    }
    fn send_event(&mut self, event: Event) {
        let mut actions = Vec::default();
        fn trigger_fire(
            entity: Entity,
            actions: &mut Vec<BattleAction>,
            bs: &BattleSimulation,
            event: &Event,
            trigger: &Trigger,
            target: &Expression,
            effect: &Effect,
        ) {
            if match event {
                Event::BattleStart => matches!(trigger, Trigger::BattleStart),
                Event::TurnEnd => matches!(trigger, Trigger::TurnEnd),
            } {
                let mut context = Context::new_battle_simulation(bs).set_owner(entity).take();
                match target.get_entity(&context) {
                    Ok(target) => {
                        context.set_target(target);
                    }
                    Err(e) => {
                        error!("Get target error: {e}")
                    }
                }
                match effect.process(&context) {
                    Ok(a) => {
                        actions.extend(a);
                    }
                    Err(e) => {
                        error!("Effect process error: {e}")
                    }
                }
            }
        }
        let mut alive_units: HashSet<Entity> = default();
        for (entity, ut) in self
            .world
            .query_filtered::<(Entity, &UnitTrigger), Without<Corpse>>()
            .iter(&self.world)
        {
            alive_units.insert(entity);
            trigger_fire(
                entity,
                &mut actions,
                self,
                &event,
                &ut.trigger,
                &ut.target,
                &ut.effect,
            );
        }
        for (entity, parent, st) in self
            .world
            .query::<(Entity, &Parent, &StatusTrigger)>()
            .iter(&self.world)
        {
            if !alive_units.contains(&parent.get()) {
                continue;
            }
            trigger_fire(
                entity,
                &mut actions,
                self,
                &event,
                &st.trigger,
                &st.target,
                &st.effect,
            );
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
        self.send_event(Event::TurnEnd);
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
        let rect = slot_rect(i, side, full_rect, self.slots);
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
        let center_rect = slot_rect(0, true, ui.available_rect_before_wrap(), self.slots);
        let up = center_rect.width() * 0.5;
        for (slot, side) in (1..=self.slots).cartesian_product([true, false]) {
            self.show_slot(slot, side, ui);
        }
        let unit_size = center_rect.width() * UNIT_SIZE;

        let mut entities: VecDeque<Entity> = self
            .world
            .query_filtered::<Entity, Without<Parent>>()
            .iter(&self.world)
            .collect();
        let context = Context::new_world(&self.world).set_t(t).take();
        while let Some(entity) = entities.pop_front() {
            let context = context.clone().set_owner(entity).take();
            if context.get_bool(VarName::visible).unwrap_or(true) {
                entities.extend(context.get_children(entity));
                if let Some(rep) = self.world.get::<Representation>(entity) {
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
    }
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
