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
use crate::resources::context::{NodesLinkResource, NodesMapResource};

#[derive(Clone, Debug, Default)]
pub struct Battle {
    pub id: u64,
    pub left: NTeam,
    pub right: NTeam,
}

impl Battle {
    pub fn to_source(self) -> Sources<'static> {
        let mut world = World::new();
        world.init_resource::<NodesMapResource>();
        world.init_resource::<NodesLinkResource>();

        let team_left = self.left.id;
        let team_right = self.right.id;
        let left_entity = world.spawn_empty().id();
        let right_entity = world.spawn_empty().id();

        let rng = rng_seeded(self.id);

        let simulation = BattleSimulation {
            world,
            battle: self.clone(),
            fusions_left: Vec::new(),
            fusions_right: Vec::new(),
            team_left,
            team_right,
            duration: 0.0,
            log: BattleLog::default(),
            rng,
        };

        // Initialize the battle simulation
        let mut source = Sources::Battle(simulation, 0.0);
        source
            .exec_context(|ctx| {
                let battle = ctx.battle()?.battle.clone();
                battle.left.spawn(ctx, Some(left_entity)).track()?;
                battle.right.spawn(ctx, Some(right_entity)).track()?;
                Ok(())
            })
            .log();

        let (fusions_left, fusions_right) = source.exec_context_ref(|ctx| {
            let left_ids = ids_by_slot_from_context(left_entity, ctx);
            let right_ids = ids_by_slot_from_context(right_entity, ctx);
            (left_ids, right_ids)
        });

        source
            .exec_context(|ctx| {
                let sim = ctx.battle_mut()?;
                sim.fusions_left = fusions_left;
                sim.fusions_right = fusions_right;
                Ok(())
            })
            .log();

        source
    }
}

