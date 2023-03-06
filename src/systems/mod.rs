use self::time_system::TimeSystem;

use super::*;

mod action_system;
mod battle_system;
mod cassette_player_system;
mod context_system;
mod file_watcher_system;
mod gallery_system;
mod game_over_system;
mod game_state_system;
mod house_system;
mod mouse_system;
mod name_system;
mod shader_system;
mod shop_system;
mod slot_system;
mod stats_ui_system;
mod time_system;
mod unit_system;
mod vfx_system;
mod world_system;

pub use action_system::*;
pub use battle_system::*;
pub use cassette_player_system::*;
pub use context_system::*;
pub use file_watcher_system::*;
pub use gallery_system::*;
pub use game_over_system::*;
pub use game_state_system::*;
use geng::prelude::itertools::Itertools;
pub use house_system::*;
pub use mouse_system::*;
pub use name_system::*;
pub use shader_system::*;
pub use shop_system::*;
pub use slot_system::*;
pub use stats_ui_system::*;
pub use unit_system::*;
pub use vfx_system::*;
pub use world_system::*;

pub trait System {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources);
    fn draw(
        &self,
        world: &legion::World,
        resources: &mut Resources,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        #![allow(unused_variables)]
    }
    fn ui<'a>(
        &'a mut self,
        cx: &'a ui::Controller,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        #![allow(unused_variables)]
        Box::new(ui::Void)
    }
}

impl Game {
    pub fn create_systems(resources: &mut Resources) -> Vec<Box<dyn System>> {
        let mut fws = FileWatcherSystem::new();
        resources.load(&mut fws);

        let mut global_systems: Vec<Box<dyn System>> = Vec::default();
        let mut game_state = GameStateSystem::new(GameState::MainMenu);
        game_state.add_systems(GameState::MainMenu, vec![]);
        game_state.add_systems(
            GameState::Battle,
            vec![
                Box::new(SlotSystem::new()),
                Box::new(CassettePlayerSystem::new(PlayMode::Play)),
                Box::new(ActionSystem::new()),
                Box::new(BattleSystem::new()),
            ],
        );
        game_state.add_systems(
            GameState::Shop,
            vec![
                Box::new(SlotSystem::new()),
                Box::new(ShopSystem::new()),
                Box::new(CassettePlayerSystem::new(PlayMode::Hidden)),
                Box::new(ActionSystem::new()),
                Box::new(MouseSystem::new()),
            ],
        );
        game_state.add_systems(
            GameState::Gallery,
            vec![
                Box::new(GallerySystem::new()),
                Box::new(SlotSystem::new()),
                Box::new(CassettePlayerSystem::new(PlayMode::Hidden)),
            ],
        );
        game_state.add_systems(
            GameState::GameOver,
            vec![
                Box::new(GameOverSystem::new()),
                Box::new(CassettePlayerSystem::new(PlayMode::Hidden)),
                Box::new(SlotSystem::new()),
            ],
        );

        global_systems.push(Box::new(fws));
        global_systems.push(Box::new(TimeSystem::new()));
        global_systems.push(Box::new(ContextSystem::new()));
        global_systems.push(Box::new(ShaderSystem::new()));
        global_systems.push(Box::new(game_state));
        global_systems
    }
}
