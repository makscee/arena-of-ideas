use super::*;

#[derive(Clone, Debug, Default)]
pub struct Battle {
    pub left: Team,
    pub right: Team,
}
#[derive(Debug)]
pub struct BattleSimulation {
    pub duration: f32,
    pub world: World,
    pub fusions_left: Vec<Entity>,
    pub fusions_right: Vec<Entity>,
    pub team_left: Entity,
    pub team_right: Entity,
    pub log: BattleLog,
}
#[derive(Default, Debug)]
pub struct BattleLog {
    pub actions: Vec<BattleAction>,
}

#[derive(Component)]
pub struct Corpse;
#[derive(Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum BattleAction {
    var_set(Entity, NodeKind, VarName, VarValue),
    strike(Entity, Entity),
    damage(Entity, Entity, i32),
    heal(Entity, Entity, i32),
    death(Entity),
    spawn(Entity),
    apply_status(Entity, StatusMagic, i32, Color32),
    send_event(Event),
    vfx(HashMap<VarName, VarValue>, String),
    wait(f32),
}
impl ToCstr for BattleAction {
    fn cstr(&self) -> Cstr {
        match self {
            BattleAction::strike(a, b) => format!("{a}|{b}"),
            BattleAction::damage(a, b, x) => format!("{a}>{b}-{x}"),
            BattleAction::heal(a, b, x) => format!("{a}>{b}+{x}"),
            BattleAction::death(a) => format!("x{a}"),
            BattleAction::var_set(a, _, var, value) => format!("{a}>${var}>{value}"),
            BattleAction::spawn(a) => format!("*{a}"),
            BattleAction::apply_status(a, status, charges, color) => {
                format!(
                    "+[{} {}]>{a}({charges})",
                    color.to_hex(),
                    status.status_name
                )
            }
            BattleAction::wait(t) => format!("~{t}"),
            BattleAction::vfx(_, vfx) => format!("vfx({vfx})"),
            BattleAction::send_event(e) => format!("event({e})"),
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
            BattleAction::strike(a, b) => {
                let strike_anim = animations().get("strike").unwrap();
                battle.apply_animation(
                    Context::default()
                        .set_owner(*a)
                        .add_target(*b)
                        .set_var(VarName::position, vec2(0.0, 0.0).into())
                        .take(),
                    strike_anim,
                );
                let pwr = battle.world.get::<UnitStats>(*a).unwrap().pwr;
                let action_a = Self::damage(*a, *b, pwr);
                let pwr = battle.world.get::<UnitStats>(*b).unwrap().pwr;
                let action_b = Self::damage(*b, *a, pwr);
                add_actions.extend_from_slice(&[action_a, action_b]);
                add_actions.extend(battle.slots_sync());
                true
            }
            BattleAction::death(a) => {
                let position = Context::new_battle_simulation(battle)
                    .set_owner(*a)
                    .get_var(VarName::position)
                    .unwrap();
                add_actions.extend(battle.die(*a));
                add_actions.push(BattleAction::vfx(
                    HashMap::from_iter([(VarName::position, position)]),
                    "death_vfx".into(),
                ));
                true
            }
            BattleAction::damage(a, b, x) => {
                let owner_pos = Context::new_battle_simulation(battle)
                    .set_owner(*a)
                    .get_var(VarName::position)
                    .unwrap();
                let target_pos = Context::new_battle_simulation(battle)
                    .set_owner(*b)
                    .get_var(VarName::position)
                    .unwrap();
                let curve = animations().get("range_effect_vfx").unwrap();
                battle.apply_animation(
                    Context::default()
                        .set_var(VarName::position, owner_pos)
                        .set_var(VarName::extra_position, target_pos.clone())
                        .take(),
                    curve,
                );
                if *x > 0 {
                    let pain = animations().get("pain_vfx").unwrap();
                    battle.apply_animation(
                        Context::default()
                            .set_var(VarName::position, target_pos.clone())
                            .take(),
                        pain,
                    );
                    let dmg = battle.world.get::<UnitStats>(*b).unwrap().dmg + x;
                    add_actions.push(Self::var_set(
                        *b,
                        NodeKind::UnitStats,
                        VarName::dmg,
                        dmg.into(),
                    ));
                }
                let text = animations().get("text").unwrap();
                battle.apply_animation(
                    Context::default()
                        .set_var(VarName::text, (-*x).to_string().into())
                        .set_var(VarName::color, RED.into())
                        .set_var(VarName::position, target_pos)
                        .take(),
                    text,
                );
                battle.duration += ANIMATION;
                true
            }
            BattleAction::heal(a, b, x) => {
                let owner_pos = Context::new_battle_simulation(battle)
                    .set_owner(*a)
                    .get_var(VarName::position)
                    .unwrap();
                let target_pos = Context::new_battle_simulation(battle)
                    .set_owner(*b)
                    .get_var(VarName::position)
                    .unwrap();
                let curve = animations().get("range_effect_vfx").unwrap();
                battle.apply_animation(
                    Context::default()
                        .set_var(VarName::position, owner_pos)
                        .set_var(VarName::extra_position, target_pos.clone())
                        .take(),
                    curve,
                );
                if *x > 0 {
                    let pain = animations().get("pleasure_vfx").unwrap();
                    battle.apply_animation(
                        Context::default()
                            .set_var(VarName::position, target_pos.clone())
                            .take(),
                        pain,
                    );
                    let dmg = (battle.world.get::<UnitStats>(*b).unwrap().dmg - x).at_least(0);
                    add_actions.push(Self::var_set(
                        *b,
                        NodeKind::UnitStats,
                        VarName::dmg,
                        dmg.into(),
                    ));
                    let text = animations().get("text").unwrap();
                    battle.apply_animation(
                        Context::default()
                            .set_var(VarName::text, format!("+{x}").into())
                            .set_var(VarName::color, GREEN.into())
                            .set_var(VarName::position, target_pos)
                            .take(),
                        text,
                    );
                }
                battle.duration += ANIMATION;
                true
            }
            BattleAction::var_set(entity, kind, var, value) => {
                if battle.world.get_mut::<NodeState>(*entity).unwrap().insert(
                    battle.duration,
                    0.1,
                    *var,
                    value.clone(),
                ) {
                    kind.set_var(*entity, *var, value.clone(), &mut battle.world);
                    true
                } else {
                    false
                }
            }
            BattleAction::spawn(entity) => {
                battle
                    .world
                    .run_system_once_with(
                        (*entity, battle.duration),
                        NodeStatePlugin::inject_entity_vars,
                    )
                    .unwrap();
                add_actions.extend_from_slice(&[BattleAction::var_set(
                    *entity,
                    NodeKind::None,
                    VarName::visible,
                    true.into(),
                )]);
                true
            }
            BattleAction::apply_status(target, status, charges, color) => {
                battle.apply_status(*target, status.clone(), *charges, *color);
                battle.duration += ANIMATION;
                true
            }
            BattleAction::wait(t) => {
                battle.duration += *t;
                false
            }
            BattleAction::vfx(vars, vfx) => {
                if let Some(vfx) = animations().get(vfx) {
                    let mut context = Context::default();
                    for (var, value) in vars {
                        context.set_var(*var, value.clone());
                    }
                    battle.apply_animation(context, vfx);
                }
                false
            }
            BattleAction::send_event(event) => {
                add_actions.extend(battle.send_event(*event));
                true
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

impl BattleSimulation {
    pub fn new(battle: Battle) -> Self {
        let mut world = World::new();
        for k in NodeKind::iter() {
            k.register_world(&mut world);
        }
        let team_left = world.spawn_empty().id();
        let team_right = world.spawn_empty().id();
        battle.left.unpack(team_left, &mut world);
        battle.right.unpack(team_right, &mut world);

        for entity in world
            .query_filtered::<Entity, With<House>>()
            .iter(&world)
            .collect_vec()
        {
            world
                .run_system_once_with((entity, 0.0), NodeStatePlugin::inject_entity_vars)
                .unwrap();
        }
        fn entities_by_slot(parent: Entity, world: &World) -> Vec<Entity> {
            Context::new_world(&world)
                .children_components_recursive::<Fusion>(parent)
                .into_iter()
                .sorted_by_key(|s| s.slot)
                .map(|n| n.entity())
                .collect_vec()
        }
        let fusions_left = entities_by_slot(team_left, &world);
        let fusions_right = entities_by_slot(team_right, &world);
        Self {
            world,
            fusions_left,
            fusions_right,
            team_left,
            team_right,
            duration: 0.0,
            log: BattleLog::default(),
        }
    }
    pub fn start(mut self) -> Self {
        let spawn_actions = self
            .fusions_left
            .iter()
            .zip_longest(self.fusions_right.iter())
            .flat_map(|e| match e {
                EitherOrBoth::Both(a, b) => {
                    vec![BattleAction::spawn(*a), BattleAction::spawn(*b)]
                }
                EitherOrBoth::Left(e) | EitherOrBoth::Right(e) => {
                    vec![BattleAction::spawn(*e)]
                }
            })
            .collect_vec();
        self.process_actions(spawn_actions);
        self.process_actions(self.slots_sync());
        let actions = self.send_event(Event::BattleStart);
        self.process_actions(actions);
        self
    }
    pub fn run(&mut self) {
        if self.ended() {
            return;
        }
        for entity in self
            .world
            .query_filtered::<Entity, (With<Fusion>, Without<Corpse>)>()
            .iter(&self.world)
            .collect_vec()
        {
            let vars = self
                .world
                .run_system_once_with(entity, NodeStatePlugin::collect_vars)
                .unwrap();
            for (var, (value, source)) in vars {
                let value = self.send_update_event(entity, var, value);
                NodeState::from_world_mut(entity, &mut self.world)
                    .unwrap()
                    .insert(self.duration, 0.0, var, value);
            }
        }
        let a = BattleAction::strike(self.fusions_left[0], self.fusions_right[0]);
        self.process_actions([a]);
        let a = self.death_check();
        self.process_actions(a);
        self.process_actions(self.slots_sync());
        let a = self.send_event(Event::TurnEnd);
        self.process_actions(a);
    }
    pub fn ended(&self) -> bool {
        self.fusions_left.is_empty() || self.fusions_right.is_empty()
    }
    fn send_update_event(&mut self, entity: Entity, var: VarName, value: VarValue) -> VarValue {
        let mut context = Context::new_battle_simulation(self)
            .set_owner(entity)
            .set_value(value)
            .take();
        let event = &Event::UpdateStat(var);
        if let Some(fusion) = self.world.get::<Fusion>(entity) {
            fusion.react(event, &mut context).log();
        }
        for child in entity.get_children(&self.world) {
            if let Some(reaction) = self.world.get::<Behavior>(child) {
                let mut status_context = context.clone().set_owner(child).take();
                if let Some(actions) = reaction.react(event, &status_context) {
                    match actions.process(&mut status_context) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Update event {event} failed: {e}");
                            continue;
                        }
                    }
                }
                context.set_value(status_context.get_value().unwrap());
            }
        }
        context.get_value().unwrap()
    }
    #[must_use]
    fn send_event(&mut self, event: Event) -> VecDeque<BattleAction> {
        info!("{} {event}", "event:".dimmed().blue());
        let mut battle_actions: VecDeque<BattleAction> = default();
        for f in self
            .world
            .query_filtered::<&Fusion, Without<Corpse>>()
            .iter(&self.world)
        {
            let mut context = Context::new_battle_simulation(self)
                .set_owner(f.entity())
                .take();
            match f.react(&event, &mut context) {
                Ok(a) => battle_actions.extend(a),
                Err(e) => error!("Fusion event {event} failed: {e}"),
            }
        }
        for (r, s) in self
            .world
            .query::<(&Behavior, &StatusMagic)>()
            .iter(&self.world)
        {
            let context = Context::new_battle_simulation(self)
                .set_owner(s.entity())
                .take();
            if let Some(actions) = r.react(&event, &context) {
                match actions.process(Context::new_battle_simulation(self).set_owner(s.entity())) {
                    Ok(a) => battle_actions.extend(a),
                    Err(e) => error!("StatusMagic {} event {event} failed: {e}", s.status_name),
                };
            }
        }
        battle_actions
    }
    fn apply_status(&mut self, target: Entity, status: StatusMagic, charges: i32, color: Color32) {
        for child in target.get_children(&self.world) {
            if let Some(child_status) = self.world.get::<StatusMagic>(child) {
                if child_status.status_name == status.status_name {
                    let mut state = NodeState::from_world_mut(child, &mut self.world).unwrap();
                    let charges = state
                        .get(VarName::charges)
                        .map(|v| v.get_i32().unwrap())
                        .unwrap()
                        + charges;
                    state.insert(self.duration, 0.0, VarName::charges, charges.into());
                    return;
                }
            }
        }
        let entity = self.world.spawn_empty().set_parent(target).id();
        status.unpack(entity, &mut self.world);

        let mut state = NodeState::from_world_mut(entity, &mut self.world).unwrap();
        state.insert(0.0, 0.0, VarName::visible, false.into());
        state.insert(self.duration, 0.0, VarName::visible, true.into());
        state.insert(self.duration, 0.0, VarName::charges, charges.into());
        state.insert(self.duration, 0.0, VarName::color, color.into());
    }
    fn apply_animation(&mut self, context: Context, anim: &Anim) {
        match anim.apply(&mut self.duration, context, &mut self.world) {
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
    #[must_use]
    fn death_check(&mut self) -> VecDeque<BattleAction> {
        let mut actions: VecDeque<BattleAction> = default();
        for (entity, stats) in self
            .world
            .query_filtered::<(Entity, &UnitStats), (Without<Corpse>, With<Fusion>)>()
            .iter(&self.world)
        {
            if stats.dmg >= stats.hp {
                actions.push_back(BattleAction::send_event(Event::Death(entity.to_bits())));
                actions.push_back(BattleAction::death(entity));
            }
        }
        actions
    }
    #[must_use]
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
                self.duration += 1.0;
            }
            [
                BattleAction::var_set(entity, NodeKind::None, VarName::visible, false.into()),
                BattleAction::wait(ANIMATION),
            ]
            .into()
        } else {
            default()
        }
    }
    #[must_use]
    fn slots_sync(&self) -> VecDeque<BattleAction> {
        let mut actions = VecDeque::default();
        for (i, (e, side)) in self
            .fusions_left
            .iter()
            .map(|e| (e, true))
            .enumerate()
            .chain(self.fusions_right.iter().map(|e| (e, false)).enumerate())
        {
            actions.push_back(BattleAction::var_set(
                *e,
                NodeKind::None,
                VarName::slot,
                i.into(),
            ));
            actions.push_back(BattleAction::var_set(
                *e,
                NodeKind::None,
                VarName::side,
                side.into(),
            ));
            let position = vec2((i + 1) as f32 * if side { -1.0 } else { 1.0 } * 2.0, 0.0);
            actions.push_back(BattleAction::var_set(
                *e,
                NodeKind::None,
                VarName::position,
                position.into(),
            ));
        }
        actions.push_back(BattleAction::wait(ANIMATION * 3.0));
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
                    .map(|u| Unit::pack(u.entity(), &self.world).unwrap())
                    .collect_vec();
            }
        }
        default()
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
        let center_rect = slot_rect(
            slot_by_side(0, slots, true),
            slots * 2 + 1,
            ui.available_rect_before_wrap(),
            true,
        );
        let unit_size = center_rect.width() * UNIT_SIZE;
        let unit_pixels = center_rect.width() * 0.5;
        for (slot, side) in (0..slots).cartesian_product([true, false]) {
            let resp = show_battle_slot(slot + 1, slots, side, ui);
            if resp.hovered() {
                self.show_card(slot, side, ui);
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
                    let fusion = context.get_component::<Fusion>(entity).unwrap();
                    fusion.paint(rect, &context, ui).log();
                }
            }
        }
    }
}

pub fn show_battle_slot(i: usize, slots: usize, player_side: bool, ui: &mut Ui) -> Response {
    let i = slot_by_side(i, slots, player_side);
    show_slot(i, slots * 2 + 1, true, ui)
}
fn slot_by_side(i: usize, team_slots: usize, player_side: bool) -> usize {
    if player_side {
        team_slots - i
    } else {
        team_slots + i
    }
}
