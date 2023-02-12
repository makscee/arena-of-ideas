use std::{collections::VecDeque, rc::Rc};

use super::*;

mod cassette;
mod effect;
mod event;
mod options;
mod shader_programs;
mod shop;
mod status;
mod trigger;
mod visual_effect;

pub use cassette::*;
pub use effect::*;
pub use event::*;
pub use options::*;
pub use shader_programs::*;
pub use shop::*;
pub use status::*;
pub use trigger::*;
pub use visual_effect::*;

pub struct Resources {
    pub options: Options,

    pub shader_programs: ShaderPrograms,
    pub down_keys: HashSet<geng::Key>,
    pub pressed_keys: HashSet<geng::Key>,
    pub down_mouse_buttons: HashSet<geng::MouseButton>,
    pub pressed_mouse_buttons: HashSet<geng::MouseButton>,
    pub mouse_pos: vec2<f32>,

    pub game_time: Time,
    pub delta_time: Time,
    pub statuses: Statuses,
    pub action_queue: VecDeque<Action>,
    pub cassette: Cassette,
    pub shop: Shop,
    pub frame_shaders: Vec<Shader>,
    pub dragged_entity: Option<legion::Entity>,

    pub unit_templates: HashMap<PathBuf, UnitTemplate>,

    pub camera: geng::Camera2d,
    pub fonts: Vec<Rc<geng::font::Ttf>>,
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
                        pixel_size: 64.0,
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
                        pixel_size: 64.0,
                        max_distance: 0.25,
                    },
                )
                .unwrap(),
            ),
        ];

        Self {
            shader_programs,
            camera,
            fonts,
            down_keys: default(),
            pressed_keys: default(),
            geng: geng.clone(),
            game_time: default(),
            delta_time: default(),
            action_queue: default(),
            statuses: default(),
            unit_templates: default(),
            cassette: default(),
            shop: default(),
            frame_shaders: default(),
            options,
            down_mouse_buttons: default(),
            pressed_mouse_buttons: default(),
            mouse_pos: vec2::ZERO,
            dragged_entity: default(),
        }
    }

    pub fn load(&mut self, fws: &mut FileWatcherSystem) {
        self.load_shader_programs(fws);
        self.load_unit_templates();
        self.shop.load(&self.unit_templates);
        fws.load_and_watch_file(
            self,
            &static_path().join("options.json"),
            Box::new(Self::load_options),
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

    fn load_unit_templates(&mut self) {
        let list = futures::executor::block_on(Self::load_list(
            &self.geng,
            PathBuf::try_from("units/_list.json").unwrap(),
        ));
        self.unit_templates.extend(
            list.iter()
                .map(|file| (file.clone(), Self::load_unit_template(&self.geng, file))),
        );
    }

    fn load_unit_template(geng: &Geng, file: &PathBuf) -> UnitTemplate {
        let json = futures::executor::block_on(<String as geng::LoadAsset>::load(
            geng,
            &static_path().join(file),
        ))
        .expect("Failed to load unit");
        serde_json::from_str(&json).expect("Failed to parse UnitTemplate")
    }

    fn load_options(resources: &mut Resources, file: &PathBuf) {
        resources.options =
            futures::executor::block_on(<Options as geng::LoadAsset>::load(&resources.geng, &file))
                .unwrap();
        dbg!(&resources.options);
    }
}

#[derive(Deserialize, Debug)]
pub struct UnitTemplate(pub Vec<SerializedComponent>);

impl UnitTemplate {
    pub fn create_unit_entity(
        &self,
        world: &mut legion::World,
        statuses: &mut Statuses,
        faction: Faction,
        slot: usize,
        position: vec2<f32>,
    ) -> legion::Entity {
        let entity = world.push((
            Position(position),
            FlagsComponent::default(),
            faction.clone(),
            UnitComponent { faction, slot },
        ));
        let mut entry = world.entry(entity).unwrap();
        entry.add_component(EntityComponent { entity });
        let mut context = Context {
            owner: entity,
            target: entity,
            creator: entity,
            vars: default(),
            status: default(),
        };
        self.0.iter().for_each(|component| {
            component.add_to_entry(&mut entry, &entity, statuses, &mut context)
        });
        debug!(
            "Unit#{:?} created. Template: {:?} Context: {:?}",
            entity, self.0, context
        );
        entry.add_component(context);
        entity
    }
}
