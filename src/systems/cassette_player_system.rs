use super::*;

pub struct CassettePlayerSystem {
    hidden: bool,
}

struct CassettePlayerButtonComponent {}

impl CassettePlayerSystem {
    pub fn new(hidden: bool) -> Self {
        Self { hidden }
    }

    pub fn init_world(world: &mut legion::World, resources: &mut Resources) {
        <(&EntityComponent, &CassettePlayerButtonComponent)>::query()
            .iter(world)
            .map(|(entity, _)| entity.entity)
            .collect_vec()
            .iter()
            .for_each(|entity| {
                resources.input.listeners.remove(entity);
                world.remove(*entity);
            });
        fn play(
            entity: legion::Entity,
            resources: &mut Resources,
            world: &mut legion::World,
            event: InputEvent,
        ) {
            match event {
                InputEvent::Click => match resources.current_state {
                    GameState::Shop => {
                        ButtonSystem::change_icon(
                            entity,
                            world,
                            &resources.options.images.pause_icon,
                        );
                        ButtonSystem::change_icon_color(
                            entity,
                            world,
                            resources.options.colors.cassette_player_btn_active,
                        );
                        resources.transition_state = GameState::Battle;
                    }
                    GameState::Battle => {
                        resources.cassette_play_mode = match resources.cassette_play_mode {
                            CassettePlayMode::Play => {
                                ButtonSystem::change_icon(
                                    entity,
                                    world,
                                    &resources.options.images.play_icon,
                                );
                                ButtonSystem::change_icon_color(
                                    entity,
                                    world,
                                    resources.options.colors.cassette_player_btn_normal,
                                );
                                CassettePlayMode::Stop
                            }
                            CassettePlayMode::Stop | CassettePlayMode::Rewind { .. } => {
                                ButtonSystem::change_icon(
                                    entity,
                                    world,
                                    &resources.options.images.pause_icon,
                                );
                                ButtonSystem::change_icon_color(
                                    entity,
                                    world,
                                    resources.options.colors.cassette_player_btn_active,
                                );
                                CassettePlayMode::Play
                            }
                        }
                    }
                    _ => {}
                },
                InputEvent::HoverStart => ButtonSystem::change_icon_color(
                    entity,
                    world,
                    resources.options.colors.cassette_player_btn_hovered,
                ),
                InputEvent::HoverStop => ButtonSystem::change_icon_color(
                    entity,
                    world,
                    resources.options.colors.cassette_player_btn_normal,
                ),
                _ => {}
            }
        }

        fn rewind(
            entity: legion::Entity,
            resources: &mut Resources,
            world: &mut legion::World,
            direction: f32,
            event: InputEvent,
        ) {
            match event {
                InputEvent::Press => {
                    match resources.current_state {
                        GameState::Battle => {}
                        _ => {
                            return;
                        }
                    }
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
                InputEvent::HoverStart => ButtonSystem::change_icon_color(
                    entity,
                    world,
                    resources.options.colors.cassette_player_btn_hovered,
                ),
                InputEvent::HoverStop => ButtonSystem::change_icon_color(
                    entity,
                    world,
                    resources.options.colors.cassette_player_btn_normal,
                ),
                _ => {}
            }
        }

        fn rewind_backward(
            entity: legion::Entity,
            resources: &mut Resources,
            world: &mut legion::World,
            event: InputEvent,
        ) {
            rewind(entity, resources, world, -1.0, event);
        }

        fn rewind_forward(
            entity: legion::Entity,
            resources: &mut Resources,
            world: &mut legion::World,
            event: InputEvent,
        ) {
            rewind(entity, resources, world, 1.0, event);
        }
        let world_entity = WorldSystem::get_context(world).owner;
        let mut buttons = vec![];
        buttons.push(ButtonSystem::create_button(
            world,
            world_entity,
            resources,
            match resources.current_state {
                GameState::Battle => resources.options.images.pause_icon.clone(),
                _ => resources.options.images.play_icon.clone(),
            },
            match resources.current_state {
                GameState::Battle => resources.options.colors.cassette_player_btn_active,
                _ => resources.options.colors.cassette_player_btn_normal,
            },
            play,
            BATTLEFIELD_POSITION + vec2(0.0, -3.0),
            &default(),
        ));
        match resources.current_state {
            GameState::Battle => {
                buttons.push(ButtonSystem::create_button(
                    world,
                    world_entity,
                    resources,
                    resources.options.images.rewind_forward_icon.clone(),
                    resources.options.colors.cassette_player_btn_normal,
                    rewind_forward,
                    BATTLEFIELD_POSITION + vec2(2.5, -3.0),
                    &hashmap! {
                        "u_scale" => ShaderUniform::Float(0.7),
                    }
                    .into(),
                ));
                buttons.push(ButtonSystem::create_button(
                    world,
                    world_entity,
                    resources,
                    resources.options.images.rewind_backward_icon.clone(),
                    resources.options.colors.cassette_player_btn_normal,
                    rewind_backward,
                    BATTLEFIELD_POSITION + vec2(-2.5, -3.0),
                    &hashmap! {
                        "u_scale" => ShaderUniform::Float(0.7),
                    }
                    .into(),
                ));
            }
            _ => {}
        }

        for button in buttons {
            world
                .entry(button)
                .unwrap()
                .add_component(CassettePlayerButtonComponent {});
        }
    }
}

const REWIND_SPEED: f32 = 5.0;

impl System for CassettePlayerSystem {
    fn update(&mut self, _world: &mut legion::World, resources: &mut Resources) {
        resources.cassette.head = match resources.cassette_play_mode {
            CassettePlayMode::Play => resources.cassette.head + resources.delta_time,
            CassettePlayMode::Stop => resources.cassette.head,
            CassettePlayMode::Rewind { ts } => (resources.cassette.head
                + (ts - resources.cassette.head) * resources.delta_time * REWIND_SPEED)
                .clamp(0.01, resources.cassette.length()),
        };
        if self.hidden {
            return;
        }
        match resources.current_state {
            GameState::Battle => resources.frame_shaders.push(resources
                .options
                .shaders
                .battle_timer
                .clone()
                .merge_uniforms(&hashmap! {
                    "u_text" => ShaderUniform::String((0, format!("{:.2}", resources.cassette.head))),
                }.into(), true)),
            _ => {}
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
}

#[derive(Clone, Copy, PartialEq)]
pub enum CassettePlayMode {
    Play,
    Stop,
    Rewind { ts: Time },
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
            }
        )
    }
}
