use std::collections::hash_map::Entry;

use geng::prelude::itertools::Itertools;
use legion::EntityStore;

use super::*;

pub struct ShaderSystem {
    font_program: ugli::Program,
}

impl System for ShaderSystem {
    fn update(&mut self, world: &mut legion::World, _resources: &mut Resources) {}

    fn draw(
        &self,
        world: &legion::World,
        resources: &Resources,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.draw_all_shaders(world, resources, framebuffer);
    }
}

impl ShaderSystem {
    pub fn new(geng: &Geng) -> Self {
        Self {
            font_program: geng.shader_lib().compile(SHADER_SOURCE).unwrap(),
        }
    }

    /// Get Shader component and merge Context into it's vars if any
    pub fn get_entity_shader(world: &legion::World, entity: legion::Entity) -> Shader {
        let mut shader = world
            .entry_ref(entity)
            .expect("Failed to find Entry")
            .get_component::<Shader>()
            .unwrap()
            .clone();
        let context = Context::construct_context(&entity, world);
        shader.parameters.uniforms = shader
            .parameters
            .uniforms
            .merge(&context.vars.clone().into());
        shader
    }

    pub fn draw_all_shaders(
        &self,
        world: &legion::World,
        resources: &Resources,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        // Get all Shader components from World for drawing
        let world_shaders = <(&Shader, &EntityComponent)>::query()
            .iter(world)
            .map(|(_, entity)| Self::get_entity_shader(world, entity.entity))
            .collect_vec();
        let shaders = [world_shaders, resources.cassette.get_shaders()].concat();
        let mut shaders_by_layer: HashMap<ShaderLayer, Vec<Shader>> = HashMap::default();
        let emtpy_vec: Vec<Shader> = Vec::new();
        for shader in shaders {
            let layer = &shader.layer;
            let vec = match shaders_by_layer.entry(layer.clone()) {
                Entry::Occupied(o) => o.into_mut(),
                Entry::Vacant(v) => v.insert(emtpy_vec.clone()),
            };
            vec.push(shader);
        }
        for (_layer, shaders) in shaders_by_layer.iter().sorted_by_key(|entry| entry.0) {
            shaders.iter().for_each(|shader| {
                // if let Some((key, value)) = shader.parameters.uniforms.find_string() {
                //     let texture = self.get_text_texture(&value, resources);
                //     let uniforms = &ugli::uniforms!(
                //         u_texture_size: texture.size().map(|x| x as f32),
                //         u_text_texture: texture,
                //     );
                //     Self::draw_shader(shader, framebuffer, resources, uniforms);
                // } else {
                Self::draw_shader(
                    shader,
                    framebuffer,
                    resources,
                    ugli::uniforms!(u_global_time: resources.game_time),
                );
                // }
            })
        }
    }

    fn get_text_texture(&self, text: &String, resources: &Resources) -> ugli::Texture {
        let text = &"69".to_string();
        let font = resources.geng.default_font();
        let ugli = resources.geng.ugli();
        let text_bb = font.measure_bounding_box(text).unwrap();
        let mut texture =
            ugli::Texture::new_uninitialized(ugli, (text_bb.size() * 500.0).map(|x| x as usize));
        let framebuffer =
            &mut ugli::Framebuffer::new_color(ugli, ugli::ColorAttachment::Texture(&mut texture));
        ugli::clear(framebuffer, Some(Rgba::TRANSPARENT_BLACK), None, None);
        let camera = geng::Camera2d {
            center: text_bb.center(),
            rotation: 0.0,
            fov: 1.0,
        };
        font.draw_with(text, |glyphs, atlas| {
            ugli::draw(
                framebuffer,
                &self.font_program,
                ugli::DrawMode::TriangleFan,
                ugli::instanced(
                    &ugli::VertexBuffer::new_dynamic(
                        ugli,
                        Aabb2::point(vec2::ZERO)
                            .extend_positive(vec2(1.0, 1.0))
                            .corners()
                            .into_iter()
                            .map(|v| draw_2d::Vertex { a_pos: v })
                            .collect(),
                    ),
                    &ugli::VertexBuffer::new_dynamic(ugli, glyphs.to_vec()),
                ),
                (
                    ugli::uniforms! {
                        u_texture: atlas,
                    },
                    geng::camera2d_uniforms(&camera, framebuffer.size().map(|x| x as f32)),
                ),
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::default()),
                    ..default()
                },
            );
        });
        texture
    }

    fn draw_shader<U>(
        shader: &Shader,
        framebuffer: &mut ugli::Framebuffer,
        resources: &Resources,
        uniforms: U,
    ) where
        U: ugli::Uniforms,
    {
        let mut instances_arr: ugli::VertexBuffer<Instance> =
            ugli::VertexBuffer::new_dynamic(resources.geng.ugli(), Vec::new());
        instances_arr.resize(shader.parameters.instances, Instance {});
        let program = resources
            .shader_programs
            .get_program(&static_path().join(&shader.path));
        let quad = Self::get_quad(shader.parameters.vertices, &resources.geng);
        ugli::draw(
            framebuffer,
            &program,
            ugli::DrawMode::TriangleStrip,
            ugli::instanced(&quad, &instances_arr),
            (
                geng::camera2d_uniforms(&resources.camera, framebuffer.size().map(|x| x as f32)),
                &shader.parameters,
                uniforms,
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::default()),
                ..default()
            },
        );
    }

    pub fn get_quad(vertices: usize, geng: &Geng) -> ugli::VertexBuffer<draw_2d::Vertex> {
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
}

#[derive(ugli::Vertex, Debug, Clone)]
pub struct Instance {}

const SHADER_SOURCE: &str = "
varying vec2 v_uv;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
attribute vec2 i_pos;
attribute vec2 i_size;
attribute vec2 i_uv_pos;
attribute vec2 i_uv_size;

uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    v_uv = i_uv_pos + a_pos * i_uv_size;
    vec3 pos = u_projection_matrix * u_view_matrix * vec3(i_pos + a_pos * i_size, 1.0);
    gl_Position = vec4(pos.xy, 0.0, pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;

void main() {
    gl_FragColor = texture2D(u_texture, v_uv);
}
#endif
";
