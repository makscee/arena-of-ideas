use super::*;
use ::rhai::{AST, Engine, EvalAltResult, Scope};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct CompiledScript {
    pub ast: AST,
}

/// Client-side representation of a Rhai script with compilation support
#[derive(bevy::prelude::Component, Debug, Clone, Serialize, Deserialize)]
pub struct RhaiScriptCompiled<T> {
    pub id: u64,
    pub code: String,
    #[serde(skip)]
    pub compiled: Option<CompiledScript>,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> Default for RhaiScriptCompiled<T> {
    fn default() -> Self {
        Self {
            id: 0,
            code: String::new(),
            compiled: None,
            _phantom: PhantomData,
        }
    }
}

impl<T> RhaiScriptCompiled<T> {
    pub fn new(id: u64, code: String) -> Self {
        Self {
            id,
            code,
            compiled: None,
            _phantom: PhantomData,
        }
    }

    pub fn from_schema(schema_script: &schema::RhaiScript<T>) -> Self {
        Self::new(0, schema_script.code.clone())
    }

    pub fn with_code(mut self, code: String) -> Self {
        self.code = code;
        self.compiled = None;
        self
    }

    pub fn is_compiled(&self) -> bool {
        self.compiled.is_some()
    }

    pub fn compile(&mut self, engine: &Engine) -> Result<(), Box<EvalAltResult>> {
        let ast = engine.compile(&self.code)?;
        self.compiled = Some(CompiledScript { ast });
        Ok(())
    }

