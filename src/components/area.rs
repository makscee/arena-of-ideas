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
            AreaType::Rectangle { size } => vars.insert(VarName::Box, Var::Vec2(size)),
        }
    }
}

impl AreaComponent {
    pub fn new(r#type: AreaType, position: vec2<f32>) -> Self {
        Self { r#type, position }
    }

    pub fn contains(&self, pos: vec2<f32>) -> bool {
        let pos = pos - self.position;
        match self.r#type {
            AreaType::Circle { radius } => pos.len() < radius,
            AreaType::Rectangle { size } => pos.x.abs() < size.x && pos.y.abs() < size.y,
        }
    }

    pub fn from_shader(shader: &Shader) -> Option<Self> {
        shader
            .parameters
            .uniforms
            .get_vec2(&VarName::Position.convert_to_uniform())
            .and_then(|position| {
                if let Some(radius) = shader
                    .parameters
                    .uniforms
                    .get_float(&VarName::Radius.convert_to_uniform())
                {
                    Some(Self {
                        r#type: AreaType::Circle { radius },
                        position,
                    })
                } else if let Some(size) = shader
                    .parameters
                    .uniforms
                    .get_vec2(&VarName::Size.convert_to_uniform())
                {
                    Some(Self {
                        r#type: AreaType::Rectangle { size },
                        position,
                    })
                } else {
                    None
                }
            })
    }
}
