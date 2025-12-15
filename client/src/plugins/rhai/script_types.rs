use super::*;
use ::rhai::{AST, Dynamic, Engine, EvalAltResult, Scope};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScriptType {
    Value { return_type: String },
    UnitAction,
    StatusAction,
    AbilityAction,
    Painter,
}

#[derive(bevy::prelude::Component, Debug, Clone, Serialize, Deserialize)]
pub struct RhaiScript {
    pub id: u64,
    pub code: String,
    pub script_type: ScriptType,
    #[serde(skip)]
    pub compiled: Option<CompiledScript>,
}

impl Default for RhaiScript {
    fn default() -> Self {
        Self {
            id: 0,
            code: String::new(),
            script_type: ScriptType::Value {
                return_type: "i32".to_string(),
            },
            compiled: None,
        }
    }
}

impl RhaiScript {
    pub fn new(id: u64, script_type: ScriptType) -> Self {
        Self {
            id,
            code: match &script_type {
                ScriptType::Value { .. } => "value = 0".to_string(),
                ScriptType::UnitAction => {
                    "// Available: owner, target, x, unit_actions\n".to_string()
                }
                ScriptType::StatusAction => "// Available: status, x, status_actions\n".to_string(),
                ScriptType::AbilityAction => "// Available: ability, ability_actions\n".to_string(),
                ScriptType::Painter => "// Available: painter_actions\n".to_string(),
            },
            script_type,
            compiled: None,
        }
    }

    pub fn with_code(mut self, code: String) -> Self {
        self.code = code;
        self
    }

    pub fn is_compiled(&self) -> bool {
        self.compiled.is_some()
    }

    pub fn execute_value<T: Clone + Send + Sync + 'static>(
        &self,
        initial_value: T,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<T, Box<EvalAltResult>> {
        let mut scope = Scope::new();
        scope.push("value", initial_value);
        scope.push("ctx", RhaiContext::with_context(ctx));

        let ast = self.get_ast()?;
        engine.run_ast_with_scope(&mut scope, &ast)?;

        let value: Dynamic = scope
            .get_value("value")
            .ok_or_else(|| "Failed to get value from scope")?;

        value
            .try_cast::<T>()
            .ok_or_else(|| "Failed to cast value to expected type".into())
    }

    pub fn execute_unit_action(
        &self,
        owner: NUnit,
        target: NUnit,
        x: i64,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<Vec<UnitAction>, Box<EvalAltResult>> {
        let mut scope = Scope::new();

        scope.push("owner", owner);
        scope.push("target", target);
        scope.push("x", x);
        scope.push("unit_actions", Vec::<UnitAction>::new());
        scope.push("ctx", RhaiContext::with_context(ctx));

        let ast = self.get_ast()?;
        engine.run_ast_with_scope(&mut scope, &ast)?;

        let actions: Vec<UnitAction> = scope
            .get_value("unit_actions")
            .ok_or_else(|| "Failed to get actions from scope")?;

        Ok(actions)
    }

    pub fn execute_status_action(
        &self,
        status: NStatusMagic,
        x: i64,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<Vec<StatusAction>, Box<EvalAltResult>> {
        let mut scope = Scope::new();

        scope.push("status", status);
        scope.push("x", x);
        scope.push("status_actions", Vec::<StatusAction>::new());
        scope.push("ctx", RhaiContext::with_context(ctx));

        let ast = self.get_ast()?;
        engine.run_ast_with_scope(&mut scope, &ast)?;

        let actions: Vec<StatusAction> = scope
            .get_value("status_actions")
            .ok_or_else(|| "Failed to get actions from scope")?;

        Ok(actions)
    }

    pub fn execute_ability_action(
        &self,
        ability: NAbilityMagic,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<Vec<AbilityAction>, Box<EvalAltResult>> {
        let mut scope = Scope::new();

        scope.push("ability", ability);
        scope.push("ability_actions", Vec::<AbilityAction>::new());
        scope.push("ctx", RhaiContext::with_context(ctx));

        let ast = self.get_ast()?;
        engine.run_ast_with_scope(&mut scope, &ast)?;

        let actions: Vec<AbilityAction> = scope
            .get_value("ability_actions")
            .ok_or_else(|| "Failed to get actions from scope")?;

        Ok(actions)
    }

    pub fn execute_painter(
        &self,
        engine: &Engine,
        ctx: &ClientContext,
    ) -> Result<Vec<String>, Box<EvalAltResult>> {
        let mut scope = Scope::new();

        scope.push("painter_actions", Vec::<String>::new());
        scope.push("ctx", RhaiContext::with_context(ctx));

        let ast = self.get_ast()?;
        engine.run_ast_with_scope(&mut scope, &ast)?;

        let actions: Vec<String> = scope
            .get_value("painter_actions")
            .ok_or_else(|| "Failed to get actions from scope")?;

        Ok(actions)
    }

    fn get_ast(&self) -> Result<AST, Box<EvalAltResult>> {
        self.compiled
            .as_ref()
            .map(|c| c.ast.clone())
            .ok_or_else(|| "Script not compiled".into())
    }
}
