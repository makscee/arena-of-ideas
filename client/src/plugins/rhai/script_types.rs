use super::*;
use ::rhai::{EvalAltResult, Scope};
use schema::RhaiScript;

/// Extension trait for RhaiScript to provide client-side execution methods
pub trait RhaiScriptExt<T> {
    fn execute<S: Into<Scope<'static>>>(
        &self,
        scope_builder: S,
        ctx: &ClientContext,
    ) -> Result<(Vec<T>, Scope<'static>), Box<EvalAltResult>>;
}

impl<T> RhaiScriptExt<T> for RhaiScript<T>
where
    T: schema::ScriptAction,
{
    fn execute<S: Into<Scope<'static>>>(
        &self,
        scope_builder: S,
        ctx: &ClientContext,
    ) -> Result<(Vec<T>, Scope<'static>), Box<EvalAltResult>> {
        let mut scope = scope_builder.into();

        scope.push(T::actions_var_name(), Vec::<T>::new());
        scope.push("ctx", RhaiContext::with_context(ctx));

        let engine = super::rhai_engine().lock();
        let ast = self.get_ast(&engine)?;
        match engine.run_ast_with_scope(&mut scope, &ast) {
            Ok(_) => {
                self.run_error.write().unwrap().take();
            }
            Err(e) => {
                self.run_error.write().unwrap().replace(e.to_string());
                return Err(format!("Run err: {e}").into());
            }
        }

        let actions: Vec<T> = scope
            .get_value(T::actions_var_name())
            .ok_or_else(|| format!("Failed to get {} from scope", T::actions_var_name()))?;

        Ok((actions, scope))
    }
}

/// Extension methods for unit action scripts
pub trait RhaiScriptUnitExt {
    fn execute_unit(
        &self,
        owner: NUnit,
        target: NUnit,
        x: i32,
        ctx: &ClientContext,
    ) -> Result<Vec<schema::UnitAction>, Box<EvalAltResult>>;
}

impl RhaiScriptUnitExt for RhaiScript<schema::UnitAction> {
    fn execute_unit(
        &self,
        owner: NUnit,
        target: NUnit,
        x: i32,
        ctx: &ClientContext,
    ) -> Result<Vec<schema::UnitAction>, Box<EvalAltResult>> {
        let mut scope = Scope::new();
        scope.push("owner", owner);
        scope.push("target", target);
        scope.push("x", x);

        RhaiScriptExt::execute(self, scope, ctx).map(|r| r.0)
    }
}

/// Extension methods for status action scripts
pub trait RhaiScriptStatusExt {
    fn execute_status(
        &self,
        status: NStatusMagic,
        x: i32,
        ctx: &ClientContext,
    ) -> Result<Vec<schema::StatusAction>, Box<EvalAltResult>>;

    fn execute_status_with_value(
        &self,
        status: NStatusMagic,
        x: i32,
        ctx: &ClientContext,
    ) -> Result<(Vec<schema::StatusAction>, VarValue), Box<EvalAltResult>>;
}

impl RhaiScriptStatusExt for RhaiScript<schema::StatusAction> {
    fn execute_status(
        &self,
        status: NStatusMagic,
        x: i32,
        ctx: &ClientContext,
    ) -> Result<Vec<schema::StatusAction>, Box<EvalAltResult>> {
        let (actions, _) = self.execute_status_with_value(status, x, ctx)?;
        Ok(actions)
    }

    fn execute_status_with_value(
        &self,
        status: NStatusMagic,
        x: i32,
        ctx: &ClientContext,
    ) -> Result<(Vec<schema::StatusAction>, VarValue), Box<EvalAltResult>> {
        let mut scope = Scope::new();
        scope.push("status", status);
        scope.push("x", x);

        let initial_value = ctx.get_var(VarName::value).ok();
        if let Some(ref value) = initial_value {
            match value {
                VarValue::i32(v) => {
                    scope.push("value", *v);
                }
                VarValue::f32(v) => {
                    scope.push("value", *v);
                }
                _ => {
                    scope.push("value", 0i32);
                }
            }
        }

        if let Ok(owner_id) = ctx.owner() {
            if let Ok(owner_unit) = ctx.load::<NUnit>(owner_id).track() {
                scope.push("owner", owner_unit.clone());
            }

            if let Ok(enemies) = ctx.battle().and_then(|sim| sim.all_enemies(owner_id)) {
                if let Some(first_enemy_id) = enemies.first().copied() {
                    if let Ok(target_unit) = ctx.load::<NUnit>(first_enemy_id).track() {
                        scope.push("target", target_unit);
                    }
                }
            }
        }

        let (actions, scope) = RhaiScriptExt::execute(self, scope, ctx)?;

        let modified_value = if let Some(rhai_value) = scope.get_value::<i32>("value") {
            match initial_value {
                Some(VarValue::i32(_)) => VarValue::i32(rhai_value as i32),
                Some(VarValue::f32(_)) => VarValue::f32(rhai_value as f32),
                _ => VarValue::i32(rhai_value as i32),
            }
        } else {
            initial_value.unwrap_or_default()
        };

        Ok((actions, modified_value))
    }
}

/// Extension methods for ability action scripts
pub trait RhaiScriptAbilityExt {
    fn execute_ability(
        &self,
        ability: NAbilityMagic,
        target: NUnit,
        ctx: &ClientContext,
    ) -> Result<Vec<schema::AbilityAction>, Box<EvalAltResult>>;
}

impl RhaiScriptAbilityExt for RhaiScript<schema::AbilityAction> {
    fn execute_ability(
        &self,
        ability: NAbilityMagic,
        target: NUnit,
        ctx: &ClientContext,
    ) -> Result<Vec<schema::AbilityAction>, Box<EvalAltResult>> {
        let mut scope = Scope::new();
        scope.push("ability", ability);
        scope.push("target", target);

        RhaiScriptExt::execute(self, scope, ctx).map(|r| r.0)
    }
}

/// Extension methods for animator action scripts
pub trait RhaiScriptAnimatorExt {
    fn execute_animator(
        &self,
        ctx: &ClientContext,
    ) -> Result<Vec<crate::resources::anim::AnimAction>, Box<EvalAltResult>>;
}

impl RhaiScriptAnimatorExt for RhaiScript<crate::resources::anim::AnimAction> {
    fn execute_animator(
        &self,
        ctx: &ClientContext,
    ) -> Result<Vec<crate::resources::anim::AnimAction>, Box<EvalAltResult>> {
        let mut scope = Scope::new();
        if let Ok(owner) = ctx.owner() {
            scope.push("owner", owner);
        }
        if let Some(target) = ctx.target() {
            scope.push("target", target);
        }
        RhaiScriptExt::execute(self, scope, ctx).map(|r| r.0)
    }
}
