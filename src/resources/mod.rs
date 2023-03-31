use std::{collections::VecDeque, rc::Rc};

use super::*;

mod ability;
mod ability_name;
mod ability_pool;
mod camera;
mod cassette;
mod condition;
mod definitions;
mod effect;
mod event;
mod expression;
mod faction;
mod floors;
mod fonts;
mod hero_pool;
mod house;
mod house_pool;
mod image;
mod image_textures;
mod input;
mod options;
mod packed_unit;
mod shader_programs;
mod shop;
mod status;
mod team;
mod team_states;
mod trigger;
mod visual_effect;

pub use ability::*;
pub use ability_name::*;
pub use ability_pool::*;
pub use camera::*;
pub use cassette::*;
pub use condition::*;
pub use definitions::*;
pub use effect::*;
pub use event::*;
pub use expression::*;
pub use faction::*;
pub use floors::*;
pub use fonts::*;
use geng::prelude::file::load_json;
pub use hero_pool::*;
pub use house::*;
pub use house_pool::*;
pub use image::*;
pub use image_textures::*;
pub use input::*;
pub use options::*;
pub use packed_unit::*;
pub use shader_programs::*;
pub use shop::*;
pub use status::*;
pub use team::*;
pub use team_states::*;
pub use trigger::*;
pub use visual_effect::*;

pub struct Resources {
    pub logger: Logger,
    pub options: Options,
    pub reload_triggered: bool,

    pub shader_programs: ShaderPrograms,
    pub image_textures: ImageTextures,

    pub global_time: Time,
    pub delta_time: Time,
    pub status_pool: StatusPool,
    pub ability_pool: AbilityPool,
    pub team_states: TeamStates,
    pub action_queue: VecDeque<Action>,
    pub cassette: Cassette,
    pub cassette_play_mode: CassettePlayMode,
    pub frame_shaders: Vec<Shader>,

    pub shop: Shop,
    pub game_won: bool,
    pub last_round: usize,
    pub floors: Floors,

    pub house_pool: HousePool,
    pub hero_pool: HeroPool,
    pub definitions: Definitions,

    pub current_state: GameState,
    pub transition_state: GameState,

    pub input: Input,
    pub camera: Camera,
    pub fonts: Fonts,
    pub geng: Option<Geng>,
}

//todo: async loading
impl Resources {
    pub fn new(options: Options) -> Self {
        Self {
            geng: default(),
            camera: Camera::new(&options),
            fonts: default(),
            logger: default(),
            shader_programs: default(),
            image_textures: default(),
            global_time: default(),
            delta_time: default(),
            action_queue: default(),
            status_pool: default(),
            cassette: default(),
            shop: default(),
            frame_shaders: default(),
            input: default(),
            house_pool: default(),
            definitions: default(),
            floors: default(),
            reload_triggered: default(),
            game_won: default(),
            last_round: default(),
            hero_pool: default(),
            cassette_play_mode: CassettePlayMode::Play,
            transition_state: GameState::MainMenu,
            current_state: GameState::MainMenu,
            options,
            ability_pool: default(),
            team_states: default(),
        }
    }

    pub fn load(&mut self, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(
            &static_path().join("options.json"),
            Box::new(Options::loader),
        );
        HousePool::loader(self, &static_path().join("houses/_list.json"), watcher);
        HeroPool::loader(self, &static_path().join("units/_list.json"), watcher);
        Floors::loader(self, &static_path().join("floors.json"), watcher);

        self.logger.load(&self.options);
    }

    pub fn load_geng(&mut self, watcher: &mut FileWatcherSystem, geng: &Geng) {
        self.geng = Some(geng.clone());
        ShaderPrograms::shader_library_loader(
            self,
            &static_path().join("shaders/library/_list.json"),
            watcher,
        );
        ShaderPrograms::loader(self, &static_path().join("shaders/_list.json"), watcher);
        self.fonts = Fonts::new(geng);
        ImageTextures::loader(self, &static_path().join("images/_list.json"), watcher);
    }
}
