use super::*;

pub struct BattleOld {
    pub left: Team,
    pub right: Team,
}
#[derive(Debug)]
pub struct BattleSimulationOld {
    pub t: f32,
    pub world: World,
    pub left_team: Entity,
    pub right_team: Entity,
    pub left_units: Vec<Entity>,
    pub right_units: Vec<Entity>,
    pub log: BattleLog,
    pub slots: usize,
}
#[derive(Default, Debug)]
pub struct BattleLog {
    pub states: HashMap<Entity, NodeState>,
    pub actions: Vec<BattleAction>,
}

impl BattleOld {
    pub fn open_window(&self, world: &mut World) {
        let mut bs = BattleSimulationOld::new(self).start();
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

impl BattleSimulationOld {
    pub fn new(battle: &BattleOld) -> Self {
        let mut world = World::new();
        for k in NodeKind::iter() {
            k.register_world(&mut world);
        }
        let mut left_units: Vec<Entity> = default();
        let mut right_units: Vec<Entity> = default();
        let mut log = BattleLog::default();
        let left_team = world.spawn_empty().id();
        let right_team = world.spawn_empty().id();
        battle.left.clone().unpack(left_team, &mut world.commands());
        battle
            .right
            .clone()
            .unpack(right_team, &mut world.commands());
        for e in Context::new_world(&world)
            .children_components::<Fusion>(left_team)
            .into_iter()
            .map(|(e, _)| e)
            .collect_vec()
        {
            left_units.push(e);
        }
        for e in Context::new_world(&world)
            .children_components::<Fusion>(right_team)
            .into_iter()
            .map(|(e, _)| e)
            .collect_vec()
        {
            right_units.push(e);
        }
        world.flush();
        Self {
            t: 0.0,
            world,
            left_team,
            right_team,
            left_units,
            right_units,
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
            // for a in a.apply(self) {
            //     actions.push_front(a);
            // }
        }
    }
    fn apply_status(&mut self, target: Entity) {
        let status = Status {
            name: "Test Status".into(),
            description: Some(StatusDescription {
                description: "Test status desc".into(),
                reaction: Some(Reaction {
                    trigger: Trigger::TurnEnd,
                    actions: vec![Action::DealDamage].into(),
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
        fn react(
            entity: Entity,
            actions: &mut Vec<BattleAction>,
            bs: &BattleSimulationOld,
            event: &Event,
            reaction: &Reaction,
        ) {
            if reaction.react(event) {
                match reaction
                    .actions
                    .process(Context::default().set_owner(entity))
                {
                    Ok(a) => actions.extend(a),
                    Err(e) => {
                        error!("React error: {e}")
                    }
                }
            }
        }
        let mut alive_units: HashSet<Entity> = default();
        for (entity, r) in self
            .world
            .query_filtered::<(Entity, &Reaction), Without<Corpse>>()
            .iter(&self.world)
        {
            alive_units.insert(entity);
            react(entity, &mut actions, self, &event, r);
        }
        for (entity, parent, r) in self
            .world
            .query::<(Entity, &Parent, &Reaction)>()
            .iter(&self.world)
        {
            if !alive_units.contains(&parent.get()) {
                continue;
            }
            react(entity, &mut actions, self, &event, r);
        }
        self.process_actions(actions.into());
    }
    pub fn start(mut self) -> Self {
        let spawn_actions = self
            .left_units
            .iter()
            .zip_longest(self.right_units.iter())
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
        let a = BattleAction::Strike(self.left_units[0], self.right_units[0]);
        self.process_actions([a].into());
        let a = self.death_check();
        self.process_actions(a);
        self.process_actions(self.slots_sync());
        self.send_event(Event::TurnEnd);
    }
    pub fn ended(&self) -> bool {
        self.left_units.is_empty() || self.right_units.is_empty()
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
        if let Some(p) = self.left_units.iter().position(|u| *u == entity) {
            self.left_units.remove(p);
            died = true;
        }
        if let Some(p) = self.right_units.iter().position(|u| *u == entity) {
            self.right_units.remove(p);
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
            .left_units
            .iter()
            .map(|e| (e, true))
            .enumerate()
            .chain(self.right_units.iter().map(|e| (e, false)).enumerate())
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
    let mut rect = full_rect.shrink(3.0);
    rect.set_height(size);
    rect.set_width(size);
    rect.translate(egui::vec2(size * pos_i, 0.0))
}
