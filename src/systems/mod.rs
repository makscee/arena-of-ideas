use super::*;

mod game_state_system;
mod shader_system;

pub use game_state_system::*;
use geng::prelude::ugli::Framebuffer;
pub use shader_system::*;

pub trait System {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources);
    fn draw(&self, world: &legion::World, resources: &Resources, framebuffer: &mut Framebuffer);
}
