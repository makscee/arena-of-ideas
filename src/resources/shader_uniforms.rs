use super::*;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ShaderUniforms(pub HashMap<String, ShaderUniform>);

impl ShaderUniforms {
    pub fn merge(&self, other: &ShaderUniforms) -> Self {
        let mut result: ShaderUniforms = self.clone();
        other
            .0
            .iter()
            .for_each(|(key, value)| result.insert_ref(key, value.clone()));
        result
    }

    pub fn merge_mut(&mut self, other: &ShaderUniforms, force: bool) -> &mut Self {
        other.0.iter().for_each(|(key, value)| {
            if force || !self.0.contains_key(key) {
                self.insert_ref(key, value.clone());
            }
        });
        self
    }

    pub fn mix(a: &Self, b: &Self, t: f32) -> Self {
        let mut result: ShaderUniforms = default();
        for (key, value) in a.0.iter() {
            let a = value;
            let b = b.get(key).unwrap_or(a);
            match (a, b) {
                (ShaderUniform::Int(a), ShaderUniform::Int(b)) => {
                    result.insert_ref(key, ShaderUniform::Int(a + (b - a) * t as i32));
                }
                (ShaderUniform::Float(a), ShaderUniform::Float(b)) => {
                    result.insert_ref(key, ShaderUniform::Float(a + (b - a) * t));
                }
                (ShaderUniform::Vec2(a), ShaderUniform::Vec2(b)) => {
                    result.insert_ref(key, ShaderUniform::Vec2(*a + (*b - *a) * t));
                }
                (ShaderUniform::Vec3(a), ShaderUniform::Vec3(b)) => {
                    result.insert_ref(key, ShaderUniform::Vec3(*a + (*b - *a) * t));
                }
                (ShaderUniform::Vec4(a), ShaderUniform::Vec4(b)) => {
                    result.insert_ref(key, ShaderUniform::Vec4(*a + (*b - *a) * t));
                }
                (ShaderUniform::Color(a), ShaderUniform::Color(b)) => {
                    result.insert_ref(
                        key,
                        ShaderUniform::Color(Rgba::new(
                            a.r + (b.r - a.r) * t,
                            a.g + (b.g - a.g) * t,
                            a.b + (b.b - a.b) * t,
                            a.a + (b.a - a.a) * t,
                        )),
                    );
                }
                (ShaderUniform::String(a), ShaderUniform::String(b)) => {
                    result.insert_ref(
                        key,
                        ShaderUniform::String(match t < 0.5 {
                            true => a.clone(),
                            false => b.clone(),
                        }),
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

    pub fn insert_ref(&mut self, key: &str, value: ShaderUniform) {
        self.0.insert(key.to_string(), value);
    }

    pub fn insert_vec_ref(&mut self, key: &str, value: vec2<f32>) -> &mut Self {
        self.insert_ref(key, ShaderUniform::Vec2(value));
        self
    }

    pub fn insert_color_ref(&mut self, key: &str, value: Rgba<f32>) -> &mut Self {
        self.insert_ref(key, ShaderUniform::Color(value));
        self
    }

    pub fn insert(mut self, key: &str, value: ShaderUniform) -> Self {
        self.insert_ref(key, value);
        self
    }

    pub fn insert_vec(mut self, key: &str, value: vec2<f32>) -> Self {
        self.insert_vec_ref(key, value);
        self
    }

    pub fn insert_color(mut self, key: &str, value: Rgba<f32>) -> Self {
        self.insert_color_ref(key, value);
        self
    }

    pub fn get(&self, key: &str) -> Option<&ShaderUniform> {
        self.0.get(key)
    }

    pub fn try_get_vec2(&self, key: &str) -> Option<vec2<f32>> {
        self.get(key).and_then(|v| match v {
            ShaderUniform::Vec2(v) => Some(*v),
            _ => None,
        })
    }

    pub fn try_get_float(&self, key: &str) -> Option<f32> {
        self.get(key).and_then(|v| match v {
            ShaderUniform::Float(v) => Some(*v),
            _ => None,
        })
    }

    pub fn try_get_int(&self, key: &str) -> Option<i32> {
        self.get(key).and_then(|v| match v {
            ShaderUniform::Int(v) => Some(*v),
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

#[derive(Debug, Clone, Deserialize, Serialize)]
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
