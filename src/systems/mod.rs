use super::*;
use geng::prelude::itertools::Itertools;

mod action_system;
mod battle_system;
mod button_system;
mod camera_system;
mod file_watcher_system;
// mod gallery_system;
mod game_over_system;
mod game_state_system;
mod input_system;
mod logger;
mod pool_ui_system;
mod rating_system;
mod save_system;
mod shader_system;
mod shop_system;
mod simulation_system;
mod slot_system;
mod stats_ui_system;
mod status_system;
mod tape_player_system;
mod team_system;
mod time_system;
mod unit_system;
mod vfx_system;
mod widgets;
mod world_system;

pub use action_system::*;
pub use battle_system::*;
pub use button_system::*;
pub use camera_system::*;
pub use file_watcher_system::*;
// pub use gallery_system::*;
pub use game_over_system::*;
pub use game_state_system::*;
pub use input_system::*;
pub use logger::*;
pub use pool_ui_system::*;
pub use rating_system::*;
pub use save_system::*;
pub use shader_system::*;
pub use shop_system::*;
pub use simulation_system::*;
pub use slot_system::*;
pub use stats_ui_system::*;
pub use status_system::*;
pub use tape_player_system::*;
pub use team_system::*;
pub use time_system::*;
pub use unit_system::*;
pub use vfx_system::*;
pub use widgets::*;
pub use world_system::*;

pub trait System {
    fn pre_update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        #![allow(unused_variables)]
    }
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        #![allow(unused_variables)]
    }
    fn post_update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        #![allow(unused_variables)]
    }
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
        world: &'a legion::World,
        resources: &'a Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        #![allow(unused_variables)]
        Box::new(ui::Void)
    }
}

impl Game {
    pub fn create_systems(_: &mut Resources) -> Vec<Box<dyn System>> {
        let mut global_systems: Vec<Box<dyn System>> = Vec::default();
        let mut game_state = GameStateSystem::new();
        game_state.add_systems(GameState::MainMenu, vec![]);
        game_state.add_systems(GameState::Battle, vec![Box::new(BattleSystem::new())]);
        game_state.add_systems(GameState::CustomGame, vec![]);
        game_state.add_systems(
            GameState::Shop,
            vec![Box::new(ShopSystem::new()), Box::new(PoolUiSystem::new())],
        );
        // game_state.add_systems(GameState::Gallery, vec![Box::new(GallerySystem::new())]);
        game_state.add_systems(GameState::GameOver, vec![Box::new(GameOverSystem::new())]);

        global_systems.push(Box::new(TimeSystem::new()));
        global_systems.push(Box::new(CameraSystem::new()));
        global_systems.push(Box::new(TapePlayerSystem::new()));
        global_systems.push(Box::new(SlotSystem::new()));
        global_systems.push(Box::new(ShaderSystem::new()));
        global_systems.push(Box::new(InputSystem::new()));
        global_systems.push(Box::new(game_state));
        global_systems
    }
}
