use super::*;
use ::rhai::{Engine, EvalAltResult, Scope};
use schema::RhaiScript;

/// Extension trait for RhaiScript to provide client-side execution methods
pub trait RhaiScriptExt<T> {
    /// Execute the script with a custom scope
    fn execute<S: Into<Scope<'static>>>(
        &self,
        scope_builder: S,
        ctx: &ClientContext,
    ) -> Result<Vec<T>, Box<EvalAltResult>>;
}

impl<T> RhaiScriptExt<T> for RhaiScript<T>
where
    T: schema::ScriptAction,
{
    fn execute<S: Into<Scope<'static>>>(
        &self,
        scope_builder: S,
        ctx: &ClientContext,
    ) -> Result<Vec<T>, Box<EvalAltResult>> {
        let mut scope = scope_builder.into();

        scope.push(T::actions_var_name(), Vec::<T>::new());
        scope.push("ctx", RhaiContext::with_context(ctx));

        let engine = super::rhai_engine().lock().unwrap();
        let ast = self.get_ast(&engine)?;
        engine.run_ast_with_scope(&mut scope, &ast)?;

        let actions: Vec<T> = scope
            .get_value(T::actions_var_name())
            .ok_or_else(|| format!("Failed to get {} from scope", T::actions_var_name()))?;

        Ok(actions)
    }
}

/// Extension methods for unit action scripts
pub trait RhaiScriptUnitExt {
    fn execute_unit(
        &self,
        owner: NUnit,
        target: NUnit,
        x: i64,
        ctx: &ClientContext,
    ) -> Result<Vec<schema::UnitAction>, Box<EvalAltResult>>;
}

impl RhaiScriptUnitExt for RhaiScript<schema::UnitAction> {
    fn execute_unit(
        &self,
        owner: NUnit,
        target: NUnit,
        x: i64,
        ctx: &ClientContext,
    ) -> Result<Vec<schema::UnitAction>, Box<EvalAltResult>> {
        let mut scope = Scope::new();
        scope.push("owner", owner);
        scope.push("target", target);
        scope.push("x", x);

        RhaiScriptExt::execute(self, scope, ctx)
    }
}

/// Extension methods for status action scripts
pub trait RhaiScriptStatusExt {
    fn execute_status(
        &self,
        status: NStatusMagic,
        x: i64,
        ctx: &ClientContext,
    ) -> Result<Vec<schema::StatusAction>, Box<EvalAltResult>>;
}

impl RhaiScriptStatusExt for RhaiScript<schema::StatusAction> {
    fn execute_status(
        &self,
        status: NStatusMagic,
        x: i64,
        ctx: &ClientContext,
    ) -> Result<Vec<schema::StatusAction>, Box<EvalAltResult>> {
        let mut scope = Scope::new();
        scope.push("status", status);
        scope.push("x", x);

        RhaiScriptExt::execute(self, scope, ctx)
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

        RhaiScriptExt::execute(self, scope, ctx)
    }
}

/// Extension methods for painter action scripts
pub trait RhaiScriptPainterExt {
    fn execute_painter(&self, ctx: &ClientContext) -> Result<Vec<String>, Box<EvalAltResult>>;
}

impl RhaiScriptPainterExt for RhaiScript<schema::RhaiPainterAction> {
    fn execute_painter(&self, ctx: &ClientContext) -> Result<Vec<String>, Box<EvalAltResult>> {
        let scope = Scope::new();
        let actions = RhaiScriptExt::execute(self, scope, ctx)?;
        Ok(actions.into_iter().map(|a| a.0).collect())
    }
}