    fn get_ast(&self) -> Result<AST, Box<EvalAltResult>> {
        self.compiled
            .as_ref()
            .map(|c| c.ast.clone())
            .ok_or_else(|| "Script not compiled".into())
    }
}

impl<T> RhaiScriptCompiled<T>
where
    T: ScriptAction,
{
    /// Generic execute function that works with any ScriptAction type
    pub fn execute<S: Into<Scope<'static>>>(
        &self,
        scope_builder: S,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<Vec<T>, Box<EvalAltResult>> {
        let mut scope = scope_builder.into();

        // Add the actions vector with the appropriate variable name
        scope.push(T::actions_var_name(), Vec::<T>::new());

        // Add the context
        scope.push("ctx", RhaiContext::with_context(ctx));

        let ast = self.get_ast()?;
        engine.run_ast_with_scope(&mut scope, &ast)?;

        // Retrieve the actions from the scope
        let actions: Vec<T> = scope
            .get_value(T::actions_var_name())
            .ok_or_else(|| format!("Failed to get {} from scope", T::actions_var_name()))?;

        Ok(actions)
    }
}

// Specific execute methods for convenience
impl RhaiScriptCompiled<Vec<super::script_actions::UnitAction>> {
    pub fn execute_unit(
        &self,
        owner: NUnit,
        target: NUnit,
        x: i64,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<Vec<super::script_actions::UnitAction>, Box<EvalAltResult>> {
        let mut scope = Scope::new();
        scope.push("owner", owner);
        scope.push("target", target);
        scope.push("x", x);

        self.execute(scope, engine, ctx)
    }
}

impl RhaiScriptCompiled<schema::UnitAction> {
    pub fn execute_unit(
        &self,
        owner: NUnit,
        target: NUnit,
        x: i64,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<Vec<super::script_actions::UnitAction>, Box<EvalAltResult>> {
        let mut scope = Scope::new();
        scope.push("owner", owner);
        scope.push("target", target);
        scope.push("x", x);

        // For marker types, execute as Vec<UnitAction>
        let mut script_clone = self.clone();
        let result: Result<Vec<super::script_actions::UnitAction>, _> = {
            let mut scope = scope;
            scope.push(
                super::script_actions::UnitAction::actions_var_name(),
                Vec::<super::script_actions::UnitAction>::new(),
            );
            scope.push("ctx", RhaiContext::with_context(ctx));

            let ast = self.get_ast()?;
            engine.run_ast_with_scope(&mut scope, &ast)?;

            scope
                .get_value(super::script_actions::UnitAction::actions_var_name())
                .ok_or_else(|| {
                    format!(
                        "Failed to get {} from scope",
                        super::script_actions::UnitAction::actions_var_name()
                    )
                    .into()
                })
        };
        result
    }
}

impl RhaiScriptCompiled<schema::StatusAction> {
    pub fn execute_status(
        &self,
        status: NStatusMagic,
        x: i64,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<Vec<super::script_actions::StatusAction>, Box<EvalAltResult>> {
        let mut scope = Scope::new();
        scope.push("status", status);
        scope.push("x", x);
        scope.push(
            super::script_actions::StatusAction::actions_var_name(),
            Vec::<super::script_actions::StatusAction>::new(),
        );
        scope.push("ctx", RhaiContext::with_context(ctx));

        let ast = self.get_ast()?;
        engine.run_ast_with_scope(&mut scope, &ast)?;

        scope
            .get_value(super::script_actions::StatusAction::actions_var_name())
            .ok_or_else(|| {
                format!(
                    "Failed to get {} from scope",
                    super::script_actions::StatusAction::actions_var_name()
                )
                .into()
            })
    }
}

impl RhaiScriptCompiled<Vec<super::script_actions::StatusAction>> {
    pub fn execute_status(
        &self,
        status: NStatusMagic,
        x: i64,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<Vec<super::script_actions::StatusAction>, Box<EvalAltResult>> {
        let mut scope = Scope::new();
        scope.push("status", status);
        scope.push("x", x);

        self.execute(scope, engine, ctx)
    }
}

impl RhaiScriptCompiled<schema::AbilityAction> {
    pub fn execute_ability(
        &self,
        ability: NAbilityMagic,
        target: NUnit,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<Vec<super::script_actions::AbilityAction>, Box<EvalAltResult>> {
        let mut scope = Scope::new();
        scope.push("ability", ability);
        scope.push("target", target);
        scope.push(
            super::script_actions::AbilityAction::actions_var_name(),
            Vec::<super::script_actions::AbilityAction>::new(),
        );
        scope.push("ctx", RhaiContext::with_context(ctx));

        let ast = self.get_ast()?;
        engine.run_ast_with_scope(&mut scope, &ast)?;

        scope
            .get_value(super::script_actions::AbilityAction::actions_var_name())
            .ok_or_else(|| {
                format!(
                    "Failed to get {} from scope",
                    super::script_actions::AbilityAction::actions_var_name()
                )
                .into()
            })
    }
}

impl RhaiScriptCompiled<Vec<super::script_actions::AbilityAction>> {
    pub fn execute_ability(
        &self,
        ability: NAbilityMagic,
        target: NUnit,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<Vec<super::script_actions::AbilityAction>, Box<EvalAltResult>> {
        let mut scope = Scope::new();
        scope.push("ability", ability);
        scope.push("target", target);

        self.execute(scope, engine, ctx)
    }
}

impl RhaiScriptCompiled<Vec<super::script_actions::RhaiPainterAction>> {
    pub fn execute_painter(
        &self,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<Vec<String>, Box<EvalAltResult>> {
        let scope = Scope::new();
        let actions = self.execute(scope, engine, ctx)?;
        Ok(actions.into_iter().map(|a| a.0).collect())
    }
}

// Tier implementation for script-based behaviors
impl schema::Tier for RhaiScriptCompiled<Vec<super::script_actions::UnitAction>> {
    fn tier(&self) -> u8 {
        1 // Default tier for scripts
    }
}

impl schema::Tier for RhaiScriptCompiled<Vec<super::script_actions::StatusAction>> {
    fn tier(&self) -> u8 {
        1 // Default tier for scripts
    }
}

impl schema::Tier for RhaiScriptCompiled<Vec<super::script_actions::AbilityAction>> {
    fn tier(&self) -> u8 {
        1 // Default tier for scripts
    }
}

impl schema::Tier for NUnitBehavior {
    fn tier(&self) -> u8 {
        let trigger_tier = self.trigger.tier();
        let target_tier = self.target.tier();
        let effect_tier = 1; // Script-based effects have tier 1
        (trigger_tier + target_tier + effect_tier) / 3
    }
}

impl schema::Tier for NStatusBehavior {
    fn tier(&self) -> u8 {
        let trigger_tier = self.trigger.tier();
        let effect_tier = 1; // Script-based effects have tier 1
        (trigger_tier + effect_tier) / 2
    }
}

impl schema::Tier for NAbilityEffect {
    fn tier(&self) -> u8 {
        1 // Script-based effects have tier 1
    }
}
