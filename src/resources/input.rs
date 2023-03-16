use super::*;

pub struct Input {
    pub cur_hovered: Option<legion::Entity>,
    pub prev_hovered: Option<legion::Entity>,
    pub hover_data: HashMap<legion::Entity, (bool, Time)>,

    pub cur_dragged: Option<legion::Entity>,

    pub listeners:
        HashMap<legion::Entity, fn(legion::Entity, &mut Resources, &mut legion::World, InputEvent)>,

    pub down_keys: HashSet<geng::Key>,
    pub pressed_keys: HashSet<geng::Key>,
    pub down_mouse_buttons: HashSet<geng::MouseButton>,
    pub up_mouse_buttons: HashSet<geng::MouseButton>,
    pub pressed_mouse_buttons: HashSet<geng::MouseButton>,
    pub mouse_pos: vec2<f32>,
    pub drag_start_pos: Option<vec2<f32>>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            listeners: default(),
            down_keys: default(),
            pressed_keys: default(),
            down_mouse_buttons: default(),
            pressed_mouse_buttons: default(),
            up_mouse_buttons: default(),
            mouse_pos: vec2::ZERO,
            drag_start_pos: default(),
            hover_data: default(),
            cur_hovered: default(),
            prev_hovered: default(),
            cur_dragged: default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    HoverStart,
    Hover,
    HoverStop,
    DragStart,
    Drag,
    DragStop,
    PressStart,
    Press,
    PressStop,
    Click,
}
