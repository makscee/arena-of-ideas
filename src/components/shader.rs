use std::{collections::VecDeque, f32::consts::PI};

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
    #[serde(skip)]
    pub update_handlers: Vec<Handler>,
    #[serde(skip)]
    pub hover_hints: Vec<(Rgba<f32>, String, String)>,
}

const DEFAULT_REQUEST_VARS: [VarName; 12] = [
    VarName::Position,
    VarName::Scale,
    VarName::Card,
    VarName::Zoom,
    VarName::Charges,
    VarName::Color,
    VarName::HouseColor,
    VarName::GlobalTime,
    VarName::Rank,
    VarName::BackgroundLight,
    VarName::BackgroundDark,
    VarName::FactionColor,
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
    pub fn insert_uniform(mut self, key: String, value: ShaderUniform) -> Self {
        self.parameters.uniforms.insert_ref(key, value);
        self
    }
    pub fn insert_uniform_ref(&mut self, key: String, value: ShaderUniform) -> &mut Self {
        self.parameters.uniforms.insert_ref(key, value);
        self
    }

    pub fn insert_color(self, key: String, value: Rgba<f32>) -> Self {
        self.insert_uniform(key, ShaderUniform::Color(value))
    }
    pub fn insert_color_ref(&mut self, key: String, value: Rgba<f32>) -> &mut Self {
        self.insert_uniform_ref(key, ShaderUniform::Color(value))
    }

    pub fn insert_int(self, key: String, value: i32) -> Self {
        self.insert_uniform(key, ShaderUniform::Int(value))
    }
    pub fn insert_int_ref(&mut self, key: String, value: i32) -> &mut Self {
        self.insert_uniform_ref(key, ShaderUniform::Int(value))
    }

    pub fn insert_float(self, key: String, value: f32) -> Self {
        self.insert_uniform(key, ShaderUniform::Float(value))
    }
    pub fn insert_float_ref(&mut self, key: String, value: f32) -> &mut Self {
        self.insert_uniform_ref(key, ShaderUniform::Float(value))
    }

    pub fn insert_vec2(self, key: String, value: vec2<f32>) -> Self {
        self.insert_uniform(key, ShaderUniform::Vec2(value))
    }
    pub fn insert_vec2_ref(&mut self, key: String, value: vec2<f32>) -> &mut Self {
        self.insert_uniform_ref(key, ShaderUniform::Vec2(value))
    }

    pub fn insert_string(self, key: String, value: String, font: usize) -> Self {
        self.insert_uniform(key, ShaderUniform::String((font, value)))
    }
    pub fn insert_string_ref(&mut self, key: String, value: String, font: usize) -> &mut Self {
        self.insert_uniform_ref(key, ShaderUniform::String((font, value)))
    }

    pub fn get_int(&self, key: &str) -> i32 {
        self.parameters.uniforms.try_get_int(key).unwrap()
    }

    pub fn get_float(&self, key: &str) -> f32 {
        self.parameters.uniforms.try_get_float(key).unwrap()
    }

    pub fn get_vec2(&self, key: &str) -> vec2<f32> {
        self.parameters.uniforms.try_get_vec2(key).unwrap()
    }

    pub fn get_string(&self, key: &str) -> String {
        self.parameters.uniforms.try_get_string(key).unwrap()
    }

    pub fn map_key_to_key(mut self, from: &str, to: &str) -> Self {
        self.parameters.uniforms.map_key_to_key(from, to);
        self
    }

    pub fn add_mapping(&mut self, key: &str, expr: ExpressionUniform) -> &mut Self {
        self.parameters.uniforms.add_mapping(key, expr);
        self
    }

    pub fn remove_mapping(mut self, key: &str) -> Self {
        self.parameters.uniforms.remove_mapping(key);
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

    pub fn set_enabled(&mut self, value: bool) {
        self.insert_float_ref("u_enabled".to_owned(), value as i32 as f32);
    }

    pub fn is_enabled(&self) -> bool {
        if let Some(enabled) = self.parameters.uniforms.try_get_float("u_enabled") {
            enabled > 0.0
        } else {
            true
        }
    }

    pub fn set_active(&mut self, value: bool) {
        self.insert_float_ref("u_active".to_owned(), value as i32 as f32);
    }

    pub fn is_active(&self) -> bool {
        if let Some(active) = self.parameters.uniforms.try_get_float("u_active") {
            active > 0.0
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
            let position = uniforms.try_get_vec2("u_position").unwrap();
            let bx = uniforms.try_get_vec2("u_box").unwrap();
            Aabb2::from_corners(position - bx, position + bx)
        };
        let position = match uniforms.try_get_int("u_ui").unwrap_or_default() {
            1 => mouse_screen,
            _ => mouse_world,
        };
        aabb.contains(position)
    }

    pub fn inject_uniforms(mut self, resources: &Resources) -> Self {
        let uniforms = &mut self.parameters.uniforms;

        let rand = uniforms.try_get_float("u_rand").unwrap_or_default();
        let t = PI * rand * 2.0
            + resources.global_time * uniforms.try_get_float("u_t_multiplier").unwrap_or(1.0);
        uniforms.insert_float_ref("u_sin".to_owned(), t.sin());
        uniforms.insert_float_ref("u_cos".to_owned(), t.cos());

        let (mut position, bx) = self.parameters.r#box.get_pos_size();
        position += uniforms.try_get_vec2("u_position").unwrap_or(vec2::ZERO);
        let offset = uniforms.try_get_vec2("u_offset").unwrap_or(vec2::ZERO);

        let index = uniforms.try_get_int("u_index").unwrap_or_default();
        let index_offset = uniforms
            .try_get_vec2("u_index_offset")
            .unwrap_or(vec2::ZERO);
        let card_offset = uniforms.try_get_vec2("u_card_offset").unwrap_or(vec2::ZERO);

        let card = uniforms.try_get_float("u_card").unwrap_or_default();
        let mut scale = uniforms.try_get_float("u_scale").unwrap_or(1.0)
            * uniforms.try_get_float("u_open").unwrap_or(1.0);
        let local_scale = uniforms.try_get_float("u_local_scale").unwrap_or(1.0);
        let card_size = uniforms.try_get_float("u_card_size").unwrap_or_default();
        let size = uniforms.try_get_float("u_size").unwrap_or(1.0)
            + card_size * card
            + uniforms.try_get_float("u_extra_size").unwrap_or_default()
                * uniforms
                    .try_get_float("u_extra_size_multiplier")
                    .unwrap_or(1.0);
        let zoom = uniforms.try_get_float("u_zoom").unwrap_or(1.0);
        let offset = offset + card_offset * card + index_offset * index as f32;
        scale *= zoom;

        let aspect_ratio = resources.camera.aspect_ratio;
        uniforms.insert_float_ref("u_aspect_ratio".to_owned(), aspect_ratio);

        let aabb = Aabb2::point(position + offset * scale)
            .extend_symmetric(bx * size * scale * local_scale);

        uniforms.insert_vec2_ref("u_position".to_owned(), aabb.center());
        uniforms.insert_vec2_ref("u_box".to_owned(), aabb.size() * 0.5);

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
    #[serde(default)]
    pub uniforms: ShaderUniforms,
    #[serde(default)]
    pub r#box: BoxParameters,
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
            r#box: default(),
        }
    }
}

