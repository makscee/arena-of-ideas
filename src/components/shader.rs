use std::collections::VecDeque;

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

    pub fn set_int(self, key: &str, value: i32) -> Self {
        self.set_uniform(key, ShaderUniform::Int(value))
    }
    pub fn set_int_ref(&mut self, key: &str, value: i32) -> &mut Self {
        self.set_uniform_ref(key, ShaderUniform::Int(value))
    }

    pub fn set_float(self, key: &str, value: f32) -> Self {
        self.set_uniform(key, ShaderUniform::Float(value))
    }
    pub fn set_float_ref(&mut self, key: &str, value: f32) -> &mut Self {
        self.set_uniform_ref(key, ShaderUniform::Float(value))
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

    pub fn set_mapping(mut self, from: &str, to: &str) -> Self {
        self.parameters.uniforms.add_mapping(from, to);
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
        let aabb = {
            let mut position = uniforms.try_get_vec2("u_position").unwrap();
            let mut bx = uniforms.try_get_vec2("u_box").unwrap();
            let card = uniforms.try_get_float("u_card").unwrap_or_default();
            bx *= 1.0 + card;
            position -= card * vec2(0.0, 0.7);
            Aabb2::from_corners(position - bx, position + bx)
        };
        let position = match uniforms.try_get_int("u_ui").unwrap_or_default() {
            1 => mouse_screen,
            _ => mouse_world,
        };
        aabb.contains(position)
    }

    pub fn inject_bounding_box(mut self, resources: &Resources) -> Self {
        let uniforms = &mut self.parameters.uniforms;

        let mut position = uniforms.try_get_vec2("u_position").unwrap_or(vec2::ZERO);
        let align = uniforms.try_get_vec2("u_align").unwrap_or(vec2::ZERO);
        let offset = uniforms.try_get_vec2("u_offset").unwrap_or(vec2::ZERO);

        let index = uniforms.try_get_int("u_index").unwrap_or_default();
        let index_offset = uniforms
            .try_get_vec2("u_index_offset")
            .unwrap_or(vec2::ZERO);
        let card_offset = uniforms.try_get_vec2("u_card_offset").unwrap_or(vec2::ZERO);

        let mut bx = uniforms.try_get_vec2("u_box").unwrap_or(vec2(1.0, 1.0));

        let card = uniforms.try_get_float("u_card").unwrap_or_default();
        let mut scale = uniforms.try_get_float("u_scale").unwrap_or(1.0);
        let card_size = uniforms.try_get_float("u_card_size").unwrap_or_default();
        let size = uniforms.try_get_float("u_size").unwrap_or(1.0) + card_size * card;
        let zoom = uniforms.try_get_float("u_zoom").unwrap_or(1.0);
        let mut offset = offset + card_offset * card + index_offset * index as f32;
        position += card * vec2(0.0, 0.7);
        scale *= 1.0 - card * 0.5;
        scale *= zoom;

        let aspect_adjust = uniforms.try_get_int("u_aspect_adjust").unwrap_or_default() != 0;
        let mut aspect_ratio = 1.0;
        if aspect_adjust {
            let screen_size = resources.camera.framebuffer_size;
            aspect_ratio = screen_size.x / screen_size.y;
            bx.x /= aspect_ratio;
            if uniforms
                .try_get_int("u_aspect_offset_adjust")
                .unwrap_or_default()
                != 0
            {
                offset.x /= aspect_ratio;
            }
        }
        uniforms.insert_float_ref("u_aspect_ratio", aspect_ratio);

        let aabb = Aabb2::point(position + offset * scale).extend_symmetric(bx * size * scale);
        let aabb = aabb.translate(align * aabb.size() * 0.5);

        uniforms.insert_vec2_ref("u_position", aabb.center());
        uniforms.insert_vec2_ref("u_box", aabb.size() * 0.5);

        self
    }

    pub fn request_vars(&self) -> HashSet<&VarName> {
        let mut result = HashSet::from_iter(DEFAULT_REQUEST_VARS.iter());
        let mut queue = VecDeque::from_iter(Some(self));
        while let Some(shader) = queue.pop_front() {
            result.extend(shader.request_vars.iter());
            queue.extend(shader.chain_before.iter());
            queue.extend(shader.chain_after.iter());
        }
        result
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub enum ShaderLayer {
    Background,
    Unit,
    Vfx,
    UI,
}

impl Default for ShaderLayer {
    fn default() -> Self {
        ShaderLayer::Unit
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
