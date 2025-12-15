mod script_actions;
mod script_context;
mod script_editor;
mod script_engine;
mod script_painter;
mod script_types;

use super::*;
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
            .add_systems(Startup, setup_rhai_engine)
            .add_systems(Update, process_script_compilation_requests);
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

    pub fn compile_script(&self, script: &RhaiScript) -> Result<AST, Box<EvalAltResult>> {
        self.engine.compile(&script.code).map_err(|e| {
            Box::new(EvalAltResult::ErrorRuntime(
                format!("Parse error: {}", e).into(),
                Position::new(0, 0)
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
    pub script_type: ScriptType,
    pub last_compiled: std::time::Instant,
}

impl std::fmt::Debug for CompiledScript {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledScript")
            .field("script_type", &self.script_type)
            .finish()
    }
}

fn setup_rhai_engine(mut res: ResMut<RhaiEngineResource>) {
    register_client_types(&mut Arc::get_mut(&mut res.engine).unwrap());
}

fn process_script_compilation_requests(
    engine_res: Res<RhaiEngineResource>,
    mut script_query: Query<&mut RhaiScript, bevy::prelude::Changed<RhaiScript>>,
) {
    for mut script in script_query.iter_mut() {
        if let Ok(ast) = engine_res.compile_script(&script) {
            let compiled = CompiledScript {
                ast,
                script_type: script.script_type.clone(),
                last_compiled: std::time::Instant::now(),
            };
            engine_res.store_compiled(script.id, compiled.clone());
            script.compiled = Some(compiled);
        }
    }
}

fn create_base_engine() -> Engine {
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
