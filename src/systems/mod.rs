use self::time_system::TimeSystem;

use super::*;

mod action_system;
mod battle_system;
mod file_watcher_system;
mod game_state_system;
mod shader_system;
mod time_system;
mod visual_queue_system;

pub use action_system::*;
pub use battle_system::*;
pub use file_watcher_system::*;
pub use game_state_system::*;
use geng::prelude::itertools::Itertools;
pub use shader_system::*;
pub use visual_queue_system::*;

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
        systems.push(Box::new(ShaderSystem::new()));
        systems.push(Box::new(fws));
        systems.push(Box::new(TimeSystem::new()));
        systems.push(Box::new(ActionSystem::new()));
        systems.push(Box::new(VisualQueueSystem::new()));
        systems
    }

    pub fn init_world(resources: &mut Resources, world: &mut legion::World) {
        Self::init_field(resources, world);
        Self::init_units(resources, world);
        Self::init_statuses(resources);

        // test effects
        // for _ in 0..10 {
        // resources.visual_queue.add_effect(VisualEffect {
        //     duration: 1.0,
        //     r#type: VisualEffectType::ShaderAnimation {
        //         program: PathBuf::try_from("shaders/vfx/circle.glsl").unwrap(),
        //         parameters: default(),
        //         from: hashmap! {
        //                 "u_color".to_string() => ShaderUniform::Color(Rgba::RED),
        //                 "u_scale".to_string() => ShaderUniform::Float(0.3),
        //                 "u_position".to_string() => ShaderUniform::Vec2(vec2(-0.8, 0.0)),

        //         }
        //         .into(),
        //         to: hashmap! {
        //             "u_color".to_string() => ShaderUniform::Color(Rgba::MAGENTA),
        //             "u_scale".to_string() => ShaderUniform::Float(0.1),
        //             "u_position".to_string() => ShaderUniform::Vec2(vec2(0.5, 0.0)),
        //         }
        //         .into(),
        //     },
        // });
        // resources.visual_queue.add_effect(VisualEffect {
        //     duration: 1.5,
        //     r#type: VisualEffectType::EntityShaderAnimation {
        //         entity: entities[0],
        //         from: hashmap! {"u_position".to_string() => ShaderUniform::Vec2(vec2(0.0,0.0))}
        //             .into(),
        //         to: hashmap! {"u_position".to_string() => ShaderUniform::Vec2(vec2(0.0,1.0))}
        //             .into(),
        //     },
        // });
        // resources.visual_queue.next_node();
        // resources.visual_queue.add_effect(VisualEffect {
        //     duration: 1.0,
        //     r#type: VisualEffectType::ShaderAnimation {
        //         program: PathBuf::try_from("shaders/vfx/circle.glsl").unwrap(),
        //         parameters: default(),
        //         to: hashmap! {
        //                 "u_color".to_string() => ShaderUniform::Color(Rgba::RED),
        //                 "u_scale".to_string() => ShaderUniform::Float(0.3),
        //                 "u_position".to_string() => ShaderUniform::Vec2(vec2(-0.8, 0.0)),

        //         }
        //         .into(),
        //         from: hashmap! {
        //             "u_color".to_string() => ShaderUniform::Color(Rgba::MAGENTA),
        //             "u_scale".to_string() => ShaderUniform::Float(0.1),
        //             "u_position".to_string() => ShaderUniform::Vec2(vec2(0.5, 0.0)),
        //         }
        //         .into(),
        //     },
        // });
        // resources.visual_queue.next_node();
        // }
    }

    fn init_units(resources: &mut Resources, world: &mut legion::World) {
        let left = resources.unit_templates.values().collect_vec()[0].create_unit_entity(
            world,
            &mut resources.statuses,
            Faction::Light,
        );
        let mut left = world.entry(left).unwrap();
        left.get_component_mut::<Position>().unwrap().0 = vec2(-1.0, 0.0);

        let right = resources.unit_templates.values().collect_vec()[1].create_unit_entity(
            world,
            &mut resources.statuses,
            Faction::Dark,
        );
        let mut right = world.entry(right).unwrap();
        right.get_component_mut::<Position>().unwrap().0 = vec2(1.0, 0.0);
    }

    fn init_statuses(resources: &mut Resources) {
        let statuses = resources
            .statuses
            .active_statuses
            .iter()
            .map(|(entity, map)| (entity.clone(), map.clone()))
            .collect_vec();
        statuses.iter().for_each(|(_entity, statuses)| {
            statuses.iter().for_each(|(status, context)| {
                Event::Init {
                    status: status.to_string(),
                }
                .send(&context.clone(), resources)
                .expect("Error on status Init");
            })
        })
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
