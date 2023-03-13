use super::*;

#[derive(Default)]
pub struct InputComponent {
    pub hovered: Option<(bool, Time)>,
    pub dragged: Option<(bool, Time)>,
    pub pressed: Option<(bool, Time)>,
}

impl VarsProvider for InputComponent {
    fn extend_vars(&self, vars: &mut Vars, _resources: &Resources) {
        if let Some((value, ts)) = self.hovered {
            vars.insert(VarName::Hovered, Var::Float(value as u8 as f32));
            vars.insert(VarName::HoveredTs, Var::Float(ts));
        }
        if let Some((value, ts)) = self.dragged {
            vars.insert(VarName::Dragged, Var::Float(value as u8 as f32));
            vars.insert(VarName::DraggedTs, Var::Float(ts));
        }
        if let Some((value, ts)) = self.pressed {
            vars.insert(VarName::Pressed, Var::Float(value as u8 as f32));
            vars.insert(VarName::PressedTs, Var::Float(ts));
        }
    }
}

impl InputComponent {
    pub fn is_hovered(&self) -> bool {
        match self.hovered {
            Some(value) => value.0,
            None => false,
        }
    }
    pub fn is_dragged(&self) -> bool {
        match self.dragged {
            Some(value) => value.0,
            None => false,
        }
    }
    pub fn is_pressed(&self) -> bool {
        match self.pressed {
            Some(value) => value.0,
            None => false,
        }
    }
    pub fn set_hovered(&mut self, value: bool, ts: Time) -> bool {
        if let Some(hovered) = &mut self.hovered {
            hovered.0 = value;
            hovered.1 = ts;
            return true;
        }
        false
    }
    pub fn set_dragged(&mut self, value: bool, ts: Time) -> bool {
        if let Some(dragged) = &mut self.dragged {
            dragged.0 = value;
            dragged.1 = ts;
            return true;
        }
        false
    }
    pub fn set_pressed(&mut self, value: bool, ts: Time) -> bool {
        if let Some(pressed) = &mut self.pressed {
            pressed.0 = value;
            pressed.1 = ts;
            return true;
        }
        false
    }
}
