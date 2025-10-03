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
    pub seed: u64,
}
#[derive(Default, Debug)]
pub struct BattleLog {
    pub actions: Vec<BattleAction>,
}

#[derive(BevyComponent)]
pub struct Corpse;
#[derive(Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum BattleAction {
    var_set(Entity, VarName, VarValue),
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
            BattleAction::var_set(entity, var, value) => {
                entity.hash(state);
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
            BattleAction::var_set(a, var, value) => format!("{a}>${var}>{value}"),
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
    pub fn apply(&self, sim: &mut BattleSimulation) -> Vec<Self> {
        let mut add_actions = Vec::default();
        let result = Context::from_battle_simulation_r(sim, |context| {
            let applied = match self {
                BattleAction::strike(a, b) => {
                    let strike_anim = animations().get("strike").unwrap();
                    context.with_layers_r(
                        [
                            ContextLayer::Owner(*a),
                            ContextLayer::Target(*b),
                            ContextLayer::Var(VarName::position, vec2(0.0, 0.0).into()),
                        ]
                        .into(),
                        |context| strike_anim.apply(context),
                    )?;
                    let pwr = context
                        .with_layer_r(ContextLayer::Owner(*a), |context| {
                            context.sum_var(VarName::pwr)
                        })?
                        .get_i32()?;
                    let action_a = Self::damage(*a, *b, pwr);
                    let pwr = context
                        .with_layer_r(ContextLayer::Owner(*b), |context| {
                            context.sum_var(VarName::pwr)
                        })?
                        .get_i32()?;
                    let action_b = Self::damage(*b, *a, pwr);
                    add_actions.extend_from_slice(&[action_a, action_b]);
                    add_actions.extend(context.battle_simulation()?.slots_sync());
                    true
                }
                BattleAction::death(a) => {
                    let position = context.with_layer_r(ContextLayer::Owner(*a), |context| {
                        context.get_var(VarName::position)
                    })?;
                    add_actions.extend(BattleSimulation::die(context, *a)?);
                    add_actions.push(BattleAction::vfx(
                        HashMap::from_iter([(VarName::position, position)]),
                        "death_vfx".into(),
                    ));
                    true
                }
                BattleAction::damage(a, b, x) => {
                    let owner_pos = context.with_layer_r(ContextLayer::Owner(*a), |context| {
                        context.get_var(VarName::position)
                    })?;
                    let target_pos = context.with_layer_r(ContextLayer::Owner(*b), |context| {
                        context.get_var(VarName::position)
                    })?;
                    let curve = animations().get("range_effect_vfx").unwrap();
                    context.with_layers_r(
                        [
                            ContextLayer::Var(VarName::position, owner_pos),
                            ContextLayer::Var(VarName::extra_position, target_pos.clone()),
                        ]
                        .into(),
                        |context| curve.apply(context),
                    )?;
                    let x = Event::OutgoingDamage(a.to_bits(), b.to_bits())
                        .update_value(context, (*x).into(), *a)
                        .get_i32()?
                        .at_least(0);
                    if x > 0 {
                        let pain = animations().get("pain_vfx").unwrap();
                        context.with_layer_r(
                            ContextLayer::Var(VarName::position, target_pos.clone()),
                            |context| pain.apply(context),
                        )?;
                        let dmg = context.component::<NFusion>(*b)?.dmg + x;
                        add_actions.push(Self::var_set(*b, VarName::dmg, dmg.into()));
                    }
                    let text = animations().get("text").unwrap();
                    context.with_layers_r(
                        [
                            ContextLayer::Var(VarName::text, (-x).to_string().into()),
                            ContextLayer::Var(VarName::color, RED.into()),
                            ContextLayer::Var(VarName::position, target_pos),
                        ]
                        .into(),
                        |context| text.apply(context),
                    )?;
                    *context.t_mut()? += ANIMATION;
                    true
                }
                BattleAction::heal(a, b, x) => {
                    let owner_pos = context.with_layer_r(ContextLayer::Owner(*a), |context| {
                        context.get_var(VarName::position)
                    })?;
                    let target_pos = context.with_layer_r(ContextLayer::Owner(*b), |context| {
                        context.get_var(VarName::position)
                    })?;
                    let curve = animations().get("range_effect_vfx").unwrap();
                    context.with_layers_r(
                        [
                            ContextLayer::Var(VarName::position, owner_pos),
                            ContextLayer::Var(VarName::extra_position, target_pos.clone()),
                        ]
                        .into(),
                        |context| curve.apply(context),
                    )?;
                    if *x > 0 {
                        let pleasure = animations().get("pleasure_vfx").unwrap();
                        context.with_layer_r(
                            ContextLayer::Var(VarName::position, target_pos.clone()),
                            |context| pleasure.apply(context),
                        )?;
                        let dmg = (context.component::<NFusion>(*b)?.dmg - x).at_least(0);
                        add_actions.push(Self::var_set(*b, VarName::dmg, dmg.into()));
                        let text = animations().get("text").unwrap();
                        context.with_layers_r(
                            [
                                ContextLayer::Var(VarName::text, format!("+{x}").into()),
                                ContextLayer::Var(VarName::color, GREEN.into()),
                                ContextLayer::Var(VarName::position, target_pos),
                            ]
                            .into(),
                            |context| text.apply(context),
                        )?;
                    }
                    *context.t_mut()? += ANIMATION;
                    true
                }
                BattleAction::var_set(entity, var, value) => {
                    let t = context.t()?;
                    let mut ns = context.component_mut::<NodeState>(*entity)?;
                    if ns.insert(t, 0.1, *var, value.clone()) {
                        ns.kind.set_var(context, *entity, *var, value.clone());
                        true
                    } else {
                        false
                    }
                }
                BattleAction::spawn(entity) => {
                    NodeStatePlugin::init_entity_vars(context, *entity).log();
                    add_actions.extend_from_slice(&[BattleAction::var_set(
                        *entity,
                        VarName::visible,
                        true.into(),
                    )]);
                    true
                }
                BattleAction::apply_status(target, status, charges, color) => {
                    BattleSimulation::apply_status(
                        context,
                        *target,
                        status.clone(),
                        *charges,
                        *color,
                    )
                    .log();
                    *context.t_mut()? += ANIMATION;
                    true
                }
                BattleAction::wait(t) => {
                    *context.t_mut()? += *t;
                    false
                }
                BattleAction::vfx(vars, vfx) => {
                    if let Some(vfx) = animations().get(vfx) {
                        context.with_layers_r(
                            vars.iter()
                                .map(|(var, value)| ContextLayer::Var(*var, value.clone()))
                                .collect(),
                            |context| vfx.apply(context),
                        )?
                    }
                    false
                }
                BattleAction::send_event(event) => {
                    add_actions.extend(BattleSimulation::send_event(context, *event)?);
                    true
                }
            };
            let t = context.t()?;
            context.battle_simulation_mut()?.duration = t;
            Ok(applied)
        });
        match result {
            Ok(applied) => {
                if applied {
                    info!("{} {self}", "+".green().dimmed());
                    sim.log.actions.push(self.clone());
                } else {
                    info!("{} {self}", "-".dimmed());
                }
            }
            Err(e) => {
                error!("BattleAction apply error: {}", e.source);
                if let Some(mut bt) = e.bt {
                    bt.resolve();
                    error!("{bt:?}");
                } else {
                }
            }
        }
        add_actions
    }
}

