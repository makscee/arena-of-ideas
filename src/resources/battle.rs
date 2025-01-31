use egui::lerp;

use super::*;

pub struct Battle {
    pub left: Team,
    pub right: Team,
}
#[derive(Debug)]
pub struct BattleSimulation {
    pub t: f32,
    pub world: World,
    pub fusions_left: Vec<Entity>,
    pub fusions_right: Vec<Entity>,
    pub log: BattleLog,
}
#[derive(Default, Debug)]
pub struct BattleLog {
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
impl BattleAction {
    pub fn apply(&self, battle: &mut BattleSimulation) -> Vec<Self> {
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
                let pos = Context::new_battle_simulation(battle)
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

impl Battle {
    pub fn open_window(self, world: &mut World) {
        let mut bs = BattleSimulation::new(self).unwrap().start();
        let mut t = 0.0;
        let mut playing = false;
        Window::new("Battle", move |ui, _| {
            ui.set_min_size(egui::vec2(800.0, 400.0));
            Slider::new("ts").full_width().ui(&mut t, 0.0..=bs.t, ui);
            ui.horizontal(|ui| {
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
            });
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

impl BattleSimulation {
    pub fn new(battle: Battle) -> Result<Self, ExpressionError> {
        let mut world = World::new();
        for k in NodeKind::iter() {
            k.register_world(&mut world);
        }
        let team_left = world.spawn_empty().id();
        let team_right = world.spawn_empty().id();
        battle.left.unpack(team_left, &mut world.commands());
        battle.right.unpack(team_right, &mut world.commands());
        world.flush();
        for fusion in world.query::<&Fusion>().iter(&world).cloned().collect_vec() {
            fusion.init(&mut world)?;
        }
        fn entities_by_slot(parent: Entity, world: &World) -> Vec<Entity> {
            Context::new_world(&world)
                .children_components_recursive::<UnitSlot>(parent)
                .into_iter()
                .sorted_by_key(|(_, s)| s.slot)
                .map(|(e, _)| e)
                .collect_vec()
        }
        let fusions_left = entities_by_slot(team_left, &world);
        let fusions_right = entities_by_slot(team_right, &world);
        Ok(Self {
            world,
            fusions_left,
            fusions_right,
            t: 0.0,
            log: BattleLog::default(),
        })
    }
    pub fn start(mut self) -> Self {
        let spawn_actions = self
            .fusions_left
            .iter()
            .zip_longest(self.fusions_right.iter())
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
        self.send_event(Event::BattleStart);
        self
    }
    pub fn run(&mut self) {
        if self.ended() {
            return;
        }
        let a = BattleAction::Strike(self.fusions_left[0], self.fusions_right[0]);
        self.process_actions([a]);
        let a = self.death_check();
        self.process_actions(a);
        self.process_actions(self.slots_sync());
        self.send_event(Event::TurnEnd);
    }
    pub fn ended(&self) -> bool {
        self.fusions_left.is_empty() || self.fusions_right.is_empty()
    }
    fn send_event(&mut self, event: Event) {
        info!("{} {event}", "event:".dimmed().blue());
        let mut actions: Vec<BattleAction> = default();
        for f in self
            .world
            .query_filtered::<&Fusion, Without<Corpse>>()
            .iter(&self.world)
        {
            let mut context = Context::new_battle_simulation(self)
                .set_owner(f.entity())
                .take();
            match f.react(&event, &mut context) {
                Ok(a) => actions.extend(a),
                Err(e) => error!("Fusion event {event} failed: {e}"),
            }
        }
        for (r, s) in self.world.query::<(&Reaction, &Status)>().iter(&self.world) {
            if r.react(&event) {
                match r
                    .actions
                    .process(Context::new_battle_simulation(self).set_owner(s.entity()))
                {
                    Ok(a) => actions.extend(a),
                    Err(e) => error!("Status {} event {event} failed: {e}", s.name),
                };
            }
        }
        self.process_actions(actions);
    }
    fn apply_status(&mut self, target: Entity) {
        let status = Status {
            name: "Test Status".into(),
            description: Some(StatusDescription {
                description: "Test status desc".into(),
                reaction: Some(Reaction {
                    trigger: Trigger::TurnEnd,
                    actions: [
                        Action::SetTarget(Box::new(Expression::RandomUnit(Box::new(
                            Expression::AllUnits,
                        )))),
                        Action::SetValue(Box::new(Expression::I(1))),
                        Action::DealDamage,
                    ]
                    .to_vec()
                    .into(),
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
    fn apply_animation(&mut self, context: Context, anim: &Anim) {
        match anim.apply(&mut self.t, context, &mut self.world) {
            Ok(_) => {}
            Err(e) => error!("Animation error: {e}"),
        }
    }
    fn process_actions(&mut self, actions: impl Into<VecDeque<BattleAction>>) {
        let mut actions = actions.into();
        while let Some(a) = actions.pop_front() {
            for a in a.apply(self) {
                actions.push_front(a);
            }
        }
    }
    fn death_check(&mut self) -> VecDeque<BattleAction> {
        let mut actions: VecDeque<BattleAction> = default();
        for (entity, stats) in self
            .world
            .query_filtered::<(Entity, &UnitStats), (Without<Corpse>, With<Fusion>)>()
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
        if let Some(p) = self.fusions_left.iter().position(|u| *u == entity) {
            self.fusions_left.remove(p);
            died = true;
        }
        if let Some(p) = self.fusions_right.iter().position(|u| *u == entity) {
            self.fusions_right.remove(p);
            died = true;
        }
        if died {
            if self.ended() {
                self.t += 1.0;
            }
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
            .fusions_left
            .iter()
            .map(|e| (e, true))
            .enumerate()
            .chain(self.fusions_right.iter().map(|e| (e, false)).enumerate())
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
    fn fusion_by_slot<'a>(&'a self, slot: usize, side: bool) -> Option<&'a Fusion> {
        let entity = if side {
            &self.fusions_left
        } else {
            &self.fusions_right
        }
        .get(slot)?;
        self.world.get::<Fusion>(*entity)
    }
    fn pack_units_by_slot(&self, slot: usize, side: bool) -> Vec<Unit> {
        if let Some(f) = self.fusion_by_slot(slot, side) {
            if let Ok(units) = f.units(&Context::new_battle_simulation(self)) {
                return units
                    .into_iter()
                    .map(|u| Unit::pack(u, &self.world).unwrap())
                    .collect_vec();
            }
        }
        default()
    }
    fn show_slot(&self, slot: usize, side: bool, slots: usize, ui: &mut Ui) -> Response {
        let slot = slot + 1;
        let full_rect = ui.available_rect_before_wrap();
        let rect = slot_rect(slot, side, full_rect, slots);
        ui.expand_to_include_rect(rect);
        let mut cui = ui.child_ui(rect, *ui.layout(), None);
        let r = cui.allocate_rect(rect, Sense::click());
        let mut stroke = if r.hovered() {
            STROKE_YELLOW
        } else {
            STROKE_DARK
        };
        let t = cui
            .ctx()
            .animate_bool(Id::new("slot_hovered").with(slot).with(side), r.hovered());
        let length = lerp(15.0..=20.0, t);
        stroke.width += t;
        corners_rounded_rect(r.rect.shrink(3.0), length, stroke, ui);
        r
    }
    fn show_card_from_units(units: &Vec<Unit>, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for unit in units {
                ui.vertical(|ui| {
                    unit.show(None, Context::default().set_owner_node(unit), ui);
                });
            }
        });
    }
    fn show_card(&self, slot: usize, side: bool, ui: &mut Ui) {
        cursor_window(ui.ctx(), |ui| {
            Self::show_card_from_units(&self.pack_units_by_slot(slot, side), ui);
        });
    }
    pub fn show_at(&mut self, t: f32, ui: &mut Ui) {
        let slots = global_settings().team_slots as usize;
        let center_rect = slot_rect(0, true, ui.available_rect_before_wrap(), slots);
        let unit_size = center_rect.width() * UNIT_SIZE;
        let unit_pixels = center_rect.width() * 0.5;
        for (slot, side) in (0..slots).cartesian_product([true, false]) {
            let resp = self.show_slot(slot, side, slots, ui);
            if resp.hovered() {
                self.show_card(slot, side, ui);
            }
            if resp.clicked() {
                let units = self.pack_units_by_slot(slot, side);
                OperationsPlugin::add(move |world| {
                    Window::new("Unit Card", move |ui, _| {
                        Self::show_card_from_units(&units, ui);
                    })
                    .order(Order::Foreground)
                    .push(world);
                });
            }
        }
        let fusions: HashSet<Entity> = HashSet::from_iter(
            self.world
                .query_filtered::<Entity, With<Fusion>>()
                .iter(&self.world),
        );
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
                        * unit_pixels;
                    let rect = Rect::from_center_size(
                        center_rect.center() + position,
                        egui::Vec2::splat(unit_size),
                    );
                    if fusions.contains(&entity) {
                        let fusion = self.world.get::<Fusion>(entity).unwrap();
                        fusion.paint(rect, ui, &self.world).log();
                    }
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
