use self::time_system::TimeSystem;

use super::*;

mod action_system;
mod battle_system;
mod cassette_player_system;
mod drag_system;
mod file_watcher_system;
mod game_state_system;
mod name_system;
mod shader_system;
mod shop_system;
mod slot_system;
mod stats_ui_system;
mod time_system;

pub use action_system::*;
pub use battle_system::*;
pub use cassette_player_system::*;
pub use drag_system::*;
pub use file_watcher_system::*;
pub use game_state_system::*;
use geng::prelude::itertools::Itertools;
pub use name_system::*;
pub use shader_system::*;
pub use shop_system::*;
pub use slot_system::*;
pub use stats_ui_system::*;

pub trait System {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources);
    fn draw(
        &self,
        world: &legion::World,
        resources: &Resources,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        #![allow(unused_variables)]
    }
    fn ui<'a>(
        &'a mut self,
        cx: &'a ui::Controller,
        resources: &Resources,
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
                Box::new(CassettePlayerSystem::new(PlayMode::Play)),
                Box::new(ActionSystem::new()),
                Box::new(SlotSystem::new()),
            ],
        );
        game_state.add_systems(
            GameState::Shop,
            vec![
                Box::new(CassettePlayerSystem::new(PlayMode::Hidden)),
                Box::new(ActionSystem::new()),
                Box::new(ShopSystem::new()),
                Box::new(DragSystem::new()),
                Box::new(SlotSystem::new()),
            ],
        );

        global_systems.push(Box::new(fws));
        global_systems.push(Box::new(TimeSystem::new()));
        global_systems.push(Box::new(ShaderSystem::new()));
        global_systems.push(Box::new(game_state));
        global_systems
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
