use std::{collections::VecDeque, rc::Rc};

use super::*;

mod ability;
mod cassette;
mod condition;
mod effect;
mod event;
mod expression;
mod fonts;
mod house;
mod options;
mod rounds;
mod shader_programs;
mod shop;
mod status;
mod trigger;
mod visual_effect;

pub use ability::*;
pub use cassette::*;
pub use condition::*;
pub use effect::*;
pub use event::*;
pub use expression::*;
pub use fonts::*;
pub use house::*;
pub use options::*;
pub use rounds::*;
pub use shader_programs::*;
pub use shop::*;
pub use status::*;
pub use trigger::*;
pub use visual_effect::*;

pub struct Resources {
    pub options: Options,
    pub reload_triggered: bool,

    pub shader_programs: ShaderPrograms,
    pub down_keys: HashSet<geng::Key>,
    pub pressed_keys: HashSet<geng::Key>,
    pub down_mouse_buttons: HashSet<geng::MouseButton>,
    pub pressed_mouse_buttons: HashSet<geng::MouseButton>,
    pub mouse_pos: vec2<f32>,

    pub game_time: Time,
    pub delta_time: Time,
    pub status_pool: StatusPool,
    pub action_queue: VecDeque<Action>,
    pub cassette: Cassette,
    pub shop: Shop,
    pub frame_shaders: Vec<Shader>,
    pub dragged_entity: Option<legion::Entity>,
    pub hovered_entity: Option<legion::Entity>,
    pub game_won: bool,
    pub last_round: usize,

    pub unit_templates: UnitTemplatesPool,
    pub rounds: Rounds,
    pub houses: HashMap<HouseName, House>,

    pub current_state: GameState,
    pub transition_state: GameState,

    pub camera: geng::Camera2d,
    pub fonts: Fonts,
    pub geng: Geng,
}

//todo: async loading
impl Resources {
    pub fn new(geng: &Geng) -> Self {
        let shader_programs = ShaderPrograms::new();

        // todo: load all Resources as geng::Assets
        let options = futures::executor::block_on(<Options as geng::LoadAsset>::load(
            geng,
            &static_path().join("options.json"),
        ))
        .unwrap();
        let camera = geng::Camera2d {
            center: vec2(0.0, 0.0),
            rotation: 0.0,
            fov: options.fov,
        };
        let fonts = vec![
            Rc::new(
                geng::font::Ttf::new(
                    geng,
                    include_bytes!("../../static/font/stats.ttf"),
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
                    include_bytes!("../../static/font/description.ttf"),
                    geng::font::ttf::Options {
                        pixel_size: 32.0,
                        max_distance: 0.25,
                    },
                )
                .unwrap(),
            ),
        ];

        Self {
            shader_programs,
            camera,
            fonts: Fonts::new(fonts),
            down_keys: default(),
            pressed_keys: default(),
            geng: geng.clone(),
            game_time: default(),
            delta_time: default(),
            action_queue: default(),
            status_pool: default(),
            unit_templates: default(),
            cassette: default(),
            shop: default(),
            frame_shaders: default(),
            options,
            down_mouse_buttons: default(),
            pressed_mouse_buttons: default(),
            mouse_pos: vec2::ZERO,
            dragged_entity: default(),
            hovered_entity: default(),
            houses: default(),
            rounds: default(),
            reload_triggered: default(),
            game_won: default(),
            last_round: default(),
            current_state: GameState::MainMenu,
            transition_state: GameState::MainMenu,
        }
    }

    pub fn load(&mut self, fws: &mut FileWatcherSystem) {
        self.load_shader_programs(fws);
        self.load_houses(fws);
        self.load_unit_templates(fws);
        self.shop.load(&self.unit_templates);
        fws.load_and_watch_file(
            self,
            &static_path().join("options.json"),
            Box::new(Self::load_options),
        );
        fws.load_and_watch_file(
            self,
            &static_path().join("rounds.json"),
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

    fn load_unit_templates(&mut self, fws: &mut FileWatcherSystem) {
        let list = futures::executor::block_on(Self::load_list(
            &self.geng,
            PathBuf::try_from("units/_list.json").unwrap(),
        ));

        for file in list {
            fws.load_and_watch_file(
                self,
                &static_path().join(file),
                Box::new(Self::load_unit_template),
            );
        }
    }

    fn load_unit_template(resources: &mut Resources, file: &PathBuf) {
        let json =
            futures::executor::block_on(<String as geng::LoadAsset>::load(&resources.geng, file))
                .expect("Failed to load unit");
        let template = serde_json::from_str(&json).expect("Failed to parse UnitTemplate");
        resources
            .unit_templates
            .define_template(file.clone(), template);
    }

    fn load_options(resources: &mut Resources, file: &PathBuf) {
        resources.options =
            futures::executor::block_on(<Options as geng::LoadAsset>::load(&resources.geng, &file))
                .unwrap();
        dbg!(&resources.options);
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
        resources.houses.insert(house.name, house);
    }

    fn load_rounds(resources: &mut Resources, file: &PathBuf) {
        let json =
            futures::executor::block_on(<String as geng::LoadAsset>::load(&resources.geng, file))
                .expect(&format!("Failed to load Rounds {:?}", file));
        let rounds: Rounds =
            serde_json::from_str(&json).expect(&format!("Failed to parse Rounds {:?}", file));
        resources.rounds = rounds;
    }
}

#[derive(Default, Debug)]
pub struct UnitTemplatesPool {
    pub heroes: HashMap<PathBuf, UnitTemplate>,
    pub enemies: HashMap<PathBuf, UnitTemplate>,
}

impl UnitTemplatesPool {
    pub fn define_template(&mut self, path: PathBuf, template: UnitTemplate) {
        match template.0.iter().any(|component| match component {
            SerializedComponent::House { houses: _ } => true,
            _ => false,
        }) {
            true => {
                self.heroes.insert(path, template);
            }
            false => {
                self.enemies.insert(path, template);
            }
        }
    }

    pub fn create_unit_entity(
        path: &PathBuf,
        resources: &mut Resources,
        world: &mut legion::World,
        faction: Faction,
        slot: usize,
        position: vec2<f32>,
    ) -> legion::Entity {
        let template = resources
            .unit_templates
            .heroes
            .get(path)
            .or_else(|| resources.unit_templates.enemies.get(path))
            .expect(&format!("Template not found: {:?}", path))
            .clone();
        let entity = world.push((
            PositionComponent(position),
            RadiusComponent(1.0),
            FlagsComponent::default(),
            faction.clone(),
            UnitComponent { faction, slot },
            AttentionComponent::default(),
        ));
        let parent = WorldSystem::get_context(world).owner;
        let mut entry = world.entry(entity).unwrap();
        entry.add_component(EntityComponent { entity });
        let context = Context {
            owner: entity,
            target: entity,
            parent: Some(parent),
            vars: default(),
        };
        entry.add_component(context.clone());
        template
            .0
            .iter()
            .for_each(|component| component.add_to_entry(resources, path, entity, &context, world));
        debug!(
            "Unit#{:?} created. Template: {:?} Context: {:?}",
            entity, template.0, context
        );
        entity
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct UnitTemplate(pub Vec<SerializedComponent>);
