use super::*;

use serde::*;

#[derive(Clone, Deserialize)]
pub struct ShaderProgram {
    pub path: String,
    #[serde(skip)]
    pub program: Option<Rc<ugli::Program>>,
    pub parameters: ShaderParameters,
    pub vertices: usize,
    pub instances: usize,
}

impl ShaderProgram {
    pub fn get_vertices(&self, geng: &Geng) -> VertexBuffer<draw_2d::Vertex> {
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

#[derive(ugli::Vertex, Debug, Clone)]
pub struct Instance {}
