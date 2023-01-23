use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EcsPosition {
    pub x: f32,
    pub y: f32,
}

/// Marker component
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EcsUnit {}

pub struct EcsShaderProgram {
    pub shader: SystemShader,
    pub parameters: ShaderParameters,
    pub vertices: usize,
    pub instances: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EcsAbility {
    pub effect: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EcsLogicEffect {
    pub effect: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EcsLogicEffectType {
    Noop,
    Damage { value: u32 },
}
