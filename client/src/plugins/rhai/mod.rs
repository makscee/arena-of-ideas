mod behavior_executor;
mod script_actions;
mod script_context;
mod script_editor;
mod script_engine;
mod script_types;

use super::*;
pub use behavior_executor::TargetResolver;
pub use script_actions::*;
pub use script_context::*;
pub use script_editor::*;
pub use script_engine::*;
pub use script_types::*;

use ::rhai::Engine;

static RHAI_ENGINE: OnceCell<Mutex<Engine>> = OnceCell::new();

pub fn rhai_engine() -> &'static Mutex<Engine> {
    RHAI_ENGINE.get_or_init(|| {
        let mut engine = create_base_engine();
        register_client_types(&mut engine);
        Mutex::new(engine)
    })
}

pub struct RhaiPlugin;

impl Plugin for RhaiPlugin {
    fn build(&self, _app: &mut App) {}
}

pub fn create_base_engine() -> Engine {
    let mut engine = Engine::new();

    engine
        .set_max_expr_depths(100, 100)
        .set_max_call_levels(500)
        .set_max_operations(100_000)
        .set_max_string_size(10_000)
        .set_max_array_size(1_000)
        .set_max_map_size(1_000);

    engine
}

fn register_client_types(engine: &mut Engine) {
    register_unit_type(engine);
    register_unit_actions_type(engine);
    register_status_type(engine);
    register_status_actions_type(engine);
    register_ability_type(engine);
    register_ability_actions_type(engine);
    register_context_type(engine);
    register_painter_type(engine);
    register_animator_type(engine);
    register_vec_extensions(engine);
    register_common_functions(engine);
}
