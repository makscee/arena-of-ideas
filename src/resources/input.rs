use super::*;

pub struct Input {
    pub dragged_entity: Option<legion::Entity>,
    pub hovered_entity: Option<legion::Entity>,

    pub down_keys: HashSet<geng::Key>,
    pub pressed_keys: HashSet<geng::Key>,
    pub down_mouse_buttons: HashSet<geng::MouseButton>,
    pub pressed_mouse_buttons: HashSet<geng::MouseButton>,
    pub mouse_pos: vec2<f32>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            dragged_entity: default(),
            hovered_entity: default(),
            down_keys: default(),
            pressed_keys: default(),
            down_mouse_buttons: default(),
            pressed_mouse_buttons: default(),
            mouse_pos: vec2::ZERO,
        }
    }
}
