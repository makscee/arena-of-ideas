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
    pub t: f32,
    pub world: World,
    pub fusions_left: Vec<u64>,
    pub fusions_right: Vec<u64>,
    pub team_left: u64,
    pub team_right: u64,
    pub log: BattleLog,
    pub rng: ChaCha8Rng,
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
    var_set(u64, VarName, VarValue),
    strike(u64, u64),
    damage(u64, u64, i32),
    heal(u64, u64, i32),
    death(u64),
    spawn(u64),
    apply_status(u64, NStatusMagic, i32, Color32),
    send_event(Event),
    vfx(HashMap<VarName, VarValue>, String),
    wait(f32),
}
impl Hash for BattleAction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            BattleAction::var_set(id, var, value) => {
                id.hash(state);
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
                status.id.hash(state);
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
        let result = sim.with_context_mut(|ctx| {
            let applied = match self {
                BattleAction::strike(a, b) => {
                    let strike_anim = animations().get("strike").unwrap();
                    ctx.with_layers_temp(
                        [
                            ContextLayer::Owner(*a),
                            ContextLayer::Target(*b),
                            ContextLayer::Var(VarName::position, vec2(0.0, 0.0).into()),
                        ]
                        .into(),
                        |context| strike_anim.apply(context),
                    )?;
                    let pwr = ctx
                        .with_owner(*a, |context| context.sum_var(VarName::pwr))?
                        .get_i32()?;
                    let action_a = Self::damage(*a, *b, pwr);
                    let pwr = ctx
                        .with_owner(*b, |context| context.sum_var(VarName::pwr))?
                        .get_i32()?;
                    let action_b = Self::damage(*b, *a, pwr);
                    add_actions.extend_from_slice(&[action_a, action_b]);
                    add_actions.extend(ctx.battle()?.slots_sync());
                    true
                }
                BattleAction::death(a) => {
                    let position =
                        ctx.with_owner(*a, |context| context.get_var(VarName::position))?;
                    add_actions.extend(BattleSimulation::die(ctx, *a)?);
                    add_actions.push(BattleAction::vfx(
                        HashMap::from_iter([(VarName::position, position)]),
                        "death_vfx".into(),
                    ));
                    true
                }
                BattleAction::damage(a, b, x) => {
                    let owner_pos = ctx.with_temp_layer(ContextLayer::Owner(*a), |context| {
                        context.get_var(VarName::position)
                    })?;
                    let target_pos = ctx.with_temp_layer(ContextLayer::Owner(*b), |context| {
                        context.get_var(VarName::position)
                    })?;
                    let curve = animations().get("range_effect_vfx").unwrap();
                    ctx.with_temp_layers(
                        [
                            ContextLayer::Var(VarName::position, owner_pos),
                            ContextLayer::Var(VarName::extra_position, target_pos.clone()),
                        ]
                        .into(),
                        |context| curve.apply(context),
                    )?;
                    let x = Event::OutgoingDamage(*a, *b)
                        .update_value(ctx, (*x).into(), *a)
                        .get_i32()?
                        .at_least(0);
                    if x > 0 {
                        let pain = animations().get("pain_vfx").unwrap();
                        ctx.with_temp_layer(
                            ContextLayer::Var(VarName::position, target_pos.clone()),
                            |context| pain.apply(context),
                        )?;
                        let dmg = ctx.load::<NFusion>(*b)?.dmg + x;
                        add_actions.push(Self::var_set(*b, VarName::dmg, dmg.into()));
                    }
                    let text = animations().get("text").unwrap();
                    ctx.with_temp_layers(
                        [
                            ContextLayer::Var(VarName::text, (-x).to_string().into()),
                            ContextLayer::Var(VarName::color, RED.into()),
                            ContextLayer::Var(VarName::position, target_pos),
                        ]
                        .into(),
                        |context| text.apply(context),
                    )?;
                    *ctx.t_mut()? += ANIMATION;
                    true
                }
                BattleAction::heal(a, b, x) => {
                    let owner_pos = ctx.with_temp_layer(ContextLayer::Owner(*a), |context| {
                        context.get_var(VarName::position)
                    })?;
                    let target_pos = ctx.with_temp_layer(ContextLayer::Owner(*b), |context| {
                        context.get_var(VarName::position)
                    })?;
                    let curve = animations().get("range_effect_vfx").unwrap();
                    ctx.with_temp_layers(
                        [
                            ContextLayer::Var(VarName::position, owner_pos),
                            ContextLayer::Var(VarName::extra_position, target_pos.clone()),
                        ]
                        .into(),
                        |context| curve.apply(context),
                    )?;
                    if *x > 0 {
                        let pleasure = animations().get("pleasure_vfx").unwrap();
                        ctx.with_temp_layer(
                            ContextLayer::Var(VarName::position, target_pos.clone()),
                            |context| pleasure.apply(context),
                        )?;
                        let dmg = (ctx.load::<NFusion>(*b)?.dmg - x).at_least(0);
                        add_actions.push(Self::var_set(*b, VarName::dmg, dmg.into()));
                        let text = animations().get("text").unwrap();
                        ctx.with_temp_layers(
                            [
                                ContextLayer::Var(VarName::text, format!("+{x}").into()),
                                ContextLayer::Var(VarName::color, GREEN.into()),
                                ContextLayer::Var(VarName::position, target_pos),
                            ]
                            .into(),
                            |context| text.apply(context),
                        )?;
                    }
                    *ctx.t_mut()? += ANIMATION;
                    true
                }
                BattleAction::var_set(id, var, value) => {
                    let t = ctx.t()?;
                    let kind = ctx.get_kind(*id)?;
                    let mut ns = ctx.load_mut::<NodeState>(*id)?;
                    if ns.insert(t, 0.1, *var, value.clone()) {
                        kind.set_var(ctx, *id, *var, value.clone()).log();
                        true
                    } else {
                        false
                    }
                }
                BattleAction::spawn(id) => {
                    let kind = ctx.get_kind(*id)?;
                    let vars = kind.get_vars(ctx, *id);
                    let mut ns = NodeState::load_mut(id.entity(ctx)?, ctx)?;
                    ns.init_vars(vars.into_iter());
                    add_actions.extend_from_slice(&[BattleAction::var_set(
                        *id,
                        VarName::visible,
                        true.into(),
                    )]);
                    true
                }
                BattleAction::apply_status(target, status, charges, color) => {
                    BattleSimulation::apply_status(ctx, *target, status.clone(), *charges, *color)
                        .log();
                    *ctx.t_mut()? += ANIMATION;
                    true
                }
                BattleAction::wait(t) => {
                    *ctx.t_mut()? += *t;
                    false
                }
                BattleAction::vfx(vars, vfx) => {
                    if let Some(vfx) = animations().get(vfx) {
                        ctx.with_layers_temp(
                            vars.iter()
                                .map(|(var, value)| ContextLayer::Var(*var, value.clone()))
                                .collect(),
                            |context| vfx.apply(context),
                        )?
                    }
                    false
                }
                BattleAction::send_event(event) => {
                    add_actions.extend(BattleSimulation::send_event(ctx, *event)?);
                    true
                }
            };
            let t = ctx.t()?;
            ctx.battle_mut()?.duration = t;
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
                error!("BattleAction apply error: {}", e);
            }
        }
        add_actions
    }
}

