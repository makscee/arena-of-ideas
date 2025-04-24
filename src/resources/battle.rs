use super::*;

#[derive(Clone, Debug, Default)]
pub struct Battle {
    pub id: u64,
    pub left: NTeam,
    pub right: NTeam,
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
    apply_status(Entity, NStatusMagic, i32, Color32),
    send_event(Event),
    vfx(HashMap<VarName, VarValue>, String),
    wait(f32),
}
impl Hash for BattleAction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            BattleAction::var_set(entity, kind, var, value) => {
                entity.hash(state);
                kind.hash(state);
                var.hash(state);
                value.hash(state);
            }
            BattleAction::strike(a, b) => {
                a.hash(state);
                b.hash(state);
            }
            BattleAction::damage(a, b, v) | BattleAction::heal(a, b, v) => {
                a.hash(state);
                b.hash(state);
                v.hash(state);
            }
            BattleAction::death(a) | BattleAction::spawn(a) => a.hash(state),
            BattleAction::apply_status(a, status, c, _) => {
                a.hash(state);
                status.hash(state);
                c.hash(state);
            }
            BattleAction::send_event(event) => event.hash(state),
            _ => {
                return;
            }
        }
        core::mem::discriminant(self).hash(state);
    }
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
                let context = &Context::new(&battle.world);
                let pwr = context
                    .get_node::<NFusion>(*a)
                    .unwrap()
                    .pwr_hp(context)
                    .unwrap()
                    .0;
                let action_a = Self::damage(*a, *b, pwr);
                let pwr = context
                    .get_node::<NFusion>(*b)
                    .unwrap()
                    .pwr_hp(context)
                    .unwrap()
                    .0;
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
                    let dmg = battle.world.get::<NFusionStats>(*b).unwrap().dmg + x;
                    add_actions.push(Self::var_set(
                        *b,
                        NodeKind::NFusionStats,
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
                    let dmg = (battle.world.get::<NFusionStats>(*b).unwrap().dmg - x).at_least(0);
                    add_actions.push(Self::var_set(
                        *b,
                        NodeKind::NFusionStats,
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
        battle.left.unpack_entity(team_left, &mut world);
        battle.right.unpack_entity(team_right, &mut world);

        for entity in world
            .query_filtered::<Entity, With<NHouse>>()
            .iter(&world)
            .collect_vec()
        {
            world
                .run_system_once_with((entity, 0.0), NodeStatePlugin::inject_entity_vars)
                .unwrap();
        }
        fn entities_by_slot(parent: Entity, world: &World) -> Vec<Entity> {
            Context::new(&world)
                .children_nodes_recursive::<NFusion>(parent)
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
            .query_filtered::<Entity, (With<NFusion>, Without<Corpse>)>()
            .iter(&self.world)
            .collect_vec()
        {
            let vars = self
                .world
                .run_system_once_with(entity, NodeStatePlugin::collect_vars)
                .unwrap();
            for (var, value) in vars {
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
        if let Some(fusion) = self.world.get::<NFusion>(entity) {
            fusion.react(event, &mut context).log();
        }
        for child in entity.get_children(&self.world) {
            if let Some(reaction) = self.world.get::<NBehavior>(child) {
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
            .query_filtered::<&NFusion, Without<Corpse>>()
            .iter(&self.world)
        {
            let mut context = Context::new_battle_simulation(self)
                .set_owner(f.entity())
                .take();
            match f.react(&event, &mut context) {
                Ok(a) => battle_actions.extend(a),
                Err(e) => error!("NFusion event {event} failed: {e}"),
            }
        }
        for (r, s) in self
            .world
            .query::<(&NBehavior, &NStatusMagic)>()
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
    fn apply_status(&mut self, target: Entity, status: NStatusMagic, charges: i32, color: Color32) {
        for child in target.get_children(&self.world) {
            if let Some(child_status) = self.world.get::<NStatusMagic>(child) {
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
        status.unpack_entity(entity, &mut self.world);

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
        for (entity, stats, fusion) in self
            .world
            .query_filtered::<(Entity, &NFusionStats, &NFusion), Without<Corpse>>()
            .iter(&self.world)
        {
            if stats.dmg >= fusion.pwr_hp(&Context::new(&self.world)).unwrap().1 {
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
}
