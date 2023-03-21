use std::{collections::VecDeque, rc::Rc};

use super::*;

mod ability;
mod ability_state;
mod camera;
mod cassette;
mod condition;
mod definitions;
mod effect;
mod event;
mod expression;
mod floors;
mod fonts;
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
mod team_pool;
mod trigger;
mod visual_effect;

pub use ability::*;
pub use ability_state::*;
pub use camera::*;
pub use cassette::*;
pub use condition::*;
pub use definitions::*;
pub use effect::*;
pub use event::*;
pub use expression::*;
pub use floors::*;
pub use fonts::*;
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
pub use team_pool::*;
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
    pub action_queue: VecDeque<Action>,
    pub cassette: Cassette,
    pub cassette_play_mode: CassettePlayMode,
    pub frame_shaders: Vec<Shader>,

    pub shop: Shop,
    pub game_won: bool,
    pub last_round: usize,
    pub floors: Floors,

    pub house_pool: HousePool,
    pub hero_pool: HashMap<PathBuf, PackedUnit>,
    pub team_pool: TeamPool,
    pub unit_corpses: HashMap<legion::Entity, (PackedUnit, Faction)>,
    pub definitions: Definitions,

    pub current_state: GameState,
    pub transition_state: GameState,

    pub input: Input,
    pub camera: Camera,
    pub fonts: Fonts,
    pub geng: Geng,
}

//todo: async loading
impl Resources {
    pub fn new(geng: &Geng) -> Self {
        // todo: load all Resources as geng::Assets
        let options = futures::executor::block_on(<Options as geng::LoadAsset>::load(
            geng,
            &static_path().join("options.json"),
        ))
        .unwrap();
        let fonts = vec![
            Rc::new(
                geng::font::Ttf::new(
                    geng,
                    include_bytes!("../../static/font/zorque.ttf"),
                    geng::font::ttf::Options {
                        pixel_size: 32.0,
                        max_distance: 0.25,
                    },
                )
                .unwrap(),
            ),
            Rc::new(
                geng::font::Ttf::new(
                    geng,
                    include_bytes!("../../static/font/roboto.ttf"),
                    geng::font::ttf::Options {
                        pixel_size: 32.0,
                        max_distance: 0.25,
                    },
                )
                .unwrap(),
            ),
            Rc::new(
                geng::font::Ttf::new(
                    geng,
                    include_bytes!("../../static/font/amontesa.ttf"),
                    geng::font::ttf::Options {
                        pixel_size: 32.0,
                        max_distance: 0.25,
                    },
                )
                .unwrap(),
            ),
        ];
        let team_pool = TeamPool::new(hashmap! {
            Faction::Team => Team::empty(options.player_team_name.clone())
        });

        Self {
            geng: geng.clone(),
            camera: Camera::new(&options),
            fonts: Fonts::new(fonts),
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
            unit_corpses: default(),
            cassette_play_mode: CassettePlayMode::Play,
            transition_state: GameState::MainMenu,
            current_state: GameState::MainMenu,
            options,
            team_pool,
        }
    }

    pub fn load(&mut self, fws: &mut FileWatcherSystem) {
        self.load_shader_programs(fws);
        self.load_image_textures(fws);
        self.load_houses(fws);
        self.load_hero_pool(fws);
        fws.load_and_watch_file(
            self,
            &static_path().join("options.json"),
            Box::new(Self::load_options),
        );
        fws.load_and_watch_file(
            self,
            &static_path().join("floors.json"),
            Box::new(Self::load_rounds),
        );
    }

    pub async fn load_list(geng: &Geng, path: PathBuf) -> Vec<PathBuf> {
        let list = <String as geng::LoadAsset>::load(&geng, &static_path().join(path))
            .await
            .expect("Failed to load list");
        let list: Vec<String> = serde_json::from_str(&list).expect("Failed to parse shaders list");
        let list = list
            .iter()
            .map(|path| PathBuf::try_from(path).unwrap())
            .collect();
        list
    }

    fn load_shader_programs(&mut self, fws: &mut FileWatcherSystem) {
        // load & watch shader library
        let list = futures::executor::block_on(Self::load_list(
            &self.geng,
            PathBuf::try_from("shaders/library/_list.json").unwrap(),
        ));
        for file in list {
            fws.load_and_watch_file(
                self,
                &static_path().join(file),
                Box::new(Self::load_shader_library_program),
            );
        }

        // load & watch regular shaders
        let list = futures::executor::block_on(Self::load_list(
            &self.geng,
            PathBuf::try_from("shaders/_list.json").unwrap(),
        ));
        for file in list {
            fws.load_and_watch_file(
                self,
                &static_path().join(file),
                Box::new(Self::load_shader_program),
            );
        }
    }

