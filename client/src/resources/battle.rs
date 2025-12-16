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

use rand_chacha::rand_core::SeedableRng;

use super::*;
use crate::resources::context::{NodesLinkResource, NodesMapResource};

fn find_ability_by_path(path: &str, ctx: &ClientContext) -> NodeResult<(NHouse, NAbilityMagic)> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() != 2 {
        return Err(NodeError::custom(format!(
            "Ability path must be in format 'House/Ability', got {path}",
        )));
    }

    let house_name = parts[0];
    let ability_name = parts[1];

    let team = ctx.first_parent_recursive(ctx.owner()?, NodeKind::NTeam)?;
    let houses = ctx.collect_kind_children_recursive(team, NodeKind::NHouse)?;

    for house_id in houses {
        let house = ctx.load_ref::<NHouse>(house_id)?;
        if house.house_name == house_name {
            if let Ok(mut ability) = house.ability.load_node(ctx) {
                if ability.ability_name == ability_name {
                    ability.load_all(ctx)?;
                    return Ok((house.clone(), ability));
                }
            }
        }
    }

    Err(NodeError::not_found_generic(path.to_owned()))
}

fn find_status_by_path(path: &str, ctx: &ClientContext) -> NodeResult<(NHouse, NStatusMagic)> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() != 2 {
        return Err(NodeError::custom(
            "Status path must be in format 'House/Status'",
        ));
    }

    let house_name = parts[0];
    let status_name = parts[1];

    let team = ctx.first_parent_recursive(ctx.owner()?, NodeKind::NTeam)?;
    let houses = ctx.collect_kind_children_recursive(team, NodeKind::NHouse)?;

    for house_id in houses {
        let house = ctx.load_ref::<NHouse>(house_id)?;
        if house.house_name == house_name {
            if let Ok(mut status) = house.status.load_node(ctx) {
                if status.status_name == status_name {
                    status.load_all(ctx)?;
                    return Ok((house.clone(), status));
                }
            }
        }
    }

    Err(NodeError::not_found_generic(path.to_owned()))
}

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
            team_left,
            team_right,
            rng,
            ..default()
        };

        // Initialize the battle simulation
        let mut source = Sources::Battle(simulation, 0.0);
        source
            .exec_context(|ctx| {
                let mut battle = ctx.battle()?.battle.clone();
                battle.left.spawn(ctx, Some(left_entity)).track()?;
                if battle.right.id == 0 {
                    battle.right = battle.right.remap_ids();
                }
                battle.right.spawn(ctx, Some(right_entity)).track()?;
                Ok(())
            })
            .log();

        let (units_left, units_right) = source.exec_context_ref(|ctx| {
            let left_ids = ids_by_slot_from_context(left_entity, ctx);
            let right_ids = ids_by_slot_from_context(right_entity, ctx);
            (left_ids, right_ids)
        });

        source
            .exec_context(|ctx| {
                let sim = ctx.battle_mut()?;
                sim.units_left = units_left;
                sim.units_right = units_right;
                Ok(())
            })
            .log();

        source
    }
}

