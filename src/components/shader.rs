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
    #[serde(default)]
    request_vars: Vec<VarName>,
    #[serde(skip)]
    pub ts: i64,
    #[serde(skip)]
    pub entity: Option<legion::Entity>,
    #[serde(skip)]
    pub parent: Option<legion::Entity>,
    #[serde(skip)]
    pub input_handlers: Vec<Handler>,
}

const DEFAULT_REQUEST_VARS: [VarName; 14] = [
    VarName::Position,
    VarName::Radius,
    VarName::Box,
    VarName::Size,
    VarName::Scale,
    VarName::Card,
    VarName::Zoom,
    VarName::Charges,
    VarName::Color,
    VarName::GlobalTime,
    VarName::Rank,
    VarName::BackgroundLight,
    VarName::BackgroundDark,
    VarName::OutlineColor,
];

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

    pub fn set_color(self, key: &str, value: Rgba<f32>) -> Self {
        self.set_uniform(key, ShaderUniform::Color(value))
    }
    pub fn set_color_ref(&mut self, key: &str, value: Rgba<f32>) -> &mut Self {
        self.set_uniform_ref(key, ShaderUniform::Color(value))
    }

    pub fn set_vec2(self, key: &str, value: vec2<f32>) -> Self {
        self.set_uniform(key, ShaderUniform::Vec2(value))
    }
    pub fn set_vec2_ref(&mut self, key: &str, value: vec2<f32>) -> &mut Self {
        self.set_uniform_ref(key, ShaderUniform::Vec2(value))
    }

    pub fn set_string(self, key: &str, value: String, font: usize) -> Self {
        self.set_uniform(key, ShaderUniform::String((font, value)))
    }
    pub fn set_string_ref(&mut self, key: &str, value: String, font: usize) -> &mut Self {
        self.set_uniform_ref(key, ShaderUniform::String((font, value)))
    }

    pub fn merge_uniforms(mut self, uniforms: &ShaderUniforms, force: bool) -> Self {
        self.merge_uniforms_ref(uniforms, force);
        self
    }

    pub fn merge_uniforms_ref(&mut self, uniforms: &ShaderUniforms, force: bool) -> &mut Self {
        self.parameters.uniforms.merge_mut(uniforms, force);
        self
    }

    pub fn is_enabled(&self) -> bool {
        if let Some(enabled) = self.parameters.uniforms.try_get_float("u_enabled") {
            enabled > 0.0
        } else {
            true
        }
    }

    pub fn add_context_ref(&mut self, context: &Context, world: &legion::World) -> &mut Self {
        let mut vars: Vars = default();
        for var in self.request_vars() {
            if let Some(value) = context.get_var(var, world) {
                vars.insert(*var, value);
            }
        }
        self.parameters.uniforms.merge_mut(&vars.into(), true);
        self
    }

    pub fn add_context(mut self, context: &Context, world: &legion::World) -> Self {
        self.add_context_ref(context, world);
        self
    }

    pub fn is_hovered(&self, mouse_screen: vec2<f32>, mouse_world: vec2<f32>) -> bool {
        if !self.is_enabled() {
            return false;
        }
        let uniforms = &self.parameters.uniforms;
        let scale = uniforms
            .try_get_float(&VarName::Scale.uniform())
            .unwrap_or(1.0);

        let offset = uniforms.try_get_vec2("u_offset").unwrap_or(vec2::ZERO);
        let position = uniforms
            .try_get_vec2(&VarName::Position.uniform())
            .unwrap_or(vec2::ZERO)
            + offset;
        let mouse_pos = if uniforms.try_get_float("u_ui").unwrap_or_default() == 1.0 {
            mouse_screen
        } else {
            mouse_world
        };
        let position = mouse_pos - position;
        if let Some(radius) = self
            .parameters
            .uniforms
            .try_get_float(&VarName::Radius.uniform())
        {
            position.len() < radius * scale
        } else if let Some(r#box) = self
            .parameters
            .uniforms
            .try_get_vec2(&VarName::Box.uniform())
        {
            position.x.abs() < r#box.x && position.y.abs() < r#box.y
        } else {
            false
        }
    }

    pub fn request_vars(&self) -> impl Iterator<Item = &VarName> {
        DEFAULT_REQUEST_VARS.iter().chain(self.request_vars.iter())
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
#[serde(deny_unknown_fields)]
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
        for (name, value) in self.uniforms.iter() {
            visitor.visit(name, value);
        }
    }
}
