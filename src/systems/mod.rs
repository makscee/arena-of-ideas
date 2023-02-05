use self::time_system::TimeSystem;

use super::*;

mod action_system;
mod battle_system;
mod cassette_player_system;
mod file_watcher_system;
mod game_state_system;
mod shader_system;
mod stats_ui_system;
mod time_system;

pub use action_system::*;
pub use battle_system::*;
pub use cassette_player_system::*;
pub use file_watcher_system::*;
pub use game_state_system::*;
use geng::prelude::itertools::Itertools;
pub use shader_system::*;
pub use stats_ui_system::*;

pub trait System {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources);
    fn draw(
        &self,
        world: &legion::World,
        resources: &Resources,
        framebuffer: &mut ugli::Framebuffer,
    );
}

impl Game {
    pub fn create_active_systems(resources: &mut Resources) -> Vec<Box<dyn System>> {
        let mut fws = FileWatcherSystem::new();
        resources.load(&mut fws);

        let mut systems: Vec<Box<dyn System>> = Vec::default();
        systems.push(Box::new(GameStateSystem::new(GameState::MainMenu)));
        systems.push(Box::new(ShaderSystem::new(&resources.geng)));
        systems.push(Box::new(fws));
        systems.push(Box::new(TimeSystem::new()));
        systems.push(Box::new(ActionSystem::new()));
        systems.push(Box::new(CassettePlayerSystem::new()));
        systems
    }

    pub fn init_world(resources: &mut Resources, world: &mut legion::World) {
        Self::init_field(resources, world);
    }

    fn init_field(resources: &mut Resources, world: &mut legion::World) {
        let shader = resources.options.field.clone();
        let entity = world.push((shader,));
        world
            .entry(entity)
            .unwrap()
            .add_component((EntityComponent { entity }));
    }
}
