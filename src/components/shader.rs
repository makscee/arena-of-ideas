use super::*;

/// Component to link to a shader program with specific parameters
pub struct Shader {
    pub path: PathBuf, // full path
    pub parameters: ShaderParameters,
    pub layer: ShaderLayer,
    pub order: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ShaderLayer {
    Background,
    Unit,
    Vfx,
    UI,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShaderParameters {
    pub vertices: usize,
    pub instances: usize,
    pub parameters: HashMap<String, ShaderParameter>,
}

impl ShaderParameters {
    pub fn new() -> Self {
        Self {
            vertices: 2,
            instances: 1,
            parameters: default(),
        }
    }
}

impl ugli::Uniforms for ShaderParameters {
    fn walk_uniforms<C>(&self, visitor: &mut C)
    where
        C: ugli::UniformVisitor,
    {
        for (name, value) in &self.parameters {
            visitor.visit(name, value);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ShaderParameter {
    Int(i32),
    Float(f32),
    Vec2(Vec2<f32>),
    Vec3(Vec3<f32>),
    Vec4(Vec4<f32>),
    Color(Rgba<f32>),
}

impl ugli::Uniform for ShaderParameter {
    fn apply(&self, gl: &ugli::raw::Context, info: &ugli::UniformInfo) {
        match self {
            Self::Int(value) => value.apply(gl, info),
            Self::Float(value) => value.apply(gl, info),
            Self::Vec2(value) => value.apply(gl, info),
            Self::Vec3(value) => value.apply(gl, info),
            Self::Vec4(value) => value.apply(gl, info),
            Self::Color(value) => value.apply(gl, info),
        }
    }
}
