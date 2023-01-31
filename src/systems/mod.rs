use self::time_system::TimeSystem;

use super::*;

mod action_system;
mod file_watcher_system;
mod game_state_system;
mod shader_system;
mod time_system;
mod visual_queue_system;

pub use action_system::*;
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
        let entities = resources
            .unit_templates
            .values()
            .map(|template| template.create_unit_entity(world, &mut resources.statuses))
            .collect_vec();
        Self::init_statuses(resources);

        for _ in 0..10 {
            resources.visual_queue.add_effect(VisualEffect {
                duration: 1.0,
                r#type: VisualEffectType::ShaderAnimation {
                    program: PathBuf::try_from("shaders/vfx/circle.glsl").unwrap(),
                    from: ShaderParameters {
                        parameters: hashmap! {
                            "u_color".to_string() => ShaderParameter::Color(Rgba::RED),
                            "u_scale".to_string() => ShaderParameter::Float(1.5),
                            "u_position".to_string() => ShaderParameter::Vec2(vec2(-0.8, 0.0)),
                        },
                        ..default()
                    },
                    to: ShaderParameters {
                        parameters: hashmap! {
                            "u_color".to_string() => ShaderParameter::Color(Rgba::MAGENTA),
                            "u_scale".to_string() => ShaderParameter::Float(0.5),
                            "u_position".to_string() => ShaderParameter::Vec2(vec2(0.5, 0.0)),
                        },
                        ..default()
                    },
                },
            });
            resources.visual_queue.next_node();
            resources.visual_queue.add_effect(VisualEffect {
                duration: 1.0,
                r#type: VisualEffectType::ShaderAnimation {
                    program: PathBuf::try_from("shaders/vfx/circle.glsl").unwrap(),
                    to: ShaderParameters {
                        parameters: hashmap! {
                            "u_color".to_string() => ShaderParameter::Color(Rgba::RED),
                            "u_scale".to_string() => ShaderParameter::Float(1.5),
                            "u_position".to_string() => ShaderParameter::Vec2(vec2(-0.8, 0.0)),
                        },
                        ..default()
                    },
                    from: ShaderParameters {
                        parameters: hashmap! {
                            "u_color".to_string() => ShaderParameter::Color(Rgba::MAGENTA),
                            "u_scale".to_string() => ShaderParameter::Float(0.5),
                            "u_position".to_string() => ShaderParameter::Vec2(vec2(0.5, 0.0)),
                        },
                        ..default()
                    },
                },
            });
            resources.visual_queue.next_node();
        }
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
}
