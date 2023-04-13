use super::*;

/// Component to link to a shader program with specific parameters
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Shader {
    pub path: PathBuf, // static path
    #[serde(default)]
    pub parameters: ShaderParameters,
    #[serde(default)]
    pub layer: ShaderLayer,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub chain_before: Box<Vec<Shader>>,
    #[serde(default)]
    pub chain_after: Box<Vec<Shader>>,
    #[serde(skip)]
    pub ts: i64,
    #[serde(skip)]
    pub entity: Option<legion::Entity>,
    #[serde(skip)]
    pub input_handlers: Vec<Handler>,
}

impl Debug for Shader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shader")
            .field("path", &self.path)
            .field("parameters", &self.parameters)
            .field("layer", &self.layer)
            .field("order", &self.order)
            .field("chain_before", &self.chain_before)
            .field("chain_after", &self.chain_after)
            .field("ts", &self.ts)
            .field("entity", &self.entity)
            .finish()
    }
}

impl Shader {
    pub fn set_uniform(mut self, key: &str, value: ShaderUniform) -> Self {
        self.parameters.uniforms.insert_ref(key, value);
        self
    }
    pub fn set_uniform_ref(&mut self, key: &str, value: ShaderUniform) -> &mut Self {
        self.parameters.uniforms.insert_ref(key, value);
        self
    }

    pub fn merge_uniforms(mut self, uniforms: &ShaderUniforms, force: bool) -> Self {
        self.merge_uniforms_ref(uniforms, force);
        self
    }

    pub fn merge_uniforms_ref(&mut self, uniforms: &ShaderUniforms, force: bool) -> &mut Self {
        self.parameters.uniforms.merge_mut(uniforms, force);
        self
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub enum ShaderLayer {
    Background,
    Unit,
    Vfx,
    UI,
    Hover,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
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
