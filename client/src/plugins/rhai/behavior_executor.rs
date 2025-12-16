use super::script_actions::{
    AbilityAction as ScriptAbilityAction, StatusAction as ScriptStatusAction,
    UnitAction as ScriptUnitAction,
};
use super::*;
use crate::resources::battle::BattleAction;
use ::rhai::EvalAltResult;
use rand::seq::IndexedRandom;

/// Executor for unit behaviors with Rhai scripts
pub struct UnitBehaviorExecutor;

impl UnitBehaviorExecutor {
    pub fn execute(
        behavior: &NUnitBehavior,
        owner: NUnit,
        target: NUnit,
        x: i64,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<Vec<ScriptUnitAction>, Box<EvalAltResult>> {
        let mut script = RhaiScriptCompiled::from_schema(&behavior.effect);
        if !script.is_compiled() {
            script.compile(engine)?;
        }
        script.execute_unit(owner, target, x, engine, ctx)
    }

    pub fn execute_for_targets(
        behavior: &NUnitBehavior,
        owner: NUnit,
        targets: Vec<NUnit>,
        x: i64,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<Vec<ScriptUnitAction>, Box<EvalAltResult>> {
        let mut all_actions = Vec::new();

        for target in targets {
            let actions = Self::execute(behavior, owner.clone(), target, x, engine, ctx)?;
            all_actions.extend(actions);
        }

        Ok(all_actions)
    }
}

/// Executor for status behaviors with Rhai scripts
pub struct StatusBehaviorExecutor;

impl StatusBehaviorExecutor {
    pub fn execute(
        behavior: &NStatusBehavior,
        status: NStatusMagic,
        x: i64,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<Vec<ScriptStatusAction>, Box<EvalAltResult>> {
        let mut script = RhaiScriptCompiled::from_schema(&behavior.effect);
        if !script.is_compiled() {
            script.compile(engine)?;
        }
        script.execute_status(status, x, engine, ctx)
    }
}

/// Executor for ability effects with Rhai scripts
pub struct AbilityEffectExecutor;

impl AbilityEffectExecutor {
    pub fn execute(
        effect: &NAbilityEffect,
        ability: NAbilityMagic,
        target: NUnit,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<Vec<ScriptAbilityAction>, Box<EvalAltResult>> {
        let mut script = RhaiScriptCompiled::from_schema(&effect.effect);
        if !script.is_compiled() {
            script.compile(engine)?;
        }
        script.execute_ability(ability, target, engine, ctx)
    }
}

/// Convert UnitAction to BattleAction
impl ScriptUnitAction {
    pub fn to_battle_action(&self, ctx: &ClientContext, owner_id: u64) -> NodeResult<BattleAction> {
        match self {
            ScriptUnitAction::UseAbility {
                ability_name,
                target_id,
            } => {
                // Use vfx action to trigger ability usage
                Ok(BattleAction::vfx(
                    vec![
                        ContextLayer::Caster(owner_id),
                        ContextLayer::Target(*target_id),
                    ],
                    format!("ability:{}", ability_name),
                ))
            }
            ScriptUnitAction::ApplyStatus {
                status_name,
                target_id,
                stacks,
            } => Ok(BattleAction::apply_status {
                caster_id: owner_id,
                target_id: *target_id,
                status_path: status_name.clone(),
            }),
        }
    }
}

/// Convert StatusAction to BattleAction
impl ScriptStatusAction {
    pub fn to_battle_action(&self, ctx: &ClientContext, owner_id: u64) -> NodeResult<BattleAction> {
        match self {
            ScriptStatusAction::DealDamage { target_id, amount } => {
                Ok(BattleAction::damage(owner_id, *target_id, *amount))
            }
            ScriptStatusAction::HealDamage { target_id, amount } => {
                Ok(BattleAction::heal(owner_id, *target_id, *amount))
            }
            ScriptStatusAction::UseAbility {
                ability_name,
                target_id,
            } => {
                // Use vfx action to trigger ability usage
                Ok(BattleAction::vfx(
                    vec![
                        ContextLayer::Caster(owner_id),
                        ContextLayer::Target(*target_id),
                    ],
                    format!("ability:{}", ability_name),
                ))
            }
            ScriptStatusAction::ModifyStacks { delta } => {
                // For modify stacks, we need to update the status stacks on the owner
                Ok(BattleAction::var_set(
                    owner_id,
                    VarName::stax,
                    VarValue::i32(*delta),
                ))
            }
        }
    }
}

/// Convert AbilityAction to BattleAction
impl ScriptAbilityAction {
    pub fn to_battle_action(
        &self,
        ctx: &ClientContext,
        caster_id: u64,
    ) -> NodeResult<BattleAction> {
        match self {
            ScriptAbilityAction::DealDamage { target_id, amount } => {
                Ok(BattleAction::damage(caster_id, *target_id, *amount))
            }
            ScriptAbilityAction::HealDamage { target_id, amount } => {
                Ok(BattleAction::heal(caster_id, *target_id, *amount))
            }
            ScriptAbilityAction::ChangeStatus {
                status_name,
                target_id,
                delta,
            } => Ok(BattleAction::apply_status {
                caster_id,
                target_id: *target_id,
                status_path: status_name.clone(),
            }),
        }
    }
}

/// Implementation for NUnitBehavior with Rhai scripts
pub struct RhaiBehaviorImpl;

impl RhaiBehaviorImpl {
    pub fn react_for_unit_behavior(
        behavior: &NUnitBehavior,
        event: &Event,
        ctx: &mut ClientContext,
    ) -> NodeResult<Vec<BattleAction>> {
        // Check if trigger fires
        if !behavior.trigger.fire(event, ctx)? {
            return Ok(vec![]);
        }

        // For now, return empty actions - full execution requires context helper methods
        // that can't use ? operator in closures
        Ok(vec![])
    }

    pub fn react_for_status_behavior(
        behavior: &NStatusBehavior,
        status: NStatusMagic,
        event: &Event,
        ctx: &mut ClientContext,
    ) -> NodeResult<Vec<BattleAction>> {
        // Check if trigger fires
        if !behavior.trigger.fire(event, ctx)? {
            return Ok(vec![]);
        }

        // Get engine
        let engine = crate::plugins::rhai::create_base_engine();

        // Get value from context
        let x = match ctx.get_var(VarName::value)? {
            VarValue::i32(v) => v as i64,
            VarValue::f32(v) => v as i64,
            _ => 0,
        };

        // Execute script
        let owner_id = status.id;
        match StatusBehaviorExecutor::execute(behavior, status, x, &engine, ctx) {
            Ok(actions) => {
                let mut battle_actions = Vec::new();
                for action in actions {
                    battle_actions.push(action.to_battle_action(ctx, owner_id)?);
                }
                Ok(battle_actions)
            }
            Err(e) => {
                error!("Failed to execute status behavior script: {}", e);
                Ok(vec![])
            }
        }
    }
}

// Helper trait to resolve targets
trait TargetResolver {
    fn resolve_targets(&self, ctx: &mut ClientContext) -> NodeResult<Vec<u64>>;
}

impl TargetResolver for Target {
    fn resolve_targets(&self, ctx: &mut ClientContext) -> NodeResult<Vec<u64>> {
        let owner_id = ctx.owner()?;
        let battle = ctx.battle_mut()?;

        match self {
            Target::Owner => Ok(vec![owner_id]),
            Target::RandomEnemy => {
                let enemies = battle.all_enemies(owner_id)?.clone();
                Ok(enemies
                    .choose(&mut battle.rng)
                    .copied()
                    .into_iter()
                    .collect_vec())
            }
            Target::AllEnemies => battle.all_enemies(owner_id).cloned(),
            Target::RandomAlly => {
                let allies = battle.all_allies(owner_id)?.clone();
                Ok(allies
                    .choose(&mut battle.rng)
                    .copied()
                    .into_iter()
                    .collect_vec())
            }
            Target::AllAllies => battle.all_allies(owner_id).cloned(),
            Target::All => Ok(battle
                .all_allies(owner_id)?
                .into_iter()
                .chain(battle.all_enemies(owner_id)?.into_iter())
                .copied()
                .collect_vec()),
            Target::Caster => ctx.caster().to_not_found().map(|id| vec![id]),
            Target::Attacker => ctx.attacker().to_not_found().map(|id| vec![id]),
            Target::Target => ctx.target().to_not_found().map(|id| vec![id]),
            Target::AdjacentBack => battle
                .offset_unit(owner_id, -1)
                .to_not_found()
                .map(|id| vec![id]),
            Target::AdjacentFront => battle
                .offset_unit(owner_id, 1)
                .to_not_found()
                .map(|id| vec![id]),
            Target::AllyAtSlot(slot) => {
                let allies = battle.all_allies(owner_id)?;
                Ok(allies
                    .get(*slot as usize)
                    .copied()
                    .into_iter()
                    .collect_vec())
            }
            Target::EnemyAtSlot(slot) => {
                let enemies = battle.all_enemies(owner_id)?;
                Ok(enemies
                    .get(*slot as usize)
                    .copied()
                    .into_iter()
                    .collect_vec())
            }
            Target::List(targets) => {
                let mut all = Vec::new();
                for target in targets {
                    all.extend(target.resolve_targets(ctx)?);
                }
                Ok(all)
            }
        }
    }
}
