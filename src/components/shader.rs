use super::*;

/// Component to link to a shader program with specific parameters
#[derive(Debug, Clone, Deserialize)]
pub struct Shader {
    pub path: PathBuf, // static path
    #[serde(default)]
    pub parameters: ShaderParameters,
    #[serde(default)]
    pub layer: ShaderLayer,
    #[serde(default)]
    pub order: i32,
    pub chain: Option<Box<Vec<Shader>>>,
}

impl Shader {
    pub fn set_uniform(mut self, key: &str, value: ShaderUniform) -> Shader {
        self.parameters.uniforms.insert(String::from(key), value);
        self
    }

    pub fn merge_uniforms(mut self, uniforms: &ShaderUniforms, force: bool) -> Shader {
        self.parameters.uniforms.merge_mut(uniforms, force);
        self
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub enum ShaderLayer {
    Background,
    Unit,
    Vfx,
    UI,
}

impl Default for ShaderLayer {
    fn default() -> Self {
        ShaderLayer::UI
    }
}

impl ShaderLayer {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShaderParameters {
    #[serde(default = "vertices_default")]
    pub vertices: usize,
    #[serde(default = "instances_default")]
    pub instances: usize,
    pub uniforms: ShaderUniforms,
}

fn vertices_default() -> usize {
    3
}

fn instances_default() -> usize {
    1
}

impl Default for ShaderParameters {
    fn default() -> Self {
        Self {
            vertices: 3,
            instances: 1,
            uniforms: default(),
        }
    }
}

impl ugli::Uniforms for ShaderParameters {
    fn walk_uniforms<C>(&self, visitor: &mut C)
    where
        C: ugli::UniformVisitor,
    {
        for (name, value) in &self.uniforms.0 {
            visitor.visit(name, value);
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ShaderUniforms(pub HashMap<String, ShaderUniform>);

impl ShaderUniforms {
    pub fn merge(&self, other: &ShaderUniforms) -> Self {
        let mut result: ShaderUniforms = self.clone();
        other
            .0
            .iter()
            .for_each(|(key, value)| result.insert(key.clone(), value.clone()));
        result
    }

    pub fn merge_mut(&mut self, other: &ShaderUniforms, force: bool) -> &mut Self {
        other.0.iter().for_each(|(key, value)| {
            if force || !self.0.contains_key(key) {
                self.insert(key.clone(), value.clone());
            }
        });
        self
    }

    pub fn mix(a: &Self, b: &Self, t: f32) -> Self {
        let mut result: ShaderUniforms = default();
        for (key, value) in a.0.iter() {
            let a = value;
            let b = b.get(key).expect(&format!("Parameter {} absent in B", key));
            match (a, b) {
                (ShaderUniform::Int(a), ShaderUniform::Int(b)) => {
                    result.insert(key.clone(), ShaderUniform::Int(a + (b - a) * t as i32));
                }
                (ShaderUniform::Float(a), ShaderUniform::Float(b)) => {
                    result.insert(key.clone(), ShaderUniform::Float(a + (b - a) * t));
                }
                (ShaderUniform::Vec2(a), ShaderUniform::Vec2(b)) => {
                    result.insert(key.clone(), ShaderUniform::Vec2(*a + (*b - *a) * t));
                }
                (ShaderUniform::Vec3(a), ShaderUniform::Vec3(b)) => {
                    result.insert(key.clone(), ShaderUniform::Vec3(*a + (*b - *a) * t));
                }
                (ShaderUniform::Vec4(a), ShaderUniform::Vec4(b)) => {
                    result.insert(key.clone(), ShaderUniform::Vec4(*a + (*b - *a) * t));
                }
                (ShaderUniform::Color(a), ShaderUniform::Color(b)) => {
                    result.insert(
                        key.clone(),
                        ShaderUniform::Color(Rgba::new(
                            a.r + (b.r - a.r) * t,
                            a.g + (b.g - a.g) * t,
                            a.b + (b.b - a.b) * t,
                            a.a + (b.a - a.a) * t,
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

    pub fn insert(&mut self, key: String, value: ShaderUniform) {
        self.0.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&ShaderUniform> {
        self.0.get(key)
    }

    pub fn get_vec2(&self, key: &str) -> Option<vec2<f32>> {
        self.get(key).and_then(|v| match v {
            ShaderUniform::Vec2(v) => Some(*v),
            _ => None,
        })
    }

    pub fn get_float(&self, key: &str) -> Option<f32> {
        self.get(key).and_then(|v| match v {
            ShaderUniform::Float(v) => Some(*v),
            _ => None,
        })
    }
}

impl From<HashMap<&str, ShaderUniform>> for ShaderUniforms {
    fn from(value: HashMap<&str, ShaderUniform>) -> Self {
        Self(HashMap::from_iter(
            value
                .into_iter()
                .map(|(key, value)| (key.to_string(), value)),
        ))
    }
}

impl From<HashMap<String, ShaderUniform>> for ShaderUniforms {
    fn from(value: HashMap<String, ShaderUniform>) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ShaderUniform {
    Int(i32),
    Float(f32),
    Vec2(vec2<f32>),
    Vec3(vec3<f32>),
    Vec4(vec4<f32>),
    Color(Rgba<f32>),
    String((usize, String)),
    Texture(Image),
}

impl ugli::Uniform for ShaderUniform {
    fn apply(&self, gl: &ugli::raw::Context, info: &ugli::UniformInfo) {
        match self {
            Self::Int(value) => value.apply(gl, info),
            Self::Float(value) => value.apply(gl, info),
            Self::Vec2(value) => value.apply(gl, info),
            Self::Vec3(value) => value.apply(gl, info),
            Self::Vec4(value) => value.apply(gl, info),
            Self::Color(value) => value.apply(gl, info),
            Self::String(_) => {}
            Self::Texture(_) => {}
        }
    }
}

#[derive(Default)]
pub struct SingleUniformVec<'a, U: ugli::Uniform>(pub Vec<ugli::SingleUniform<'a, U>>);

impl<'a, U: ugli::Uniform> ugli::Uniforms for SingleUniformVec<'a, U> {
    fn walk_uniforms<C>(&self, visitor: &mut C)
    where
        C: ugli::UniformVisitor,
    {
        self.0
            .iter()
            .for_each(|uniform| uniform.walk_uniforms(visitor));
    }
}
