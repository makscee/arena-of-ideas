use super::*;

mod entity_ext;
mod r#fn;

pub use entity_ext::*;
pub use r#fn::*;

pub fn egui_ctx(world: &mut World) -> &egui::Context {
    world.query::<&EguiContext>().single(world).unwrap().get()
}
