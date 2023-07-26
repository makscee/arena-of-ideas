use super::*;

pub struct InputSystem {}

const DRAG_THRESHOLD: f32 = 0.1;
impl InputSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn process_shaders(
        shaders: &mut Vec<ShaderChain>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        Self::update_frame_data(shaders, resources);
        Self::handle_events(shaders, world, resources)
    }

    pub fn update_frame_data(shaders: &mut Vec<ShaderChain>, resources: &mut Resources) {
        let (prev, cur) = &mut resources.input_data.frame_data;
        mem::swap(prev, cur);
        cur.mouse = resources.input_data.mouse_world_pos;
        if resources
            .input_data
            .pressed_mouse_buttons
            .contains(&geng::MouseButton::Left)
        {
            if prev.state == InputState::Drag {
                cur.state = InputState::Drag;
                cur.attention = prev.attention.clone();
                return;
            }
            if prev.state == InputState::Press {
                cur.state = match (prev.mouse - cur.mouse).len() < DRAG_THRESHOLD {
                    true => InputState::Press,
                    false => InputState::Drag,
                };
                cur.attention = prev.attention.clone();
                return;
            }
        }

        if resources
            .input_data
            .down_mouse_buttons
            .contains(&geng::MouseButton::Left)
        {
            if prev.state == InputState::Hover {
                cur.state = InputState::Press;
                cur.attention = prev.attention.clone();
                return;
            }
        }

        if resources
            .input_data
            .up_mouse_buttons
            .contains(&geng::MouseButton::Left)
        {
            if prev.state == InputState::Press {
                cur.state = InputState::Click;
                cur.attention = prev.attention.clone();
                return;
            }
            if prev.state == InputState::Drag {
                cur.state = InputState::Hover;
                cur.attention = prev.attention.clone();
                return;
            }
        }
        cur.attention = None;
        cur.state = InputState::None;

        let mut hovered = None;
        for shader in shaders.iter().rev() {
            for shader in shader.iter().rev() {
                if shader.entity.is_some() {
                    if shader.is_hovered(
                        resources.input_data.mouse_screen_pos,
                        resources.input_data.mouse_world_pos,
                    ) {
                        hovered = Some(shader);
                        break;
                    }
                }
            }
        }

        if let Some(hovered) = hovered {
            cur.attention = hovered.entity;
            cur.state = InputState::Hover;
        }
    }

    pub fn handle_events(
        shaders: &mut Vec<ShaderChain>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        let (prev, cur) = &resources.input_data.frame_data.clone();

        if let Some(shader_entity) = cur.attention {
            match cur.state {
                InputState::None => {}
                InputState::Hover => {
                    Self::send_event(HandleEvent::Hover, shader_entity, shaders, resources, world)
                }
                InputState::Press => {
                    Self::send_event(HandleEvent::Press, shader_entity, shaders, resources, world)
                }
                InputState::Click => {
                    Self::send_event(HandleEvent::Click, shader_entity, shaders, resources, world)
                }
                InputState::Drag => {
                    if cur.mouse != prev.mouse {
                        Self::send_event(
                            HandleEvent::Drag {
                                delta: cur.mouse - prev.mouse,
                            },
                            shader_entity,
                            shaders,
                            resources,
                            world,
                        );
                    }
                }
            }
        }

        if cur.state != prev.state || cur.attention != prev.attention {
            if let Some(shader_entity) = prev.attention {
                match prev.state {
                    InputState::None | InputState::Click => {}
                    InputState::Hover => {
                        Self::send_event(
                            HandleEvent::HoverStop,
                            shader_entity,
                            shaders,
                            resources,
                            world,
                        );
                    }
                    InputState::Press => Self::send_event(
                        HandleEvent::PressStop,
                        shader_entity,
                        shaders,
                        resources,
                        world,
                    ),
                    InputState::Drag => Self::send_event(
                        HandleEvent::DragStop,
                        shader_entity,
                        shaders,
                        resources,
                        world,
                    ),
                }
            }

            if let Some(shader_entity) = cur.attention {
                match cur.state {
                    InputState::None | InputState::Click => {}
                    InputState::Hover => Self::send_event(
                        HandleEvent::HoverStart,
                        shader_entity,
                        shaders,
                        resources,
                        world,
                    ),
                    InputState::Press => Self::send_event(
                        HandleEvent::PressStart,
                        shader_entity,
                        shaders,
                        resources,
                        world,
                    ),
                    InputState::Drag => Self::send_event(
                        HandleEvent::DragStart,
                        shader_entity,
                        shaders,
                        resources,
                        world,
                    ),
                };
            }
        }
    }

    fn send_event(
        event: HandleEvent,
        shader_entity: legion::Entity,
        shaders: &mut Vec<ShaderChain>,
        resources: &mut Resources,
        world: &mut legion::World,
    ) {
        for shader in shaders.iter_mut() {
            for shader in shader.iter_mut() {
                if let Some(entity) = shader.entity {
                    if entity != shader_entity {
                        continue;
                    }
                    match &event {
                        HandleEvent::HoverStart
                        | HandleEvent::HoverStop
                        | HandleEvent::DragStart
                        | HandleEvent::DragStop
                        | HandleEvent::PressStart
                        | HandleEvent::PressStop
                        | HandleEvent::Click => {
                            resources
                                .input_data
                                .input_events
                                .insert(entity, (event.clone(), resources.global_time));
                        }
                        _ => {}
                    }
                    for f in shader.input_handlers.clone() {
                        (f)(event, entity, shader, world, resources);
                    }
                    match &event {
                        HandleEvent::HoverStart => {
                            resources.input_data.hovered_entity = Some(entity);
                            for (color, title, text) in shader.hover_hints.iter() {
                                PanelsSystem::open_hint(color.clone(), title, text, resources);
                            }
                        }
                        HandleEvent::HoverStop => {
                            resources.input_data.hovered_entity = None;
                            PanelsSystem::close_hints(resources);
                        }
                        HandleEvent::DragStart => {
                            resources.input_data.dragged_entity = Some(entity)
                        }
                        HandleEvent::DragStop => resources.input_data.dragged_entity = None,
                        _ => {}
                    }
                }
            }
        }
    }
}

impl System for InputSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        let mut shaders = mem::take(&mut resources.prepared_shaders);
        Self::process_shaders(&mut shaders, world, resources);
        resources.prepared_shaders = shaders;
    }
}