fn ids_by_slot_from_context(parent: Entity, ctx: &ClientContext) -> Vec<u64> {
    let ids = parent.ids(ctx).unwrap_or_default();
    let id = ids.into_iter().next().unwrap_or(0);
    ctx.load_children_ref::<NFusion>(id)
        .unwrap_or_default()
        .into_iter()
        .sorted_by_key(|s| s.index)
        .filter_map(|n| {
            if ctx.first_child_recursive(n.id, NodeKind::NUnit).is_ok() {
                Some(n.id)
            } else {
                None
            }
        })
        .collect_vec()
}
#[derive(Debug)]
pub struct BattleSimulation {
    pub world: World,
    pub battle: Battle,
    pub duration: f32,
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
    pub fn apply(&self, ctx: &mut ClientContext) -> Vec<Self> {
        let mut add_actions = Vec::default();
        let result: NodeResult<bool> = (|| {
            let applied = match self {
                BattleAction::strike(a, b) => {
                    add_actions.push(
                        Self::new_vfx("strike")
                            .with_owner(*a)
                            .with_target(*b)
                            .into(),
                    );
                    let fusion_a = ctx.load::<NFusion>(*a)?;
                    let fusion_b = ctx.load::<NFusion>(*b)?;
                    add_actions.push(Self::damage(*a, *b, fusion_a.pwr_ctx_get(ctx)));
                    add_actions.push(Self::damage(*b, *a, fusion_b.pwr_ctx_get(ctx)));
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
                        let dmg = ctx.load::<NFusion>(*b)?.dmg_ctx_get(ctx) + x;
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
                        let dmg = ctx.load::<NFusion>(*b)?.dmg - *x;
                        add_actions.push(Self::var_set(*b, VarName::dmg, dmg.at_least(0).into()));
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
                                |ctx| {
                                    ctx.debug_layers();
                                    text.apply(ctx)
                                },
                            )?;
                        }
                    }
                    ctx.battle_mut()?.duration += ANIMATION;
                    true
                }
                BattleAction::var_set(id, var, value) => {
                    let old_value = ctx.source().get_var(*id, *var).unwrap_or_default();
                    if *var == VarName::position {
                        // dbg!(id, var, value, &old_value);
                    }
                    if old_value.eq(value) {
                        false
                    } else {
                        ctx.source_mut().set_var(*id, *var, value.clone())?;
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
                    ctx.battle_mut()?.duration += ANIMATION;
                    true
                }
                BattleAction::wait(t) => {
                    ctx.battle_mut()?.duration += *t;
                    false
                }
                BattleAction::vfx(layers, vfx) => {
                    if let Some(vfx) = animations().get(vfx) {
                        ctx.with_layers(layers.clone(), |context| vfx.apply(context))?;
                        true
                    } else {
                        false
                    }
                }
                BattleAction::send_event(event) => {
                    add_actions.extend(BattleSimulation::send_event(ctx, *event)?);
                    true
                }
            };
            Ok(applied)
        })();
        match result {
            Ok(applied) => {
                if applied {
                    info!("{} {self}", "+".green().dimmed());
                    if let Ok(sim) = ctx.battle_mut() {
                        sim.log.actions.push(self.clone());
                    }
                } else {
                    // info!("{} {self}", "-".dimmed());
                }
            }
            Err(e) => {
                error!("{} {self}: {e}", "x".red().dimmed());
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

impl Default for BattleSimulation {
    fn default() -> Self {
        Self {
            world: World::new(),
            battle: Battle {
                id: 0,
                left: NTeam::default(),
                right: NTeam::default(),
            },
            duration: 0.0,
            fusions_left: Vec::new(),
            fusions_right: Vec::new(),
            team_left: 0,
            team_right: 0,
            log: BattleLog::default(),
            rng: ChaCha8Rng::seed_from_u64(0),
        }
    }
}

impl BattleSimulation {
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
        let left = &self.fusions_left;
        if left.contains(&id) {
            return Ok(left);
        } else {
            let right = &self.fusions_right;
            if right.contains(&id) {
                return Ok(right);
            }
        }
        Err(NodeError::custom(format!(
            "Failed to find allies: {id} is not in any team"
        )))
    }

    pub fn all_enemies(&self, id: u64) -> Result<&Vec<u64>, NodeError> {
        let left = &self.fusions_left;
        let right = &self.fusions_right;
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

    pub fn ended(&self) -> bool {
        self.fusions_left.is_empty() || self.fusions_right.is_empty()
    }
    pub fn start(ctx: &mut ClientContext) -> NodeResult<()> {
        let spawn_actions = {
            let sim = ctx.battle()?;
            sim.fusions_left
                .iter()
                .zip_longest(sim.fusions_right.iter())
                .flat_map(|e| match e {
                    EitherOrBoth::Both(a, b) => {
                        vec![BattleAction::spawn(*a), BattleAction::spawn(*b)]
                    }
                    EitherOrBoth::Left(e) | EitherOrBoth::Right(e) => {
                        vec![BattleAction::spawn(*e)]
                    }
                })
                .collect_vec()
        };

        process_actions(ctx, spawn_actions);

        let actions = ctx.battle()?.slots_sync();
        process_actions(ctx, actions);

        match BattleSimulation::send_event(ctx, Event::BattleStart) {
            Ok(a) => {
                process_actions(ctx, a);
                let a = BattleSimulation::death_check(ctx)?;
                process_actions(ctx, a);
            }
            Err(e) => error!("BattleStart event error: {e}"),
        }

        Ok(())
    }
    pub fn run(ctx: &mut ClientContext) -> NodeResult<()> {
        if ctx.battle()?.ended() {
            return Ok(());
        }

        let sim = ctx.battle()?;
        let ids = sim
            .fusions_left
            .iter()
            .chain(sim.fusions_right.iter())
            .copied()
            .collect_vec();
        for id in ids {
            let vars = node_kind_match!(ctx.get_kind(id)?, ctx.load::<NodeType>(id)?.get_vars());
            for (var, value) in vars {
                let value = Event::UpdateStat(var).update_value(ctx, value, id);
                let t = ctx.t().to_not_found()?;
                let entity = id.entity(ctx)?;
                let mut state = NodeStateHistory::load_mut(entity, ctx)?;
                state.insert(t, 0.0, var, value);
            }
        }

        ctx.battle_mut()?.duration += TURN;
        let sim = ctx.battle()?;
        if !sim.fusions_left.is_empty() && !sim.fusions_right.is_empty() {
            let a = BattleAction::strike(sim.fusions_left[0], sim.fusions_right[0]);
            process_actions(ctx, vec![a]);
        }
        let a = BattleSimulation::death_check(ctx)?;
        process_actions(ctx, a);
        let sync_actions = ctx.battle()?.slots_sync();
        process_actions(ctx, sync_actions);
        match BattleSimulation::send_event(ctx, Event::TurnEnd) {
            Ok(a) => process_actions(ctx, a),
            Err(e) => error!("TurnEnd event error: {e}"),
        }

        Ok(())
    }
    pub fn send_event(
        ctx: &mut ClientContext,
        event: Event,
    ) -> Result<VecDeque<BattleAction>, NodeError> {
        info!("{} {event}", "event:".dimmed().blue());
        let mut battle_actions: VecDeque<BattleAction> = default();
        let mut fusion_statuses: Vec<(u64, u64)> = default();
        let sim = ctx.battle()?;
        for id in sim.all_fusions() {
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
    pub fn apply_status(
        ctx: &mut ClientContext,
        target: u64,
        status: NStatusMagic,
        color: Color32,
    ) -> NodeResult<()> {
        let t = ctx.t().to_not_found()?;
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
        let new_status_id = new_status.id();
        new_status.spawn(ctx, Some(entity))?;
        // Status is already saved with its charges during spawn

        ctx.add_link(target, new_status_id)?;

        let mut state = NodeStateHistory::load_mut(entity, ctx)?;
        state.insert(0.0, 0.0, VarName::visible, false.into());
        state.insert(t, 0.0, VarName::visible, true.into());
        state.insert(t, 0.0, VarName::color, color.into());
        Ok(())
    }

    pub fn die(ctx: &mut ClientContext, id: u64) -> NodeResult<Vec<BattleAction>> {
        let entity = id.entity(ctx)?;
        ctx.world_mut()?.entity_mut(entity).insert(Corpse);
        let mut died = false;
        let sim = ctx.battle_mut()?;
        if let Some(p) = sim.fusions_left.iter().position(|u| *u == id) {
            sim.fusions_left.remove(p);
            died = true;
        }
        if let Some(p) = sim.fusions_right.iter().position(|u| *u == id) {
            sim.fusions_right.remove(p);
            died = true;
        }
        if died {
            if sim.ended() {
                sim.duration += 1.0;
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
}

fn process_actions(ctx: &mut ClientContext, actions: impl Into<VecDeque<BattleAction>>) {
    let mut actions: VecDeque<BattleAction> = actions.into();
    while let Some(a) = actions.pop_front() {
        actions.extend(a.apply(ctx));
    }
}

impl BattleSimulation {
    pub fn death_check(ctx: &mut ClientContext) -> NodeResult<VecDeque<BattleAction>> {
        let mut actions: VecDeque<BattleAction> = default();
        *ctx.t_mut().unwrap() = ctx.battle()?.duration;
        let sim = ctx.battle()?;
        for id in sim.all_fusions() {
            let fusion = ctx.load::<NFusion>(id)?;
            let hp = fusion.hp_ctx_get(ctx);
            let dmg = fusion.dmg_ctx_get(ctx);
            if hp <= dmg {
                actions.push_back(BattleAction::send_event(Event::Death(id)));
                actions.push_back(BattleAction::death(id));
            }
        }
        Ok(actions)
    }
    pub fn slots_sync(&self) -> VecDeque<BattleAction> {
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
}
