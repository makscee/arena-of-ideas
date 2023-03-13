use super::*;

#[derive(Clone, Debug)]
pub struct AreaComponent {
    pub r#type: AreaType,
    pub position: vec2<f32>,
}

#[derive(Clone, Debug)]
pub enum AreaType {
    Circle { radius: f32 },
    Rectangle { size: vec2<f32> },
}

impl VarsProvider for AreaComponent {
    fn extend_vars(&self, vars: &mut Vars, _resources: &Resources) {
        vars.insert(VarName::Position, Var::Vec2(self.position));
        match self.r#type {
            AreaType::Circle { radius } => vars.insert(VarName::Radius, Var::Float(radius)),
            AreaType::Rectangle { size } => vars.insert(VarName::Size, Var::Vec2(size)),
        }
    }
}

impl AreaComponent {
    pub fn contains(&self, pos: vec2<f32>) -> bool {
        let pos = pos - self.position;
        match self.r#type {
            AreaType::Circle { radius } => pos.len() < radius,
            AreaType::Rectangle { size } => pos.x.abs() < size.x && pos.y.abs() < size.y,
        }
    }
}