impl ugli::Uniforms for ShaderParameters {
    fn walk_uniforms<C>(&self, visitor: &mut C)
    where
        C: ugli::UniformVisitor,
    {
        for (name, value) in self.uniforms.iter() {
            visitor.visit(name, &value);
        }
    }
}

impl ShaderParameters {
    pub fn merge(&mut self, other: &ShaderParameters, force: bool) {
        if force {
            self.r#box = other.r#box;
            self.instances = other.instances;
            self.vertices = other.vertices;
        }
        self.uniforms.merge_mut(&other.uniforms, force);
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct BoxParameters {
    #[serde(default = "vec_zero")]
    pub pos: vec2<f32>,
    #[serde(default = "vec_one")]
    pub size: vec2<f32>,
    #[serde(default = "vec_zero")]
    pub center: vec2<f32>,
    #[serde(default = "vec_zero")]
    pub anchor: vec2<f32>,
}

fn vec_zero() -> vec2<f32> {
    vec2::ZERO
}

fn vec_one() -> vec2<f32> {
    vec2(1.0, 1.0)
}

impl Default for BoxParameters {
    fn default() -> Self {
        Self {
            pos: vec2::ZERO,
            size: vec2(1.0, 1.0),
            center: vec2::ZERO,
            anchor: vec2::ZERO,
        }
    }
}

impl BoxParameters {
    pub fn consider_parent(&mut self, parent: &BoxParameters) {
        let (pos, size) = parent.get_pos_size();
        self.pos += size * self.anchor + pos;
    }

    pub fn get_pos_size(&self) -> (vec2<f32>, vec2<f32>) {
        (self.pos - self.center * self.size, self.size)
    }
}
