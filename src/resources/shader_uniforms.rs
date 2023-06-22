use super::*;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ShaderUniforms {
    #[serde(flatten)]
    data: HashMap<String, ShaderUniform>,
    #[serde(default)]
    local: HashMap<String, ShaderUniform>,
    #[serde(default)]
    mapping: HashMap<String, String>,
}

impl ShaderUniforms {
    pub fn single(key: &str, value: ShaderUniform) -> Self {
        Self {
            data: hashmap! {key.to_owned() => value},
            local: default(),
            mapping: default(),
        }
    }

    pub fn merge(&self, other: &ShaderUniforms) -> Self {
        let mut result: ShaderUniforms = self.clone();
        other
            .iter()
            .for_each(|(key, value)| result.insert_ref(key.to_owned(), value.clone()));
        result
    }

    pub fn merge_mut(&mut self, other: &ShaderUniforms, force: bool) -> &mut Self {
        other.iter().for_each(|(key, value)| {
            if force || !self.data.contains_key(key) {
                self.insert_ref(key.to_owned(), value.clone());
            }
        });
        self
    }

    pub fn mix(a: &Self, b: &Self, t: f32, defaults: &Self) -> Self {
        let mut result: ShaderUniforms = default();
        let keys: HashSet<&String> = HashSet::from_iter(a.data.keys().chain(a.mapping.keys()));
        for key in keys {
            let a = a.get(key).unwrap_or_else(|| {
                defaults.get(a.map(key)).expect(&format!(
                    "Failed to mix key {key} a: {a:?} defaults: {defaults:?}"
                ))
            });
            let b = b.get(key).unwrap_or_else(|| {
                defaults.get(b.map(key)).expect(&format!(
                    "Failed to mix key {key} b: {b:?} defaults: {defaults:?}"
                ))
            });
            match (&a, &b) {
                (ShaderUniform::Int(a), ShaderUniform::Int(b)) => {
                    result.insert_ref(key.to_owned(), ShaderUniform::Int(a + (b - a) * t as i32));
                }
                (ShaderUniform::Float(a), ShaderUniform::Float(b)) => {
                    result.insert_ref(key.to_owned(), ShaderUniform::Float(a + (b - a) * t));
                }
                (ShaderUniform::Vec2(a), ShaderUniform::Vec2(b)) => {
                    result.insert_ref(key.to_owned(), ShaderUniform::Vec2(*a + (*b - *a) * t));
                }
                (ShaderUniform::Vec3(a), ShaderUniform::Vec3(b)) => {
                    result.insert_ref(key.to_owned(), ShaderUniform::Vec3(*a + (*b - *a) * t));
                }
                (ShaderUniform::Vec4(a), ShaderUniform::Vec4(b)) => {
                    result.insert_ref(key.to_owned(), ShaderUniform::Vec4(*a + (*b - *a) * t));
                }
                (ShaderUniform::Color(a), ShaderUniform::Color(b)) => {
                    result.insert_ref(
                        key.to_owned(),
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
                        key.to_owned(),
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

    pub fn insert_ref(&mut self, key: String, value: ShaderUniform) {
        self.data.insert(key, value);
    }

    pub fn insert_vec2_ref(&mut self, key: String, value: vec2<f32>) -> &mut Self {
        self.insert_ref(key, ShaderUniform::Vec2(value));
        self
    }

    pub fn insert_float_ref(&mut self, key: String, value: f32) -> &mut Self {
        self.insert_ref(key, ShaderUniform::Float(value));
        self
    }

    pub fn insert_int_ref(&mut self, key: String, value: i32) -> &mut Self {
        self.insert_ref(key, ShaderUniform::Int(value));
        self
    }

    pub fn insert_color_ref(&mut self, key: String, value: Rgba<f32>) -> &mut Self {
        self.insert_ref(key, ShaderUniform::Color(value));
        self
    }

    pub fn insert_string_ref(&mut self, key: String, value: String, font: usize) -> &mut Self {
        self.insert_ref(key, ShaderUniform::String((font, value)));
        self
    }

    pub fn insert(mut self, key: String, value: ShaderUniform) -> Self {
        self.insert_ref(key, value);
        self
    }

    pub fn insert_vec2(mut self, key: String, value: vec2<f32>) -> Self {
        self.insert_vec2_ref(key, value);
        self
    }

    pub fn insert_float(mut self, key: String, value: f32) -> Self {
        self.insert_float_ref(key, value);
        self
    }

    pub fn insert_int(mut self, key: String, value: i32) -> Self {
        self.insert_int_ref(key, value);
        self
    }

    pub fn insert_color(mut self, key: String, value: Rgba<f32>) -> Self {
        self.insert_color_ref(key, value);
        self
    }

    pub fn insert_local_ref(&mut self, key: String, value: ShaderUniform) -> &mut Self {
        self.local.insert(key.to_owned(), value);
        self
    }

    pub fn insert_color_local_ref(&mut self, key: String, value: Rgba<f32>) -> &mut Self {
        self.insert_local_ref(key, ShaderUniform::Color(value))
    }

    pub fn get(&self, key: &str) -> Option<ShaderUniform> {
        let key = self.map(key);
        if key.starts_with("c_") {
            return Some(ShaderUniform::Color(options_color(key)));
        }
        self.local.get(key).or(self.data.get(key)).cloned()
    }

    pub fn map<'a>(&'a self, mut key: &'a str) -> &'a str {
        while let Some(map) = self.mapping.get(key) {
            key = map.as_str()
        }
        key
    }

    pub fn try_get_vec2(&self, key: &str) -> Option<vec2<f32>> {
        self.get(key).and_then(|v| match v {
            ShaderUniform::Vec2(v) => Some(v),
            _ => None,
        })
    }

    pub fn try_get_float(&self, key: &str) -> Option<f32> {
        self.get(key).and_then(|v| match v {
            ShaderUniform::Float(v) => Some(v),
            _ => None,
        })
    }

    pub fn try_get_int(&self, key: &str) -> Option<i32> {
        self.get(key).and_then(|v| match v {
            ShaderUniform::Int(v) => Some(v),
            _ => None,
        })
    }

    pub fn try_get_string(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|v| match v {
            ShaderUniform::String(v) => Some(v.1.to_owned()),
            _ => None,
        })
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&String, ShaderUniform)> + 'a {
        self.data.keys().map(|key| {
            (
                key,
                self.get(key).unwrap_or_else(|| {
                    self.data
                        .get(key)
                        .expect(&format!("Key not found {key}"))
                        .clone()
                }),
            )
        })
    }

    pub fn iter_mappings<'a>(&'a self) -> impl Iterator<Item = (&String, &String)> + 'a {
        self.mapping.iter()
    }

    pub fn iter_data<'a>(&'a self) -> impl Iterator<Item = (String, ShaderUniform)> + 'a {
        self.data
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
    }

    pub fn iter_local<'a>(&'a self) -> impl Iterator<Item = (&String, &ShaderUniform)> + 'a {
        self.local.iter()
    }

    pub fn add_mapping(&mut self, from: &str, to: &str) -> &mut Self {
        self.mapping.insert(from.to_string(), to.to_string());
        self
    }

    pub fn remove_mapping(&mut self, key: &str) -> &mut Self {
        self.mapping.remove(key);
        self
    }
}

impl From<HashMap<&str, ShaderUniform>> for ShaderUniforms {
    fn from(value: HashMap<&str, ShaderUniform>) -> Self {
        Self {
            data: HashMap::from_iter(
                value
                    .into_iter()
                    .map(|(key, value)| (key.to_string(), value)),
            ),
            local: default(),
            mapping: default(),
        }
    }
}

impl From<HashMap<String, ShaderUniform>> for ShaderUniforms {
    fn from(value: HashMap<String, ShaderUniform>) -> Self {
        Self {
            data: value,
            local: default(),
            mapping: default(),
        }
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