impl BattleSimulation {
    pub fn new(battle: Battle) -> Self {
        let mut world = World::new();
        let team_left = world.spawn_empty().id();
        let team_right = world.spawn_empty().id();
        world
            .with_context_mut(|ctx| {
                battle.left.spawn(ctx, team_left)?;
                battle.right.spawn(ctx, team_right)
            })
            .log();
        fn entities_by_slot(parent: Entity, world: &World) -> Vec<Entity> {
            world
                .with_context(|ctx| {
                    Ok(ctx
                        .load_collect_children_recursive::<NFusion>(ctx.id(parent)?)?
                        .into_iter()
                        .sorted_by_key(|s| s.index)
                        .filter_map(|n| {
                            if ctx.load_first_parent_recursive::<NUnit>(n.id).is_ok() {
                                n.id.entity(ctx).ok()
                            } else {
                                None
                            }
                        })
                        .collect_vec())
                })
                .unwrap()
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
            seed: battle.id,
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

        match Context::from_battle_simulation_r(&mut self, |context| {
            Self::send_event(context, Event::BattleStart)
        }) {
            Ok(a) => {
                self.process_actions(a);
                let a = self.death_check();
                self.process_actions(a);
            }
            Err(e) => error!("BattleStart event error: {e}"),
        };
        self
    }
    pub fn run(&mut self) {
        if self.ended() {
            return;
        }
        let entities = self
            .fusions_left
            .iter()
            .chain(self.fusions_right.iter())
            .copied()
            .collect_vec();
        Context::from_battle_simulation_r(self, |context| {
            for entity in entities {
                let vars = context
                    .component::<NodeState>(entity)?
                    .kind
                    .get_vars(context, entity);
                for (var, value) in vars {
                    let value = Event::UpdateStat(var).update_value(context, value, entity);
                    let t = context.t()?;
                    context
                        .component_mut::<NodeState>(entity)?
                        .insert(t, 0.0, var, value);
                }
            }
            Ok(())
        })
        .log();
        let a = BattleAction::strike(self.fusions_left[0], self.fusions_right[0]);
        self.process_actions([a]);
        let a = self.death_check();
        self.process_actions(a);
        self.process_actions(self.slots_sync());
        match Context::from_battle_simulation_r(self, |context| {
            Self::send_event(context, Event::TurnEnd)
        }) {
            Ok(a) => self.process_actions(a),
            Err(e) => error!("TurnEnd event error: {e}"),
        };
    }
    pub fn ended(&self) -> bool {
        self.fusions_left.is_empty() || self.fusions_right.is_empty()
    }
    #[must_use]
    fn send_event(
        ctx: &mut ClientContext,
        event: Event,
    ) -> Result<VecDeque<BattleAction>, NodeError> {
        info!("{} {event}", "event:".dimmed().blue());
        let mut battle_actions: VecDeque<BattleAction> = default();
        for entity in ctx.battle_simulation()?.all_fusions() {
            ctx.with_owner(entity, |context| {
                match context.load::<NFusion>(entity)?.react(&event, context) {
                    Ok(a) => battle_actions.extend(a),
                    Err(e) => error!("NFusion event {event} failed: {e}"),
                };
                Ok(())
            })
            .log();
        }
        for (r, s) in ctx
            .world_mut()?
            .query::<(&NStatusBehavior, &NStatusMagic)>()
            .iter(ctx.world()?)
        {
            ctx.with_owner(s.id, |ctx| {
                if let Some(actions) = r.reactions.react(&event, &ctx) {
                    match actions.process(ctx) {
                        Ok(a) => battle_actions.extend(a),
                        Err(e) => {
                            error!("StatusMagic {} event {event} failed: {e}", s.status_name)
                        }
                    };
                }
                Ok(())
            })?;
        }
        Ok(battle_actions)
    }
    fn apply_status(
        context: &mut ClientContext,
        target: Entity,
        status: NStatusMagic,
        charges: i32,
        color: Color32,
    ) -> NodeResult<()> {
        let t = context.t()?;
        for child in context.get_children_of_kind(context.id(target)?, NodeKind::NStatusMagic)? {
            if let Ok(child_status) = context.load::<NStatusMagic>(child) {
                if child_status.status_name == status.status_name {
                    let mut state = context.load_mut::<NodeState>(child)?;
                    let charges = state
                        .get(VarName::charges)
                        .map(|v| v.get_i32().unwrap())
                        .to_e_not_found()?
                        + charges;
                    state.insert(t, 0.0, VarName::charges, charges.into());
                    return Ok(());
                }
            }
        }
        let entity = context.world_mut()?.spawn_empty().id();
        status.spawn(context, entity)?;
        let rep_entity = context.world_mut()?.spawn_empty().id();
        status_rep().clone().spawn(context, rep_entity)?;
        context.add_link_entities(entity, rep_entity)?;
        context.add_link_entities(target, entity)?;

        let mut state = context.load_entity_mut::<NodeState>(entity)?;
        state.insert(0.0, 0.0, VarName::visible, false.into());
        state.insert(t, 0.0, VarName::visible, true.into());
        state.insert(t, 0.0, VarName::charges, charges.into());
        state.insert(t, 0.0, VarName::color, color.into());
        Ok(())
    }
    fn process_actions(&mut self, actions: impl Into<VecDeque<BattleAction>>) {
        let mut actions: VecDeque<BattleAction> = actions.into();
        while let Some(a) = actions.pop_front() {
            for a in a.apply(self) {
                actions.push_front(a);
            }
        }
    }
    #[must_use]
    fn death_check(&mut self) -> VecDeque<BattleAction> {
        let mut actions: VecDeque<BattleAction> = default();
        Context::from_battle_simulation_r(self, |context| {
            for entity in context.battle_simulation()?.all_fusions() {
                let dmg = context.component::<NFusion>(entity)?.dmg;
                context.with_owner(entity, |context| {
                    if context.sum_var(VarName::hp)?.get_i32()? <= dmg {
                        actions.push_back(BattleAction::send_event(Event::Death(entity.to_bits())));
                        actions.push_back(BattleAction::death(entity));
                    }
                    Ok(())
                })?;
            }
            Ok(())
        })
        .log();
        actions
    }
    #[must_use]
    fn die(context: &mut ClientContext, entity: Entity) -> Result<Vec<BattleAction>, NodeError> {
        context.world_mut()?.entity_mut(entity).insert(Corpse);
        let mut died = false;
        let bs = context.battle_simulation_mut()?;
        if let Some(p) = bs.fusions_left.iter().position(|u| *u == entity) {
            bs.fusions_left.remove(p);
            died = true;
        }
        if let Some(p) = bs.fusions_right.iter().position(|u| *u == entity) {
            bs.fusions_right.remove(p);
            died = true;
        }
        if died {
            if bs.ended() {
                bs.duration += 3.0;
            }

            let mut actions = [BattleAction::var_set(
                entity,
                VarName::visible,
                false.into(),
            )]
            .to_vec();
            for child in context.children_entity(entity)? {
                actions.push(BattleAction::var_set(child, VarName::visible, false.into()));
            }
            actions.push(BattleAction::wait(ANIMATION));
            Ok(actions)
        } else {
            Ok(default())
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
            actions.push_back(BattleAction::var_set(*e, VarName::slot, i.into()));
            actions.push_back(BattleAction::var_set(*e, VarName::side, side.into()));
            let position = vec2((i + 1) as f32 * if side { -1.0 } else { 1.0 } * 2.0, 0.0);
            actions.push_back(BattleAction::var_set(
                *e,
                VarName::position,
                position.into(),
            ));
        }
        actions.push_back(BattleAction::wait(ANIMATION * 3.0));
        actions
    }

    pub fn left_units(&self) -> &Vec<Entity> {
        &self.fusions_left
    }
    pub fn right_units(&self) -> &Vec<Entity> {
        &self.fusions_right
    }
    pub fn all_fusions(&self) -> Vec<Entity> {
        let mut units = self.fusions_left.clone();
        units.append(&mut self.fusions_right.clone());
        units
    }
    pub fn all_allies(&self, entity: Entity) -> Result<&Vec<Entity>, NodeError> {
        let left = self.left_units();
        if left.contains(&entity) {
            return Ok(left);
        } else {
            let right = self.right_units();
            if right.contains(&entity) {
                return Ok(right);
            }
        }
        Err(NodeError::Custom(format!(
            "Failed to find allies: {entity} is not in any team"
        )))
    }
    pub fn all_enemies(&self, entity: Entity) -> Result<&Vec<Entity>, NodeError> {
        let left = self.left_units();
        let right = self.right_units();
        if left.contains(&entity) {
            return Ok(right);
        } else if right.contains(&entity) {
            return Ok(left);
        }
        Err(NodeError::Custom(format!(
            "Failed to find enemies: {entity} is not in any team"
        )))
    }
    pub fn offset_unit(&self, entity: Entity, offset: i32) -> Option<Entity> {
        let allies = self.all_allies(entity).ok()?;
        let pos = allies.iter().position(|e| *e == entity)?;
        allies.into_iter().enumerate().find_map(|(i, e)| {
            if i as i32 - pos as i32 == offset {
                Some(*e)
            } else {
                None
            }
        })
    }
}

impl Default for BattleSimulation {
    fn default() -> Self {
        Self {
            duration: default(),
            world: default(),
            fusions_left: default(),
            fusions_right: default(),
            team_left: Entity::PLACEHOLDER,
            team_right: Entity::PLACEHOLDER,
            log: default(),
            seed: 0,
        }
    }
}
