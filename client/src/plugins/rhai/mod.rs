mod behavior_executor;
mod script_actions;
mod script_context;
mod script_editor;
mod script_engine;
mod script_painter;
mod script_types;

use super::*;
pub use behavior_executor::*;
pub use script_actions::*;
pub use script_context::*;
pub use script_editor::*;
pub use script_engine::*;
pub use script_painter::*;
pub use script_types::*;

use ::rhai::{AST, Engine, EvalAltResult, Position};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct RhaiPlugin;

impl Plugin for RhaiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RhaiEngineResource::new())
            .add_systems(Startup, setup_rhai_engine);
    }
}

#[derive(Resource)]
pub struct RhaiEngineResource {
    pub engine: Arc<Engine>,
    pub compiled_scripts: Arc<RwLock<HashMap<u64, CompiledScript>>>,
}

impl RhaiEngineResource {
    pub fn new() -> Self {
        Self {
            engine: Arc::new(create_base_engine()),
            compiled_scripts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn compile_script<T>(&self, script: &RhaiScript<T>) -> Result<AST, Box<EvalAltResult>> {
        self.engine.compile(&script.code).map_err(|e| {
            Box::new(EvalAltResult::ErrorRuntime(
                format!("Parse error: {}", e).into(),
                Position::new(0, 0),
            ))
        })
    }

    pub fn store_compiled(&self, id: u64, script: CompiledScript) {
        self.compiled_scripts.write().insert(id, script);
    }

    pub fn get_compiled(&self, id: u64) -> Option<CompiledScript> {
        self.compiled_scripts.read().get(&id).cloned()
    }
}

#[derive(Clone)]
pub struct CompiledScript {
    pub ast: AST,
    pub last_compiled: std::time::Instant,
}

impl std::fmt::Debug for CompiledScript {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledScript")
            .field("last_compiled", &self.last_compiled)
            .finish()
    }
}

fn setup_rhai_engine(mut res: ResMut<RhaiEngineResource>) {
    register_client_types(&mut Arc::get_mut(&mut res.engine).unwrap());
}

pub fn create_base_engine() -> Engine {
    let mut engine = Engine::new();

    engine.set_max_expr_depths(100, 100);
    engine.set_max_call_levels(50);
    engine.set_max_operations(100_000);
    engine.set_max_string_size(10_000);
    engine.set_max_array_size(1_000);
    engine.set_max_map_size(1_000);

    engine
}

fn register_client_types(engine: &mut Engine) {
    register_unit_type(engine);
    register_unit_actions_type(engine);
    register_status_type(engine);
    register_status_actions_type(engine);
    register_ability_type(engine);
    register_ability_actions_type(engine);
    register_painter_functions(engine);
    register_context_type(engine);
    register_painter_type(engine);
    register_vec_extensions(engine);
    register_common_functions(engine);
}
