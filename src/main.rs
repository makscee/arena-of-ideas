mod components;
mod module_bindings;
mod plugins;
pub mod prelude;
mod resources;
mod utils;

pub use prelude::*;

fn main() {
    let mut app = App::new();
    app.init_state::<GameState>()
        .add_plugins(DefaultPlugins)
        .run();
}