impl BattleSimulation {
    pub fn new(battle: Battle) -> Self {
        let mut world = World::new();
        let team_left = battle.left.id;
        let team_right = battle.right.id;
        let left_entity = world.spawn_empty().id();
        let right_entitiy = world.spawn_empty().id();
        world
            .with_context_mut(|ctx| {
                battle.left.spawn(ctx, left_entity)?;
                battle.right.spawn(ctx, right_entitiy)
            })
            .log();
        fn ids_by_slot(parent: Entity, world: &World) -> Vec<u64> {
            world
                .with_context(|ctx| {
                    Ok(ctx
                        .load_collect_children_recursive::<NFusion>(ctx.id(parent)?)?
                        .into_iter()
                        .sorted_by_key(|s| s.index)
                        .filter_map(|n| {
                            if ctx.first_child_recursive(n.id, NodeKind::NUnit).is_ok() {
                                Some(n.id)
                            } else {
                                None
                            }
                        })
                        .collect_vec())
                })
                .unwrap()
        }
        let fusions_left = ids_by_slot(left_entity, &world);
        let fusions_right = ids_by_slot(right_entitiy, &world);
        Self {
            world,
            fusions_left,
            fusions_right,
            team_left,
            team_right,
            duration: 0.0,
            t: 0.0,
            log: BattleLog::default(),
            rng: rng_seeded(battle.id),
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

        match self.with_context_mut(|context| Self::send_event(context, Event::BattleStart)) {
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
        let ids = self
            .fusions_left
            .iter()
            .chain(self.fusions_right.iter())
            .copied()
            .collect_vec();
        self.with_context_mut(|ctx| {
            for id in ids {
                let vars = ctx.get_kind(id).unwrap().get_vars(ctx, id);
                for (var, value) in vars {
                    let value = Event::UpdateStat(var).update_value(ctx, value, id);
                    let t = ctx.t()?;
                    ctx.load_mut::<NodeState>(id)?.insert(t, 0.0, var, value);
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
        match self.with_context_mut(|context| Self::send_event(context, Event::TurnEnd)) {
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
        for id in ctx.battle()?.all_fusions() {
            ctx.with_owner(id, |context| {
                match context.load::<NFusion>(id)?.react(&event, context) {
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
            ctx.with_temp_owner(s.id, |ctx| {
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
        ctx: &mut ClientContext,
        target: u64,
        status: NStatusMagic,
        charges: i32,
        color: Color32,
    ) -> NodeResult<()> {
        let t = ctx.t()?;
        for child in ctx.get_children_of_kind(target, NodeKind::NStatusMagic)? {
            if let Ok(child_status) = ctx.load::<NStatusMagic>(child) {
                if child_status.status_name == status.status_name {
                    let mut state = ctx.load_mut::<NodeState>(child)?;
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
        let entity = ctx.world_mut()?.spawn_empty().id();
        status.spawn(ctx, entity)?;
        let rep_entity = ctx.world_mut()?.spawn_empty().id();
        status_rep().clone().spawn(ctx, rep_entity)?;
        ctx.add_link_entities(entity, rep_entity)?;
        ctx.add_link_entities(target.entity(ctx)?, entity)?;

        let mut state = ctx.load_entity_mut::<NodeState>(entity)?;
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
        self.with_context_mut(|context| {
            for id in context.battle()?.all_fusions() {
                let dmg = context.load::<NFusion>(id)?.dmg;
                context.with_owner(id, |context| {
                    if context.sum_var(VarName::hp)?.get_i32()? <= dmg {
                        actions.push_back(BattleAction::send_event(Event::Death(id)));
                        actions.push_back(BattleAction::death(id));
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
    fn die(ctx: &mut ClientContext, id: u64) -> NodeResult<Vec<BattleAction>> {
        let entity = id.entity(ctx)?;
        ctx.world_mut()?.entity_mut(entity).insert(Corpse);
        let mut died = false;
        let bs = ctx.battle_mut()?;
        if let Some(p) = bs.fusions_left.iter().position(|u| *u == id) {
            bs.fusions_left.remove(p);
            died = true;
        }
        if let Some(p) = bs.fusions_right.iter().position(|u| *u == id) {
            bs.fusions_right.remove(p);
            died = true;
        }
        if died {
            if bs.ended() {
                bs.duration += 3.0;
            }

            let mut actions = [BattleAction::var_set(id, VarName::visible, false.into())].to_vec();
            for child in ctx.children_recursive(id)? {
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

    pub fn left_units(&self) -> &Vec<u64> {
        &self.fusions_left
    }
    pub fn right_units(&self) -> &Vec<u64> {
        &self.fusions_right
    }
    pub fn all_fusions(&self) -> Vec<u64> {
        let mut units = self.fusions_left.clone();
        units.append(&mut self.fusions_right.clone());
        units
    }
    pub fn all_allies(&self, id: u64) -> NodeResult<&Vec<u64>> {
        let left = self.left_units();
        if left.contains(&id) {
            return Ok(left);
        } else {
            let right = self.right_units();
            if right.contains(&id) {
                return Ok(right);
            }
        }
        Err(NodeError::Custom(format!(
            "Failed to find allies: {id} is not in any team"
        )))
    }
    pub fn all_enemies(&self, id: u64) -> Result<&Vec<u64>, NodeError> {
        let left = self.left_units();
        let right = self.right_units();
        if left.contains(&id) {
            return Ok(right);
        } else if right.contains(&id) {
            return Ok(left);
        }
        Err(NodeError::Custom(format!(
            "Failed to find enemies: {id} is not in any team"
        )))
    }
    pub fn offset_unit(&self, unit_id: u64, offset: i32) -> Option<u64> {
        let allies = self.all_allies(unit_id).ok()?;
        let pos = allies.iter().position(|id| *id == unit_id)?;
        allies.into_iter().enumerate().find_map(|(i, id)| {
            if i as i32 - pos as i32 == offset {
                Some(*id)
            } else {
                None
            }
        })
    }
}
