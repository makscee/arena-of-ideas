use super::*;
use crate::ugli::VertexBuffer;
use geng::draw_2d::Vertex;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ShaderConfig {
    pub path: String,
    #[serde(default)]
    pub parameters: ShaderParameters,
    #[serde(default = "default_vertices")]
    pub vertices: usize,
    #[serde(default = "default_instances")]
    pub instances: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PostfxConfig {
    pub pipes: Vec<PostfxPipeConfig>,
    pub blend_shader: ShaderConfig,
    pub final_shader: ShaderConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PostfxPipeConfig {
    pub shaders: Vec<ShaderConfig>,
}

pub struct PostfxProgram {
    pub pipes: Vec<Vec<ShaderProgram>>,
    pub blend_shader: ShaderProgram,
    pub final_shader: ShaderProgram,
}

fn default_vertices() -> usize {
    4
}

fn default_instances() -> usize {
    1
}

#[derive(Clone)]
pub struct ShaderProgram {
    pub program: Rc<ugli::Program>,
    pub parameters: ShaderParameters,
    pub vertices: usize,
    pub instances: usize,
}

impl Default for ShaderConfig {
    fn default() -> Self {
        ShaderConfig {
            path: "".to_string(),
            parameters: default(),
            instances: 1,
            vertices: 1,
        }
    }
}

#[derive(ugli::Vertex, Debug, Clone)]
pub struct Instance {}

impl ShaderProgram {
    pub fn get_vertices(&self, geng: &Geng) -> VertexBuffer<Vertex> {
        let vert_count = self.vertices;
        let mut vertices = vec![draw_2d::Vertex {
            a_pos: vec2(-1.0, -1.0),
        }];
        for i in 0..vert_count {
            vertices.push(draw_2d::Vertex {
                a_pos: vec2((i as f32 / vert_count as f32) * 2.0 - 1.0, 1.0),
            });
            vertices.push(draw_2d::Vertex {
                a_pos: vec2(((i + 1) as f32 / vert_count as f32) * 2.0 - 1.0, -1.0),
            });
        }

        vertices.push(draw_2d::Vertex {
            a_pos: vec2(1.0, 1.0),
        });

        ugli::VertexBuffer::new_dynamic(geng.ugli(), vertices)
    }
}

// impl Default for ShaderProgram {
//     fn default() -> Self {
//         Self::Circle {
//             color: Color::BLACK,
//         }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShaderParameters(pub HashMap<String, ShaderParameter>);

impl ShaderParameters {
    pub fn new() -> Self {
        let mut parameters = Self {
            ..Default::default()
        };
        parameters
    }
}

impl ugli::Uniforms for ShaderParameters {
    fn walk_uniforms<C>(&self, visitor: &mut C)
    where
        C: ugli::UniformVisitor,
    {
        for (name, value) in &self.0 {
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
    Color(Color<f32>),
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
