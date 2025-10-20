//! # Battle System with New Node Operation Pattern
//!
//! This file demonstrates the new node operation pattern:
//!
//! ## Reading Nodes
//! - Load nodes from context by reference using `ctx.load::<NodeType>(id)?`
//! - Access inner nodes with generated `*_ref()` methods that can be chained with `?`
//! - For var fields, use `NodeStateHistory::find_var(ctx, var, entity)` or `ctx.get_var(var)`
//!
//! ## Editing Nodes
//! 1. Get owned node value (can clone a loaded reference): `let mut node = ctx.load::<NodeType>(id)?.clone()`
//! 2. Edit fields with generated `set_*()` methods for data fields or `set_var(var, value)` for var fields
//! 3. Save the node with `node.save(ctx)?` - this calls `set_dirty()` and updates NodeStateHistory
//!
//! ## Var Fields
//! - All var field operations go through context and NodeStateHistory for battle simulation variants
//! - Saving updates history for var fields automatically
//! - Use `NodeStateHistory::get_at(t, var)` for time-based var retrieval in simulations

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
    pub fusions_left: Vec<u64>,
    pub fusions_right: Vec<u64>,
    pub team_left: u64,
    pub team_right: u64,
    pub log: BattleLog,
    pub rng: ChaCha8Rng,
}
#[derive(Default, Debug, Clone)]
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
    apply_status(u64, NStatusMagic, Color32),
    send_event(Event),
    vfx(Vec<ContextLayer>, String),
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
            BattleAction::apply_status(a, status, _) => {
                a.hash(state);
                status.id.hash(state);
                // Note: stacks are in state component, using status_name for hash consistency
                status.status_name.hash(state);
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
            BattleAction::apply_status(a, status, color) => {
                format!(
                    "+[{} {}]>{a}({})",
                    color.to_hex(),
                    status.status_name,
                    status.state().unwrap().stacks
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
        let result = sim.with_context_mut(sim.duration, |ctx| {
            let applied = match self {
                BattleAction::strike(a, b) => {
                    let owner_pos = ctx.with_owner(*a, |ctx| ctx.get_var(VarName::position))?;
                    let target_pos = ctx.with_owner(*b, |ctx| ctx.get_var(VarName::position))?;
                    add_actions.push(
                        Self::new_vfx("strike")
                            .with_var(VarName::position, owner_pos)
                            .with_var(VarName::extra_position, target_pos.clone())
                            .into(),
                    );
                    if let Some(strike_anim) = animations().get("strike") {
                        ctx.with_layers(
                            [
                                ContextLayer::Owner(*a),
                                ContextLayer::Target(*b),
                                ContextLayer::Var(VarName::position, vec2(0.0, 0.0).into()),
                            ],
                            |context| strike_anim.apply(context),
                        )?;
                    }
                    let fusion_a = ctx.load::<NFusion>(*a)?;
                    let fusion_b = ctx.load::<NFusion>(*b)?;
                    add_actions.push(Self::damage(*a, *b, fusion_a.pwr));
                    add_actions.push(Self::damage(*b, *a, fusion_b.pwr));
                    add_actions.extend(ctx.battle()?.slots_sync());
                    true
                }
                BattleAction::death(a) => {
                    let position =
                        ctx.with_owner(*a, |context| context.get_var(VarName::position))?;
                    add_actions.extend(BattleSimulation::die(ctx, *a)?);
                    add_actions.push(
                        Self::new_vfx("death_vfx")
                            .with_var(VarName::position, position)
                            .into(),
                    );
                    true
                }
                BattleAction::damage(a, b, x) => {
                    let owner_pos = ctx.with_owner(*a, |ctx| ctx.get_var(VarName::position))?;
                    let target_pos = ctx.with_owner(*b, |ctx| ctx.get_var(VarName::position))?;
                    add_actions.push(
                        Self::new_vfx("range_effect_vfx")
                            .with_var(VarName::position, owner_pos)
                            .with_var(VarName::extra_position, target_pos.clone())
                            .into(),
                    );
                    let x = Event::OutgoingDamage(*a, *b)
                        .update_value(ctx, (*x).into(), *a)
                        .get_i32()?
                        .at_least(0);
                    let x = Event::IncomingDamage(*a, *b)
                        .update_value(ctx, x.into(), *b)
                        .get_i32()?
                        .at_least(0);
                    if x > 0 {
                        let dmg = ctx.load::<NFusion>(*b)?.dmg + x;
                        add_actions.push(Self::var_set(*b, VarName::dmg, dmg.into()));
                        add_actions.push(
                            Self::new_vfx("pain_vfx")
                                .with_var(VarName::position, target_pos.clone())
                                .into(),
                        );
                    }
                    add_actions.push(
                        Self::new_vfx("text")
                            .with_var(VarName::position, target_pos)
                            .with_var(VarName::text, (-x).to_string())
                            .with_var(VarName::color, HexColor::from("#FF0000".to_string()))
                            .into(),
                    );
                    // *ctx.t_mut()? += ANIMATION;
                    true
                }
                BattleAction::heal(a, b, x) => {
                    let owner_pos = ctx.with_owner(*a, |ctx| ctx.get_var(VarName::position))?;
                    let target_pos = ctx.with_owner(*b, |ctx| ctx.get_var(VarName::position))?;
                    if let Some(curve) = animations().get("range_effect_vfx") {
                        ctx.with_layers(
                            [
                                ContextLayer::Var(VarName::position, owner_pos),
                                ContextLayer::Var(VarName::extra_position, target_pos.clone()),
                            ],
                            |context| curve.apply(context),
                        )?;
                    }
                    if *x > 0 {
                        if let Some(pleasure) = animations().get("pleasure_vfx") {
                            ctx.with_layer(
                                ContextLayer::Var(VarName::position, target_pos.clone()),
                                |context| pleasure.apply(context),
                            )?;
                        }
                        let dmg = (ctx.load::<NFusion>(*b)?.dmg - x).at_least(0);
                        add_actions.push(Self::var_set(*b, VarName::dmg, dmg.into()));
                        if let Some(text) = animations().get("text") {
                            ctx.with_layers(
                                [
                                    ContextLayer::Var(VarName::position, target_pos),
                                    ContextLayer::Var(VarName::text, format!("+{}", x).into()),
                                    ContextLayer::Var(
                                        VarName::color,
                                        HexColor::from("#00FF00".to_string()).into(),
                                    ),
                                ],
                                |context| text.apply(context),
                            )?;
                        }
                    }
                    *ctx.t_mut()? += ANIMATION;
                    true
                }
                BattleAction::var_set(id, var, value) => {
                    // Try to set var on the node first
                    let kind = ctx.get_kind(*id)?;
                    let set_on_node = match kind {
                        NodeKind::NFusion => {
                            if let Ok(mut node) = ctx.load::<NFusion>(*id).cloned() {
                                if node.set_var(*var, value.clone()).is_ok() {
                                    node.save(ctx).is_ok()
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };

                    // If node set failed or var doesn't exist on node, set directly on state
                    if !set_on_node {
                        let entity = id.entity(ctx)?;
                        let current = if let Ok(state) = NodeStateHistory::load(entity, ctx) {
                            state.get(*var)
                        } else {
                            None
                        };

                        if current.as_ref().map_or(false, |v| v.eq(value)) {
                            false
                        } else {
                            let t = ctx.t()?;
                            let mut state = NodeStateHistory::load_mut(entity, ctx)?;
                            state.insert(t, 0.0, *var, value.clone());
                            true
                        }
                    } else {
                        true
                    }
                }
                BattleAction::spawn(id) => {
                    add_actions.extend_from_slice(&[BattleAction::var_set(
                        *id,
                        VarName::visible,
                        true.into(),
                    )]);
                    true
                }
                BattleAction::apply_status(target, status, color) => {
                    BattleSimulation::apply_status(ctx, *target, status.clone(), *color).log();
                    *ctx.t_mut()? += ANIMATION;
                    true
                }
                BattleAction::wait(t) => {
                    *ctx.t_mut()? += *t;
                    false
                }
                BattleAction::vfx(layers, vfx) => {
                    if let Some(vfx) = animations().get(vfx) {
                        ctx.with_layers(layers.clone(), |context| vfx.apply(context))?
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
                    // info!("{} {self}", "-".dimmed());
                }
            }
            Err(e) => {
                error!("BattleAction apply error: {}", e);
            }
        }
        add_actions
    }
    pub fn new_vfx(name: impl ToString) -> VfxBuilder {
        VfxBuilder {
            name: name.to_string(),
            layers: Vec::new(),
        }
    }
}

pub struct VfxBuilder {
    name: String,
    layers: Vec<ContextLayer>,
}

impl VfxBuilder {
    pub fn with_owner(mut self, id: u64) -> Self {
        self.layers.push(ContextLayer::Owner(id));
        self
    }

    pub fn with_target(mut self, id: u64) -> Self {
        self.layers.push(ContextLayer::Target(id));
        self
    }

    pub fn with_caster(mut self, id: u64) -> Self {
        self.layers.push(ContextLayer::Caster(id));
        self
    }

    pub fn with_status(mut self, id: u64) -> Self {
        self.layers.push(ContextLayer::Status(id));
        self
    }

    pub fn with_var(mut self, name: VarName, value: impl Into<VarValue>) -> Self {
        self.layers.push(ContextLayer::Var(name, value.into()));
        self
    }
}

impl From<VfxBuilder> for BattleAction {
    fn from(builder: VfxBuilder) -> Self {
        BattleAction::vfx(builder.layers, builder.name)
    }
}

impl BattleSimulation {
    pub fn new(battle: Battle) -> Self {
        let mut world = World::new();
        world.init_resource::<NodeEntityMap>();
        world.init_resource::<NodeLinks>();
        dbg!(&battle.left);
        let team_left = battle.left.id;
        let team_right = battle.right.id;
        let left_entity = world.spawn_empty().id();
        let right_entitiy = world.spawn_empty().id();
        let mut bs = Self {
            world,
            fusions_left: default(),
            fusions_right: default(),
            team_left,
            team_right,
            duration: 0.0,
            log: BattleLog::default(),
            rng: rng_seeded(battle.id),
        };
        bs.with_context_mut(bs.duration, |ctx| {
            battle.left.spawn(ctx, Some(left_entity))?;
            battle.right.spawn(ctx, Some(right_entitiy))
        })
        .log();
        fn ids_by_slot(parent: Entity, world: &World) -> Vec<u64> {
            world
                .with_context(|ctx| {
                    Ok(ctx
                        .load_collect_children::<NFusion>(ctx.id(parent)?)?
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
        let fusions_left = ids_by_slot(left_entity, &bs.world);
        let fusions_right = ids_by_slot(right_entitiy, &bs.world);
        bs.fusions_left = fusions_left;
        bs.fusions_right = fusions_right;
        bs
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

        match self.with_context_mut(self.duration, |context| {
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
        let ids = self
            .fusions_left
            .iter()
            .chain(self.fusions_right.iter())
            .copied()
            .collect_vec();
        self.with_context_mut(self.duration, |ctx| {
            for id in ids {
                let vars = ctx.get_kind(id).unwrap().get_vars(ctx, id);
                for (var, value) in vars {
                    let value = Event::UpdateStat(var).update_value(ctx, value, id);
                    let t = ctx.t()?;
                    let entity = id.entity(ctx)?;
                    let mut state = NodeStateHistory::load_mut(entity, ctx)?;
                    state.insert(t, 0.0, var, value);
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
        match self.with_context_mut(self.duration, |context| {
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
        let mut fusion_statuses: Vec<(u64, u64)> = default();
        for id in ctx.battle()?.all_fusions() {
            let statuses = ctx.collect_kind_children(id, NodeKind::NStatusMagic)?;
            if !statuses.is_empty() {
                fusion_statuses.extend(statuses.into_iter().map(|s_id| (id, s_id)));
            }
            ctx.with_owner(id, |ctx| {
                match ctx.load::<NFusion>(id)?.clone().react(&event, ctx) {
                    Ok(a) => battle_actions.extend(a),
                    Err(e) => error!("NFusion event {event} failed: {e}"),
                };
                Ok(())
            })
            .log();
        }
        for (fusion_id, status_id) in fusion_statuses {
            match ctx.with_layers(
                [
                    ContextLayer::Owner(fusion_id),
                    ContextLayer::Status(status_id),
                ],
                |ctx| {
                    let behavior = ctx.load::<NStatusBehavior>(status_id)?;
                    let actions = behavior
                        .reactions
                        .react(&event, ctx)
                        .to_not_found()?
                        .clone();
                    actions.process(ctx)
                },
            ) {
                Ok(actions) => battle_actions.extend(actions),
                Err(e) => e.log(),
            }
        }
        Ok(battle_actions)
    }
    fn apply_status(
        ctx: &mut ClientContext,
        target: u64,
        status: NStatusMagic,
        color: Color32,
    ) -> NodeResult<()> {
        let t = ctx.t()?;
        for child in ctx.get_children_of_kind(target, NodeKind::NStatusMagic)? {
            if let Ok(child_status) = ctx.load::<NStatusMagic>(child) {
                if child_status.status_name == status.status_name {
                    // Update stacks on the status state component
                    if let Ok(mut state) = child_status.state_ref(ctx).cloned() {
                        let new_stacks = state.stacks + status.state_ref(ctx)?.stacks;
                        state.set_var(VarName::stacks, new_stacks.into())?;
                        state.save(ctx)?;
                        return Ok(());
                    }
                }
            }
        }
        let entity = ctx.world_mut()?.spawn_empty().id();
        let new_status = status.remap_ids();
        new_status.spawn(ctx, Some(entity))?;
        // Status is already saved with its charges during spawn

        ctx.add_link_entities(target.entity(ctx)?, entity)?;

        let mut state = NodeStateHistory::load_mut(entity, ctx)?;
        state.insert(0.0, 0.0, VarName::visible, false.into());
        state.insert(t, 0.0, VarName::visible, true.into());
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
        self.with_context_mut(self.duration, |ctx| {
            for id in ctx.battle()?.all_fusions() {
                let fusion = ctx.load::<NFusion>(id)?;
                let hp = fusion.hp_get(ctx)?;
                let dmg = fusion.dmg_get(ctx)?;
                dbg!(hp, dmg);
                if hp <= dmg {
                    actions.push_back(BattleAction::send_event(Event::Death(id)));
                    actions.push_back(BattleAction::death(id));
                }
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
        Err(NodeError::custom(format!(
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
        Err(NodeError::custom(format!(
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

    pub fn as_context(&self, t: f32) -> Context<WorldSource<'_>> {
        Context::new(WorldSource::new_battle(self, t))
    }

    pub fn as_context_mut(&mut self, t: f32) -> Context<WorldSource<'_>> {
        Context::new(WorldSource::new_battle_mut(self, t))
    }

    pub fn with_context<R, F>(&self, t: f32, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Context<WorldSource<'_>>) -> NodeResult<R>,
    {
        let source = WorldSource::new_battle(self, t);
        Context::exec(source, f)
    }

    pub fn with_context_mut<R, F>(&mut self, t: f32, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Context<WorldSource<'_>>) -> NodeResult<R>,
    {
        let source = WorldSource::new_battle_mut(self, t);
        Context::exec(source, f)
    }
}
