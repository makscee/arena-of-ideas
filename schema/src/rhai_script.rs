use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// A Rhai script that can be compiled and executed to produce actions of type T
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhaiScript<T> {
    pub code: String,
    pub description: String,
    #[serde(skip)]
    compiled_ast: Arc<RwLock<Option<rhai::AST>>>,
    #[serde(skip)]
    _phantom: std::marker::PhantomData<T>,
}

impl<T> PartialEq for RhaiScript<T> {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code && self.description == other.description
    }
}

impl<T> Default for RhaiScript<T> {
    fn default() -> Self {
        Self {
            code: String::new(),
            description: String::new(),
            compiled_ast: Arc::new(RwLock::new(None)),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> RhaiScript<T> {
    pub fn new(code: String) -> Self {
        Self {
            code,
            description: String::new(),
            compiled_ast: Arc::new(RwLock::new(None)),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn empty() -> Self {
        Self {
            code: String::new(),
            description: String::new(),
            compiled_ast: Arc::new(RwLock::new(None)),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the compiled AST, compiling if necessary
    pub fn get_ast(&self, engine: &rhai::Engine) -> Result<rhai::AST, Box<rhai::EvalAltResult>> {
        let mut ast_guard = self.compiled_ast.write().unwrap();

        if let Some(ast) = ast_guard.as_ref() {
            return Ok(ast.clone());
        }

        // Compile and cache
        let ast = engine.compile(&self.code)?;
        *ast_guard = Some(ast.clone());
        Ok(ast)
    }

    /// Clear the compiled AST (useful when code changes)
    pub fn clear_compiled(&self) {
        *self.compiled_ast.write().unwrap() = None;
    }

    /// Check if the script is already compiled
    pub fn is_compiled(&self) -> bool {
        self.compiled_ast.read().unwrap().is_some()
    }
}

/// Trait for script action types that defines how they are executed
pub trait ScriptAction: Clone + Send + Sync + 'static {
    /// The name of the variable in the script scope that holds the actions vec
    fn actions_var_name() -> &'static str;
}

/// Actions that can be performed by units
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UnitAction {
    UseAbility {
        ability_name: String,
        target_id: u64,
    },
    ApplyStatus {
        status_name: String,
        target_id: u64,
        stacks: i32,
    },
}

impl ScriptAction for UnitAction {
    fn actions_var_name() -> &'static str {
        "unit_actions"
    }
}

/// Actions that can be performed by status effects
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StatusAction {
    DealDamage {
        target_id: u64,
        amount: i32,
    },
    HealDamage {
        target_id: u64,
        amount: i32,
    },
    UseAbility {
        ability_name: String,
        target_id: u64,
    },
    ModifyStacks {
        delta: i32,
    },
}

impl ScriptAction for StatusAction {
    fn actions_var_name() -> &'static str {
        "status_actions"
    }
}

/// Actions that can be performed by abilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AbilityAction {
    DealDamage {
        target_id: u64,
        amount: i32,
    },
    HealDamage {
        target_id: u64,
        amount: i32,
    },
    ChangeStatus {
        status_name: String,
        target_id: u64,
        delta: i32,
    },
}

impl ScriptAction for AbilityAction {
    fn actions_var_name() -> &'static str {
        "ability_actions"
    }
}

/// Actions that can be performed by painter scripts (client-side only visualization)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PainterAction {
    Paint,
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
    Curve { thickness: f32, curvature: f32 },
    Text { text: String },
    Hollow { width: f32 },
    Solid,
    Translate { x: f32, y: f32 },
    Rotate { angle: f32 },
    ScaleMesh { scale: f32 },
    ScaleRect { scale: f32 },
    Color { r: u8, g: u8, b: u8, a: u8 },
    Alpha { alpha: f32 },
    Feathering { amount: f32 },
    Exit,
}

impl ScriptAction for PainterAction {
    fn actions_var_name() -> &'static str {
        "painter"
    }
}
