use super::*;

pub struct InputSystem {}

impl InputSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn process_shaders(
        shaders: &mut Vec<Shader>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        Self::update_frame_data(shaders, resources);
        Self::handle_events(shaders, world, resources)
    }

    pub fn update_frame_data<'a>(shaders: &mut Vec<Shader>, resources: &mut Resources) {
        let (prev, cur) = &mut resources.input_data.frame_data;
        mem::swap(prev, cur);
        cur.mouse = resources.input_data.mouse_pos;
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
                cur.state = match prev.mouse == cur.mouse {
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
            if shader.entity.is_some() {
                if let Some(area) = AreaComponent::from_shader(shader) {
                    if area.contains(resources.input_data.mouse_pos) {
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

    pub fn handle_events<'a>(
        shaders: &mut Vec<Shader>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        let (prev, cur) = &resources.input_data.frame_data.clone();
        let mut prev_shader = None;
        let mut cur_shader = None;
        if cur.attention.is_some() || prev.attention.is_some() {
            for (ind, shader) in shaders.iter_mut().enumerate() {
                if let Some(entity) = shader.entity.as_ref() {
                    if let Some(prev) = prev.attention {
                        if prev == *entity {
                            prev_shader = Some(ind);
                        }
                    }
                    if let Some(cur) = cur.attention {
                        if cur == *entity {
                            cur_shader = Some(ind);
                        }
                    }
                }
            }
        }

        match cur.state {
            InputState::None => {}
            InputState::Hover => {
                Self::send_event(InputEvent::Hover, cur_shader, shaders, resources, world)
            }
            InputState::Press => {
                Self::send_event(InputEvent::Press, cur_shader, shaders, resources, world)
            }
            InputState::Click => {
                Self::send_event(InputEvent::Click, cur_shader, shaders, resources, world)
            }
            InputState::Drag => {
                if cur.mouse != prev.mouse {
                    Self::send_event(
                        InputEvent::Drag {
                            delta: cur.mouse - prev.mouse,
                        },
                        cur_shader,
                        shaders,
                        resources,
                        world,
                    );
                }
            }
        }

        if cur.state != prev.state || cur.attention != prev.attention {
            match prev.state {
                InputState::None | InputState::Click => {}
                InputState::Hover => {
                    Self::send_event(
                        InputEvent::HoverStop,
                        prev_shader,
                        shaders,
                        resources,
                        world,
                    );
                }
                InputState::Press => Self::send_event(
                    InputEvent::PressStop,
                    prev_shader,
                    shaders,
                    resources,
                    world,
                ),
                InputState::Drag => {
                    Self::send_event(InputEvent::DragStop, prev_shader, shaders, resources, world)
                }
            }

            match cur.state {
                InputState::None | InputState::Click => {}
                InputState::Hover => Self::send_event(
                    InputEvent::HoverStart,
                    cur_shader,
                    shaders,
                    resources,
                    world,
                ),
                InputState::Press => Self::send_event(
                    InputEvent::PressStart,
                    cur_shader,
                    shaders,
                    resources,
                    world,
                ),
                InputState::Drag => {
                    Self::send_event(InputEvent::DragStart, cur_shader, shaders, resources, world)
                }
            };
        }
    }

    fn send_event(
        event: InputEvent,
        ind: Option<usize>,
        shaders: &mut Vec<Shader>,
        resources: &mut Resources,
        world: &mut legion::World,
    ) {
        if let Some(ind) = ind {
            let mut shader = shaders.remove(ind);
            let entity = shader.entity.unwrap();
            match &event {
                InputEvent::HoverStart
                | InputEvent::HoverStop
                | InputEvent::DragStart
                | InputEvent::DragStop
                | InputEvent::PressStart
                | InputEvent::PressStop
                | InputEvent::Click => {
                    resources
                        .input_data
                        .input_events
                        .insert(entity, (event.clone(), resources.global_time));
                }
                _ => {}
            }
            for f in shader.input_handlers.clone() {
                (f)(event, entity, &mut shader, world, resources);
            }
            shaders.insert(ind, shader);
            match &event {
                InputEvent::HoverStart => resources.input_data.hovered_entity = Some(entity),
                InputEvent::HoverStop => resources.input_data.dragged_entity = None,
                InputEvent::DragStart => resources.input_data.dragged_entity = Some(entity),
                InputEvent::DragStop => resources.input_data.dragged_entity = None,
                _ => {}
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
