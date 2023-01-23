use geng::prelude::itertools::Itertools;

use super::*;

pub struct ShaderSystem {}

impl ShaderSystem {
    pub fn draw(world: &World, geng: &Geng, assets: &Assets, framebuffer: &mut ugli::Framebuffer) {
        let shaders = <&Shader>::query().iter(world).collect_vec();
        shaders.iter().for_each(|shader| {
            Self::draw_shader(shader, &geng, framebuffer, assets, ugli::uniforms!())
        });
    }

    fn draw_shader<U>(
        shader: &Shader,
        geng: &Geng,
        framebuffer: &mut ugli::Framebuffer,
        assets: &Assets,
        uniforms: U,
    ) where
        U: ugli::Uniforms,
    {
        let mut instances_arr: ugli::VertexBuffer<Instance> =
            ugli::VertexBuffer::new_dynamic(geng.ugli(), Vec::new());
        instances_arr.resize(shader.parameters.instances, Instance {});
        let program = assets.shader_programs.get_program(&shader.path);
        let quad = Self::get_quad(shader.parameters.vertices, &geng);
        ugli::draw(
            framebuffer,
            &program,
            ugli::DrawMode::TriangleStrip,
            ugli::instanced(&quad, &instances_arr),
            (
                ugli::uniforms! {
                    u_time: 0.0,
                },
                geng::camera2d_uniforms(&assets.camera, framebuffer.size().map(|x| x as f32)),
                &shader.parameters,
                uniforms,
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::default()),
                ..default()
            },
        );
    }

    fn get_quad(vertices: usize, geng: &Geng) -> ugli::VertexBuffer<draw_2d::Vertex> {
        let vert_count = vertices;
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

    pub fn load_shaders(world: &mut World) {}
}

#[derive(ugli::Vertex, Debug, Clone)]
pub struct Instance {}
