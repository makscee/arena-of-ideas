use std::collections::VecDeque;

use super::*;

mod effect;
mod event;
mod shader_programs;
mod status;
mod trigger;
mod visual_effect;

pub use effect::*;
pub use event::*;
pub use shader_programs::*;
pub use status::*;
pub use trigger::*;
pub use visual_effect::*;

pub struct Resources {
    pub shader_programs: ShaderPrograms,
    pub down_key: Option<geng::Key>,

    pub game_time: Time,
    pub delta_time: Time,
    pub statuses: Statuses,
    pub action_queue: VecDeque<Action>,

    pub unit_templates: HashMap<PathBuf, UnitTemplate>,

    pub camera: geng::Camera2d,
    pub geng: Geng,
}

//todo: async loading
impl Resources {
    pub fn new(geng: &Geng) -> Self {
        let shader_programs = ShaderPrograms::new();
        let camera = geng::Camera2d {
            center: vec2(0.0, 0.0),
            rotation: 0.0,
            fov: 5.0,
        };

        Self {
            shader_programs,
            camera,
            down_key: default(),
            geng: geng.clone(),
            game_time: default(),
            delta_time: default(),
            action_queue: default(),
            statuses: default(),
            unit_templates: default(),
        }
    }

    pub fn load(&mut self, fws: &mut FileWatcherSystem) {
        self.load_shader_programs(fws);
        self.load_unit_templates();
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
            &static_path().join(file),
        ))
        .expect(&format!("Failed to load shader {:?}", file));
        debug!("Load shader program {:?}", file);
        resources
            .shader_programs
            .insert_program(file.clone(), program)
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
}

#[derive(Deserialize, Debug)]
pub struct UnitTemplate(pub Vec<Component>);

impl UnitTemplate {
    pub fn create_unit_entity(
        &self,
        world: &mut legion::World,
        statuses: &mut Statuses,
    ) -> legion::Entity {
        let entity = world.push((Position::default(), FlagsComponent::default()));
        let mut entry = world.entry(entity).unwrap();
        entry.add_component(UnitComponent { entity });
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
