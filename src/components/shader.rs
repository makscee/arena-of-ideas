use super::*;

/// Component to link to a shader program with specific parameters
pub struct Shader {
    pub path: PathBuf, // full path
    pub parameters: ShaderParameters,
    pub layer: ShaderLayer,
    pub order: i32,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ShaderLayer {
    Background,
    Unit,
    Vfx,
    UI,
}

#[derive(Debug, Clone, Deserialize, Default)]
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

impl ShaderParameters {
    pub fn mix(a: &Self, b: &Self, t: f32) -> Self {
        let mut result = Self::new();
        result.instances = (a.instances + b.instances) * t as usize;
        result.vertices = (a.vertices + b.vertices) * t as usize;
        for (key, value) in a.parameters.iter() {
            let a = value;
            let b = b
                .parameters
                .get(key)
                .expect(&format!("Parameter {} absent in B", key));
            match (a, b) {
                (ShaderParameter::Int(a), ShaderParameter::Int(b)) => {
                    result
                        .parameters
                        .insert(key.clone(), ShaderParameter::Int((a + b) * t as i32));
                }
                (ShaderParameter::Float(a), ShaderParameter::Float(b)) => {
                    result
                        .parameters
                        .insert(key.clone(), ShaderParameter::Float((a + b) * t));
                }
                (ShaderParameter::Vec2(a), ShaderParameter::Vec2(b)) => {
                    result
                        .parameters
                        .insert(key.clone(), ShaderParameter::Vec2((*a + *b) * t));
                }
                (ShaderParameter::Vec3(a), ShaderParameter::Vec3(b)) => {
                    result
                        .parameters
                        .insert(key.clone(), ShaderParameter::Vec3((*a + *b) * t));
                }
                (ShaderParameter::Vec4(a), ShaderParameter::Vec4(b)) => {
                    result
                        .parameters
                        .insert(key.clone(), ShaderParameter::Vec4((*a + *b) * t));
                }
                (ShaderParameter::Color(a), ShaderParameter::Color(b)) => {
                    result.parameters.insert(
                        key.clone(),
                        ShaderParameter::Color(Rgba::new(
                            (a.r + b.r) * t,
                            (a.g + b.g) * t,
                            (a.b + b.b) * t,
                            (a.a + b.a) * t,
                        )),
                    );
                }
                _ => panic!(
                    "Failed to mix: not matching parameter types: {:?} {:?}",
                    a, b
                ),
            }
        }

        result
    }
}

#[derive(Debug, Clone, Deserialize)]
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
