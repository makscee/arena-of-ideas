use std::collections::hash_map::Entry;

use geng::prelude::itertools::Itertools;
use legion::EntityStore;

use super::*;

pub struct ShaderSystem {}

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
        Self {}
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
            for shader in shaders.iter() {
                let uniforms = ugli::uniforms!(u_global_time: resources.game_time);
                if let Some((key, value)) = shader.parameters.uniforms.find_string() {
                    if let Some(texture) = self.get_text_texture(&value, resources) {
                        Self::draw_shader(
                            shader,
                            framebuffer,
                            resources,
                            (
                                uniforms,
                                ugli::uniforms!(
                                    u_texture_size: texture.size().map(|x| x as f32),
                                    u_text_texture: texture,
                                ),
                            ),
                        );
                        continue;
                    }
                }
                Self::draw_shader(shader, framebuffer, resources, uniforms);
            }
        }
    }

    fn get_text_texture(&self, text: &String, resources: &Resources) -> Option<ugli::Texture> {
        resources.font.create_text_sdf(text, 64.0)
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
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
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