    fn load_shader_program(resources: &mut Resources, file: &PathBuf) {
        let program = futures::executor::block_on(<ugli::Program as geng::LoadAsset>::load(
            &resources.geng,
            &file,
        ));
        match program {
            Ok(program) => {
                debug!("Load shader program {:?}", file);
                resources
                    .shader_programs
                    .insert_program(file.clone(), program)
            }
            Err(error) => {
                error!("Failed to load shader program {:?} {}", file, error);
            }
        }
    }

    fn load_shader_library_program(resources: &mut Resources, file: &PathBuf) {
        let program =
            &futures::executor::block_on(<String as geng::LoadAsset>::load(&resources.geng, &file))
                .expect(&format!("Failed to load shader {:?}", file));
        debug!("Load shader library program {:?}", file);
        resources
            .geng
            .shader_lib()
            .add(file.file_name().unwrap().to_str().unwrap(), program);
    }

    fn load_image_textures(&mut self, fws: &mut FileWatcherSystem) {
        let list = futures::executor::block_on(Self::load_list(
            &self.geng,
            PathBuf::try_from("images/_list.json").unwrap(),
        ));
        for file in list {
            fws.load_and_watch_file(
                self,
                &static_path().join(file),
                Box::new(Self::load_image_texture),
            );
        }
    }

    fn load_image_texture(resources: &mut Resources, file: &PathBuf) {
        let texture = futures::executor::block_on(<ugli::Texture as geng::LoadAsset>::load(
            &resources.geng,
            &file,
        ));
        match texture {
            Ok(texture) => {
                debug!("Load texture {:?}", file);
                resources
                    .image_textures
                    .insert_texture(file.clone(), texture)
            }
            Err(error) => {
                error!("Failed to load texture {:?} {}", file, error);
            }
        }
    }

    fn load_hero_pool(&mut self, fws: &mut FileWatcherSystem) {
        let list = futures::executor::block_on(Self::load_list(
            &self.geng,
            PathBuf::try_from("units/_list.json").unwrap(),
        ));

        for file in list {
            fws.load_and_watch_file(self, &static_path().join(file), Box::new(Self::load_hero));
        }
    }

    fn load_hero(resources: &mut Resources, file: &PathBuf) {
        let json =
            futures::executor::block_on(<String as geng::LoadAsset>::load(&resources.geng, file))
                .expect("Failed to load unit");
        let unit =
            serde_json::from_str(&json).expect(&format!("Failed to parse UnitTemplate {:?}", file));
        resources.hero_pool.insert(file.clone(), unit);
    }

    fn load_options(resources: &mut Resources, file: &PathBuf) {
        resources.options =
            futures::executor::block_on(<Options as geng::LoadAsset>::load(&resources.geng, &file))
                .unwrap();
        resources.logger.load(&resources.options);
    }

    fn load_houses(&mut self, fws: &mut FileWatcherSystem) {
        let list = futures::executor::block_on(Self::load_list(
            &self.geng,
            PathBuf::try_from("houses/_list.json").unwrap(),
        ));
        for file in list {
            fws.load_and_watch_file(self, &static_path().join(file), Box::new(Self::load_house));
        }
    }

    fn load_house(resources: &mut Resources, file: &PathBuf) {
        let json =
            futures::executor::block_on(<String as geng::LoadAsset>::load(&resources.geng, file))
                .expect(&format!("Failed to load House {:?}", file));
        let house: House =
            serde_json::from_str(&json).expect(&format!("Failed to parse House: {:?}", file));
        house.statuses.iter().for_each(|(name, status)| {
            let mut status = status.clone();
            if status.color.is_none() {
                status.color = Some(house.color);
            }
            resources.status_pool.define_status(name.clone(), status)
        });
        house.abilities.iter().for_each(|(name, ability)| {
            resources
                .definitions
                .insert(name.clone(), house.color, ability.description.clone())
        });
        resources.house_pool.insert_house(house.name, house);
    }

    fn load_rounds(resources: &mut Resources, file: &PathBuf) {
        let json =
            futures::executor::block_on(<String as geng::LoadAsset>::load(&resources.geng, file))
                .expect(&format!("Failed to load Rounds {:?}", file));
        let rounds: Floors =
            serde_json::from_str(&json).expect(&format!("Failed to parse Rounds {:?}", file));
        resources.floors = rounds;
    }
}
