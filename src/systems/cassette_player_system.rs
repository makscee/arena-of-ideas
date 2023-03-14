use super::*;

use geng::ui::*;

pub struct CassettePlayerSystem {
    hidden: bool,
}

impl CassettePlayerSystem {
    pub fn new(hidden: bool) -> Self {
        Self { hidden }
    }

    pub fn init_world(
        world_entity: legion::Entity,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        fn play(entity: legion::Entity, resources: &mut Resources, _: &mut legion::World) {
            resources.cassette_play_mode = CassettePlayMode::Play;
        }
        fn stop(entity: legion::Entity, resources: &mut Resources, _: &mut legion::World) {
            resources.cassette_play_mode = CassettePlayMode::Stop;
        }

        let entity = world.push((
            ButtonComponent::new(play),
            AreaComponent {
                r#type: AreaType::Rectangle {
                    size: vec2(1.0, 1.0),
                },
                position: BATTLEFIELD_POSITION - vec2(0.0, 2.0),
            },
            InputComponent {
                hovered: Some(default()),
                dragged: None,
                pressed: Some(default()),
            },
            resources
                .options
                .shaders
                .icon
                .clone()
                .set_uniform(
                    "u_texture",
                    ShaderUniform::Texture(resources.options.images.money_icon.clone()),
                )
                .set_uniform("u_icon_color", ShaderUniform::Color(Rgba::MAGENTA)),
        ));
        let mut entry = world.entry(entity).unwrap();
        entry.add_component(EntityComponent { entity });
        entry.add_component(Context {
            owner: entity,
            target: entity,
            parent: Some(world_entity),
            vars: default(),
        });
    }
}

const REWIND_SPEED: f32 = 5.0;

impl System for CassettePlayerSystem {
    fn update(&mut self, _world: &mut legion::World, resources: &mut Resources) {
        resources.cassette.head = match resources.cassette_play_mode {
            CassettePlayMode::Play | CassettePlayMode::Hidden => {
                resources.cassette.head + resources.delta_time
            }
            CassettePlayMode::Stop => resources.cassette.head,
            CassettePlayMode::Rewind { ts } => (resources.cassette.head
                + (ts - resources.cassette.head) * resources.delta_time * REWIND_SPEED)
                .clamp(0.01, resources.cassette.length()),
        };
        if self.hidden {
            return;
        }

        if resources.input.down_keys.contains(&geng::Key::Space) {
            resources.cassette_play_mode = match resources.cassette_play_mode {
                CassettePlayMode::Play => CassettePlayMode::Stop,
                CassettePlayMode::Stop | CassettePlayMode::Rewind { .. } => CassettePlayMode::Play,
                _ => panic!("Wrong Play Mode"),
            };
        }
        if resources.input.pressed_keys.contains(&geng::Key::Left)
            || resources.input.pressed_keys.contains(&geng::Key::Right)
        {
            let direction = if resources.input.pressed_keys.contains(&geng::Key::Right) {
                1.0
            } else {
                -1.0
            };
            resources.cassette_play_mode = CassettePlayMode::Rewind {
                ts: match resources.cassette_play_mode {
                    CassettePlayMode::Play | CassettePlayMode::Stop => {
                        resources.cassette.head + resources.delta_time * direction
                    }

                    CassettePlayMode::Rewind { ts } => {
                        ts + resources.delta_time * direction * REWIND_SPEED
                    }
                    _ => panic!("Wrong Play Mode"),
                }
                .clamp(0.0, resources.cassette.length()),
            };
        }
    }

    fn ui<'a>(
        &'a mut self,
        cx: &'a ui::Controller,
        world: &'a legion::World,
        resources: &'a Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        if resources.cassette_play_mode == CassettePlayMode::Hidden {
            return Box::new(ui::Void);
        }
        Box::new(
            (
                Text::new(
                    format!("{:.2}", resources.cassette.head),
                    resources.fonts.get_font(0),
                    64.0,
                    Rgba::BLACK,
                ),
                Text::new(
                    format!("Mode {}", resources.cassette_play_mode),
                    resources.fonts.get_font(0),
                    64.0,
                    Rgba::BLACK,
                ),
            )
                .column()
                .align(vec2(0.0, 0.0))
                .fixed_size(vec2(200.0, 60.0))
                .uniform_padding(16.0),
        )
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum CassettePlayMode {
    Play,
    Stop,
    Rewind { ts: Time },
    Hidden,
}

impl Display for CassettePlayMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CassettePlayMode::Play => "Play",
                CassettePlayMode::Stop => "Stop",
                CassettePlayMode::Rewind { .. } => "Rewind",
                CassettePlayMode::Hidden => "Hidden",
            }
        )
    }
}
