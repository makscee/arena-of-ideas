use super::*;

mod file_watcher_system;
mod game_state_system;
mod shader_system;

pub use file_watcher_system::*;
pub use game_state_system::*;
pub use shader_system::*;

pub trait System {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources);
    fn draw(
        &self,
        world: &legion::World,
        resources: &Resources,
        framebuffer: &mut ugli::Framebuffer,
    );
}
