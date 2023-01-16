use std::path::PathBuf;

use super::*;

use serde::*;

#[derive(Clone, Deserialize, geng::Assets)]
#[asset(json)]
#[serde(deny_unknown_fields)]
pub struct ShaderProgram {
    pub path: PathBuf,
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

    pub async fn load(&mut self, geng: &Geng) {
        let path = &static_path().join(&self.path);
        let program = <Program as geng::LoadAsset>::load(geng, path).await;
        match program {
            Ok(result) => self.program = Some(Rc::new(result)),
            Err(error) => error!("Shader load error: {}", error),
        }
    }
}

#[derive(ugli::Vertex, Debug, Clone)]
pub struct Instance {}