fn ids_by_slot_from_context(team: Entity, ctx: &ClientContext) -> Vec<u64> {
    let ids = team.ids(ctx).unwrap_or_default();
    let id = ids.into_iter().next().unwrap_or(0);
    if let Ok(team) = ctx.load::<NTeam>(id) {
        if let Ok(slots) = team.slots.load_nodes(ctx) {
            slots
                .into_iter()
                .sorted_by_key(|s| s.index)
                .filter_map(|slot| Some(slot.unit.load_node(ctx).ok()?.id))
                .collect_vec()
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}
#[derive(Debug)]
pub struct BattleSimulation {
    pub world: World,
    pub battle: Battle,
    pub duration: f32,
    pub turns: usize,
    pub units_left: Vec<u64>,
    pub units_right: Vec<u64>,
    pub team_left: u64,
    pub team_right: u64,
    pub log: BattleLog,
    pub rng: ChaCha8Rng,
    pub battle_texts: HashMap<BattleText, Vec<(f32, String)>>,
    pub fired: HashSet<u64>,
}
#[derive(Default, Debug, Clone)]
pub struct BattleLog {
    pub actions: Vec<BattleAction>,
}

#[derive(BevyComponent)]
pub struct Corpse;
#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub enum BattleText {
    CurrentEvent,
    Turn,
    Fatigue,
}

#[derive(Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum BattleAction {
    var_set(u64, VarName, VarValue),
    strike(u64, u64),
    damage(u64, u64, i32),
    heal(u64, u64, i32),
    death(u64),
    spawn(u64),
    apply_status {
        caster_id: u64,
        target_id: u64,
        status_path: String,
    },
    use_ability {
        caster_id: u64,
        target_id: u64,
        ability_path: String,
    },
    send_event(Event),
    vfx(Vec<ContextLayer>, String),
    wait(f32),
    fatigue(i32),
}
impl Hash for BattleAction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
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
            BattleAction::fatigue(a) => a.hash(state),
            BattleAction::apply_status {
                caster_id,
                target_id,
                status_path,
            } => {
                caster_id.hash(state);
                target_id.hash(state);
                status_path.hash(state);
            }
            BattleAction::use_ability {
                caster_id,
                target_id,
                ability_path,
            } => {
                caster_id.hash(state);
                target_id.hash(state);
                ability_path.hash(state);
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
            BattleAction::var_set(a, var, value) => format!("{a}${var}>{value}"),
            BattleAction::spawn(a) => format!("*{a}"),
            BattleAction::apply_status {
                caster_id,
                target_id,
                status_path,
            } => {
                format!("{caster_id} +{status_path}>{target_id}")
            }
            BattleAction::use_ability {
                caster_id,
                target_id,
                ability_path,
            } => format!("{caster_id}@{target_id}:{ability_path}"),
            BattleAction::wait(t) => format!("~{t}"),
            BattleAction::vfx(_, vfx) => format!("vfx({vfx})"),
            BattleAction::send_event(e) => format!("event({e})"),
            BattleAction::fatigue(pwr) => format!("fatigue({pwr})"),
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
        *ctx.t_mut().unwrap() = ctx.battle().unwrap().duration;
        let mut add_actions = Vec::default();
        let result: NodeResult<bool> = (|| {
            let applied = match self {
                BattleAction::strike(a, b) => {
                    BattleSimulation::send_event(ctx, Event::BeforeStrike(*a, *b))?;
                    add_actions.push(
                        Self::new_vfx("strike")
                            .with_owner(*a)
                            .with_target(*b)
                            .into(),
                    );
                    add_actions.push(Self::wait(animation_time() * 3.0));
                    let mut unit_a = ctx.load::<NUnit>(*a).track()?;
                    let mut unit_b = ctx.load::<NUnit>(*b).track()?;
                    let behavior_a = unit_a.behavior.load_node_mut(ctx).track()?;
                    let behavior_b = unit_b.behavior.load_node_mut(ctx).track()?;
                    let pwr_a = behavior_a.stats.load_node_mut(ctx).track()?.pwr;
                    let pwr_b = behavior_b.stats.load_node_mut(ctx).track()?.pwr;
                    add_actions.extend(ctx.battle()?.slots_sync());
                    add_actions.push(Self::wait(animation_time()));
                    add_actions.push(Self::damage(*a, *b, pwr_a));
                    add_actions.push(Self::damage(*b, *a, pwr_b));
                    add_actions.push(Self::wait(animation_time() * 2.0));
                    BattleSimulation::send_event(ctx, Event::AfterStrike(*a, *b))?;
                    true
                }
                BattleAction::death(a) => {
                    let position = ctx
                        .with_owner(*a, |context| context.get_var(VarName::position))
                        .track()?;
                    add_actions.push(
                        Self::new_vfx("death_vfx")
                            .with_var(VarName::position, position)
                            .into(),
                    );
                    add_actions.extend(BattleSimulation::die(ctx, *a)?);
                    true
                }
                BattleAction::damage(a, b, x) => {
                    let owner_pos = ctx
                        .get_var_inherited(*a, VarName::position)
                        .unwrap_or_default();
                    let target_pos = ctx.get_var_inherited(*b, VarName::position).track()?;
                    add_actions.push(
                        Self::new_vfx("range_effect_vfx")
                            .with_var(VarName::position, owner_pos)
                            .with_var(VarName::extra_position, target_pos.clone())
                            .into(),
                    );
                    let (value, actions) =
                        Event::ChangeOutgoingDamage(*a, *b).update_value(ctx, (*x).into(), *a);
                    add_actions.extend(actions);
                    let x = value.get_i32()?.at_least(0);
                    debug!("Before {x}");
                    let (value, actions) =
                        Event::ChangeIncomingDamage(*a, *b).update_value(ctx, x.into(), *b);
                    debug!("After {value}");
                    add_actions.extend(actions);
                    let x = value.get_i32()?.at_least(0);
                    if x > 0 {
                        BattleSimulation::send_event(ctx, Event::DamageDealt(*a, *b, x))?;
                        let dmg = ctx
                            .load::<NUnit>(*b)
                            .track()?
                            .state
                            .load_mut(ctx)
                            .track()?
                            .get()?
                            .dmg
                            + x;
                        add_actions.push(Self::var_set(*b, VarName::dmg, dmg.into()));
                        add_actions.push(
                            Self::new_vfx("pain_vfx")
                                .with_var(VarName::position, target_pos.clone())
                                .into(),
                        );
                    }
                    add_actions.push(
                        Self::new_text(format!("[b [red -{x}]]"), target_pos)
                            .with_var(VarName::scale, 2.0)
                            .into(),
                    );
                    add_actions.push(Self::wait(animation_time()));
                    true
                }
                BattleAction::heal(a, b, x) => {
                    let owner_pos = ctx
                        .with_owner(*a, |ctx| ctx.get_var(VarName::position))
                        .track()?;
                    let target_pos = ctx
                        .with_owner(*b, |ctx| ctx.get_var(VarName::position))
                        .track()?;
                    add_actions.push(
                        Self::new_vfx("range_effect_vfx")
                            .with_var(VarName::position, owner_pos)
                            .with_var(VarName::extra_position, target_pos.clone())
                            .into(),
                    );
                    if *x > 0 {
                        if let Some(pleasure) = animations().get("pleasure_vfx") {
                            ctx.with_layer(
                                ContextLayer::Var(VarName::position, target_pos.clone()),
                                |context| pleasure.apply(context),
                            )?;
                        }
                        let dmg = ctx
                            .load::<NUnit>(*b)
                            .track()?
                            .state
                            .load_mut(ctx)
                            .track()?
                            .get()?
                            .dmg
                            - *x;
                        add_actions.push(Self::var_set(*b, VarName::dmg, dmg.at_least(0).into()));
                        add_actions.push(
                            Self::new_text(format!("[b [green +{}]]", x), target_pos)
                                .with_var(VarName::scale, 1.5)
                                .into(),
                        );
                    }
                    add_actions.push(Self::wait(animation_time()));
                    true
                }
                BattleAction::var_set(id, var, value) => {
                    let old_value = ctx.source().get_var(*id, *var).unwrap_or_default();
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
                    add_actions.push(Self::wait(animation_time()));
                    true
                }
                BattleAction::apply_status {
                    caster_id,
                    target_id,
                    status_path,
                } => {
                    let (house, status) = find_status_by_path(status_path, ctx)?;
                    let color = house.color.load_node(ctx)?.color.c32();
                    BattleSimulation::apply_status(
                        ctx,
                        *caster_id,
                        *target_id,
                        status.clone(),
                        color,
                    )?;
                    add_actions.push(Self::wait(animation_time()));
                    true
                }
                BattleAction::use_ability {
                    caster_id,
                    target_id,
                    ability_path,
                } => {
                    let (_, ability) = find_ability_by_path(ability_path, ctx)?;
                    let effect = ability.effect.load_node(ctx)?;
                    let target = ctx.load::<NUnit>(*target_id).track()?;
                    let actions = effect
                        .effect
                        .execute_ability(ability.clone(), target, ctx)
                        .to_node_result()?;
                    let result = actions.is_empty();
                    for action in actions {
                        if let Ok(battle_action) = action.to_battle_action(ctx, *caster_id) {
                            add_actions.push(battle_action);
                        }
                    }
                    result
                }
                BattleAction::wait(t) => {
                    ctx.battle_mut()?.duration += *t;
                    false
                }
                BattleAction::vfx(layers, vfx) => {
                    if let Some(vfx) = animations().get(vfx) {
                        ctx.with_layers(layers.clone(), |ctx| vfx.apply(ctx))?;
                        true
                    } else {
                        false
                    }
                }
                BattleAction::fatigue(pwr) => {
                    let all_fusions = ctx.battle()?.all_fusions();
                    for fusion_id in all_fusions {
                        add_actions.push(Self::damage(0, fusion_id, *pwr));
                    }
                    add_actions.push(Self::wait(animation_time()));
                    true
                }
                BattleAction::send_event(event) => {
                    BattleSimulation::send_event(ctx, *event)?;
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
    pub fn new_text(text: impl ToString, position: impl Into<VarValue>) -> VfxBuilder {
        BattleAction::new_vfx("text")
            .with_var(VarName::text, text.to_string())
            .with_var(VarName::position, position)
            .with_var(VarName::color, colorix().high_contrast_text())
            .with_var(VarName::scale, 0.7)
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
            units_left: Vec::new(),
            units_right: Vec::new(),
            team_left: 0,
            team_right: 0,
            log: BattleLog::default(),
            rng: ChaCha8Rng::seed_from_u64(0),
            battle_texts: HashMap::new(),
            turns: 0,
            fired: HashSet::new(),
        }
    }
}

impl BattleSimulation {
    pub fn left_units(&self) -> &Vec<u64> {
        &self.units_left
    }

    pub fn right_units(&self) -> &Vec<u64> {
        &self.units_right
    }

    pub fn all_fusions(&self) -> Vec<u64> {
        let mut units = self.units_left.clone();
        units.append(&mut self.units_right.clone());
        units
    }

    pub fn all_allies(&self, id: u64) -> NodeResult<&Vec<u64>> {
        let left = &self.units_left;
        if left.contains(&id) {
            return Ok(left);
        } else {
            let right = &self.units_right;
            if right.contains(&id) {
                return Ok(right);
            }
        }
        Err(NodeError::custom(format!(
            "Failed to find allies: {id} is not in any team"
        )))
    }

    pub fn all_enemies(&self, id: u64) -> Result<&Vec<u64>, NodeError> {
        let left = &self.units_left;
        let right = &self.units_right;
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
        self.units_left.is_empty() || self.units_right.is_empty()
    }
    pub fn start(ctx: &mut ClientContext) -> NodeResult<()> {
        let spawn_actions = {
            let sim = ctx.battle()?;
            sim.units_left
                .iter()
                .zip_longest(sim.units_right.iter())
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
            Ok(_) => {
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

        let sim = ctx.battle_mut()?;
        sim.fired.clear();

        let sim = ctx.battle()?;
        let ids = sim
            .units_left
            .iter()
            .chain(sim.units_right.iter())
            .copied()
            .collect_vec();
        for id in ids {
            let vars = ctx.load::<NUnitStats>(id).track()?.get_vars();
            let mut actions = Vec::new();
            for (var, value) in vars {
                let (value, new_actions) = Event::UpdateStat(var).update_value(ctx, value, id);
                actions.extend(new_actions);
                let t = ctx.t().to_not_found()?;
                let entity = id.entity(ctx)?;
                let mut state = NodeStateHistory::load_mut(entity, ctx)?;
                state.insert(t, 0.0, var, value);
            }
            process_actions(ctx, actions);

            for (index, status) in ctx
                .get_children_of_kind(id, NodeKind::NStatusMagic)?
                .into_iter()
                .enumerate()
            {
                if let Ok(status_node) = ctx.load::<NStatusMagic>(status).track() {
                    if let Ok(state) = status_node.state.load_node(ctx) {
                        if state.stax > 0 {
                            ctx.source_mut()
                                .set_var(status, VarName::index, (index as i32).into())
                                .ok();
                        }
                    }
                }
            }
        }

        let sim = ctx.battle_mut()?;
        sim.duration += animation_time();
        sim.turns += 1;
        sim.add_text(
            BattleText::Turn,
            format!("[tw Turn] [yellow [b {}]]", sim.turns),
        );

        let sim = ctx.battle()?;
        if !sim.units_left.is_empty() && !sim.units_right.is_empty() {
            let a = BattleAction::strike(sim.units_left[0], sim.units_right[0]);
            process_actions(ctx, vec![a]);
        }

        let turn = ctx.battle()?.turns as i32;
        let fatigue_start = global_settings().match_settings.fatigue_start_turn as i32;

        if turn > fatigue_start {
            let fatigue_action = BattleAction::fatigue(turn - fatigue_start);
            let sim = ctx.battle_mut()?;
            sim.add_text(
                BattleText::Fatigue,
                format!("[tw Fatigue] [red [b {}]]", turn - fatigue_start),
            );
            process_actions(ctx, vec![fatigue_action]);
        }

        let mut actions = Vec::new();
        for (status, state) in ctx
            .world_mut()?
            .query::<(&NStatusMagic, &NState)>()
            .iter(ctx.world()?)
        {
            let alive = state.stax > 0;
            let visible = ctx
                .get_var_inherited(status.id, VarName::visible)
                .get_bool()
                .unwrap_or_default();
            if alive != visible {
                actions.push(BattleAction::var_set(
                    status.id,
                    VarName::visible,
                    alive.into(),
                ));
            }
        }
        process_actions(ctx, actions);
        let a = BattleSimulation::death_check(ctx)?;
        process_actions(ctx, a);
        let sync_actions = ctx.battle()?.slots_sync();
        process_actions(ctx, sync_actions);
        BattleSimulation::send_event(ctx, Event::TurnEnd)?;
        Ok(())
    }
    pub fn add_text(&mut self, text_type: BattleText, text: String) {
        self.battle_texts
            .entry(text_type)
            .or_insert_with(Vec::new)
            .push((self.duration, text));
    }
    pub fn get_text_at(&self, text_type: BattleText, t: f32) -> Option<&str> {
        self.battle_texts.get(&text_type).and_then(|entries| {
            entries
                .iter()
                .rev()
                .find(|(timestamp, _)| *timestamp <= t)
                .map(|(_, text)| text.as_str())
        })
    }
    pub fn send_event(ctx: &mut ClientContext, event: Event) -> NodeResult<()> {
        info!("{} {event}", "event:".dimmed().blue());
        ctx.exec_mut(|ctx| {
            match &event {
                Event::DamageDealt(attacker, _, _) => ctx.set_attacker(*attacker),
                _ => {}
            }
            let mut fusion_statuses: Vec<(u64, u64)> = default();
            let fusion_ids = {
                let sim = ctx.battle_mut()?;
                sim.add_text(BattleText::CurrentEvent, format!("[b [yellow ⚡️ {event}]]"));
                sim.all_fusions()
            };

            for id in fusion_ids {
                let should_skip = ctx.battle()?.fired.contains(&id);
                if should_skip {
                    continue;
                }
                let statuses = ctx.collect_kind_children(id, NodeKind::NStatusMagic)?;
                if !statuses.is_empty() {
                    fusion_statuses.extend(statuses.into_iter().map(|s_id| (id, s_id)));
                }
                ctx.with_layers([ContextLayer::Owner(id)], |ctx| {
                    if let Ok(unit) = ctx.load::<NUnit>(id).track() {
                        if let Ok(behavior) = unit.clone().behavior.load_node(ctx) {
                            if behavior.trigger.fire(&event, ctx)? {
                                use crate::plugins::rhai::{RhaiScriptUnitExt, TargetResolver};

                                match behavior.target.resolve_targets(ctx) {
                                    Ok(target_ids) => {
                                        if !target_ids.is_empty() {
                                            ctx.battle_mut()?.fired.insert(id);
                                            let mut all_battle_actions = Vec::new();

                                            for target_id in target_ids {
                                                if let Ok(target_unit) = ctx.load::<NUnit>(target_id).track() {
                                                    match behavior.effect.execute_unit(
                                                        unit.clone(),
                                                        target_unit,
                                                        0,
                                                        ctx,
                                                    ) {
                                                        Ok(actions) => {
                                                            for action in actions {
                                                                use crate::plugins::rhai::ToBattleAction;
                                                                if let Ok(ba) = action.to_battle_action(ctx, id) {
                                                                    all_battle_actions.push(ba);
                                                                }
                                                            }
                                                        }
                                                        Err(e) => error!("NFusion event {event} failed: {e}"),
                                                    }
                                                }
                                            }

                                            if !all_battle_actions.is_empty() {
                                                process_actions(ctx, all_battle_actions);
                                            }
                                        }
                                    }
                                    Err(e) => error!("Failed to resolve targets for unit behavior: {e}"),
                                }
                            }
                        }
                    }
                    Ok(())
                })
                .log();
            }
            for (fusion_id, status_id) in fusion_statuses {
                let should_skip = ctx.battle()?.fired.contains(&status_id);
                if should_skip {
                    continue;
                }
                match ctx.with_layers(
                    [
                        ContextLayer::Owner(fusion_id),
                        ContextLayer::Status(status_id),
                    ],
                    |ctx| {
                        let status = ctx.load::<NStatusMagic>(status_id)?;
                        let stax = status.state.load_node(ctx)?.stax;
                        if stax <= 0 {
                            return Ok(vec![]);
                        }
                        let behavior = status.behavior.load_node(ctx).track()?;
                        if behavior.trigger.fire(&event, ctx)? {
                            match behavior.effect.execute_status(status.clone(), stax, ctx) {
                                Ok(actions) => {
                                    if !actions.is_empty() {
                                        let owner_id = fusion_id;
                                        let mut battle_actions = Vec::new();
                                        for action in actions {
                                            use crate::plugins::rhai::ToBattleAction;
                                            if let Ok(ba) = action.to_battle_action(ctx, owner_id) {
                                                battle_actions.push(ba);
                                            }
                                        }
                                        process_actions(ctx, battle_actions);
                                    }
                                    Ok(default())
                                }
                                Err(e) => {
                                    error!("Status behavior failed: {e}");
                                    Ok(default())
                                }
                            }
                        } else {
                            Ok(default())
                        }
                    },
                ) {
                    Ok(actions) => {
                        if !actions.is_empty() {
                            ctx.battle_mut()?.fired.insert(status_id);
                            process_actions(ctx, actions);
                        }
                    }
                    Err(e) => e.log(),
                }
            }
            Ok(())
        })
    }
    pub fn apply_status(
        ctx: &mut ClientContext,
        caster: u64,
        target: u64,
        status: NStatusMagic,
        color: Color32,
    ) -> NodeResult<()> {
        let t = ctx.t().to_not_found()?;
        let mut new_index = 0;
        for child in ctx
            .get_children_of_kind(target, NodeKind::NStatusMagic)
            .track()?
        {
            if let Ok(child_status) = ctx.load::<NStatusMagic>(child) {
                new_index += 1;
                if child_status.status_name == status.status_name {
                    // Update stax on the status state component
                    if let Ok(mut state) = child_status.state.load_node(ctx) {
                        let new_stax = state.stax + status.state.load_node(ctx)?.stax;
                        state.set_var(VarName::stax, new_stax.into())?;
                        ctx.source_mut().commit(state)?;
                        BattleSimulation::send_event(
                            ctx,
                            Event::StatusApplied(caster, target, child_status.id),
                        )?;
                        return Ok(());
                    }
                }
            }
        }
        let entity = ctx.world_mut()?.spawn_empty().id();
        let status_id = status.id;
        let new_status = status.remap_ids();
        let new_status_id = new_status.id();
        new_status.spawn(ctx, Some(entity))?;
        // Status is already saved with its charges during spawn

        ctx.add_link(target, new_status_id)?;

        let mut state = NodeStateHistory::load_mut(entity, ctx)?;
        state.insert(0.0, 0.0, VarName::visible, false.into());
        state.insert(t, 0.0, VarName::visible, true.into());
        state.insert(t, 0.0, VarName::color, color.into());
        state.insert(t, 0.0, VarName::index, new_index.into());
        BattleSimulation::send_event(ctx, Event::StatusApplied(caster, target, new_status_id))?;
        BattleSimulation::send_event(ctx, Event::StatusGained(caster, target))?;
        Ok(())
    }

    pub fn die(ctx: &mut ClientContext, id: u64) -> NodeResult<Vec<BattleAction>> {
        let entity = id.entity(ctx)?;
        ctx.world_mut()?.entity_mut(entity).insert(Corpse);
        let mut died = false;
        let sim = ctx.battle_mut()?;
        if let Some(p) = sim.units_left.iter().position(|u| *u == id) {
            sim.units_left.remove(p);
            died = true;
        }
        if let Some(p) = sim.units_right.iter().position(|u| *u == id) {
            sim.units_right.remove(p);
            died = true;
        }
        if died {
            if sim.ended() {
                sim.duration += 2.0;
            }

            let mut actions = [BattleAction::var_set(id, VarName::visible, false.into())].to_vec();
            for child in ctx.children_recursive(id)? {
                actions.push(BattleAction::var_set(child, VarName::visible, false.into()));
            }
            actions.push(BattleAction::wait(animation_time()));
            Ok(actions)
        } else {
            Ok(default())
        }
    }
}

fn process_actions(ctx: &mut ClientContext, actions: impl Into<VecDeque<BattleAction>>) {
    let mut actions: VecDeque<BattleAction> = actions.into();
    while let Some(a) = actions.pop_front() {
        for a in a.apply(ctx).into_iter().rev() {
            actions.push_front(a);
        }
    }
}

impl BattleSimulation {
    pub fn death_check(ctx: &mut ClientContext) -> NodeResult<VecDeque<BattleAction>> {
        let mut actions: VecDeque<BattleAction> = default();
        *ctx.t_mut().unwrap() = ctx.battle()?.duration;
        let sim = ctx.battle()?;
        for id in sim.all_fusions() {
            let hp = ctx.get_var_inherited(id, VarName::hp).get_i32().track()?;
            let dmg = ctx.get_var_inherited(id, VarName::dmg).get_i32().track()?;
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
            .units_left
            .iter()
            .map(|e| (e, true))
            .enumerate()
            .chain(self.units_right.iter().map(|e| (e, false)).enumerate())
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
        actions
    }
}
