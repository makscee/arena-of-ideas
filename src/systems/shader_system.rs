use std::collections::{hash_map::Entry, VecDeque};

use geng::prelude::{itertools::Itertools, ugli::SingleUniform};
use legion::EntityStore;

use super::*;

pub struct ShaderSystem {}

impl System for ShaderSystem {
    fn update(&mut self, _world: &mut legion::World, resources: &mut Resources) {
        resources.frame_shaders.clear();
    }

    fn draw(
        &self,
        world: &legion::World,
        resources: &mut Resources,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.draw_all_shaders(world, resources, framebuffer);
    }
}

impl ShaderSystem {
    pub fn new() -> Self {
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
        match ContextSystem::try_get_context(entity, world) {
            Ok(context) => {
                shader.parameters.uniforms = shader
                    .parameters
                    .uniforms
                    .merge(&context.vars.clone().into())
            }
            Err(_) => {}
        }

        shader
    }

    pub fn draw_all_shaders(
        &self,
        world: &legion::World,
        resources: &mut Resources,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        // Get all Shader components from World for drawing
        let world_shaders: HashMap<legion::Entity, Shader> = HashMap::from_iter(
            <&EntityComponent>::query()
                .filter(!component::<UnitComponent>() & component::<Shader>())
                .iter(world)
                .map(|entity| (entity.entity, Self::get_entity_shader(world, entity.entity))),
        );

        let shaders = Cassette::get_shaders(resources, world_shaders)
            .into_iter()
            .chain(resources.frame_shaders.drain(..))
            .collect_vec();

        for shader in shaders {
            let uniforms = ugli::uniforms!(
                u_game_time: resources.cassette.head,
                u_global_time: resources.global_time,
            );
            let texts = shader
                .parameters
                .uniforms
                .0
                .iter()
                .filter_map(|(key, uniform)| match uniform {
                    ShaderUniform::String((font, text)) => {
                        Some((*font, text, key, format!("{}_size", key)))
                    }
                    _ => None,
                })
                .collect_vec();
            resources.fonts.load_textures(
                texts
                    .iter()
                    .map(|(font, text, _, _)| (*font, *text))
                    .collect_vec(),
            );
            let images = shader
                .parameters
                .uniforms
                .0
                .iter()
                .filter_map(|(key, uniform)| match uniform {
                    ShaderUniform::Texture(image) => Some((image, key)),
                    _ => None,
                })
                .collect_vec();
            let mut texture_uniforms = SingleUniformVec::default();
            let mut texture_size_uniforms = SingleUniformVec::default();
            for (font, text, key, size_key) in texts.iter() {
                let texture = resources.fonts.get_texture(*font, text);
                texture_uniforms.0.push(SingleUniform::new(key, texture));
                texture_size_uniforms.0.push(SingleUniform::new(
                    size_key.as_str(),
                    texture.and_then(|texture| Some(texture.size().map(|x| x as f32))),
                ));
            }
            for (image, key) in images {
                let texture = resources.image_textures.get_texture(image);
                if texture.is_none() {
                    panic!("Can't find texture {:?}", image);
                }
                texture_uniforms.0.push(SingleUniform::new(key, texture));
            }
            Self::draw_shader(
                &shader,
                framebuffer,
                &resources.geng,
                &resources.camera.camera,
                &resources.shader_programs,
                (texture_uniforms, texture_size_uniforms, uniforms),
            );
        }
    }

    pub fn draw_shader<U>(
        shader: &Shader,
        framebuffer: &mut ugli::Framebuffer,
        geng: &Geng,
        camera: &geng::Camera2d,
        shader_programs: &ShaderPrograms,
        uniforms: U,
    ) where
        U: ugli::Uniforms,
    {
        let mut queue = VecDeque::from([(shader.clone(), shader.parameters.clone())]);
        let mut chain: Vec<(Shader, ShaderParameters)> = default();
        while let Some((shader, parameters)) = queue.pop_front() {
            chain.extend(Self::flatten_shader_chain(shader, parameters))
        }
        for (shader, parameters) in chain.into_iter().rev() {
            let program = shader_programs.get_program(&static_path().join(&shader.path));

            let mut instances_arr: ugli::VertexBuffer<Instance> =
                ugli::VertexBuffer::new_dynamic(geng.ugli(), Vec::new());
            instances_arr.resize(shader.parameters.instances, Instance {});
            let uniforms = (
                geng::camera2d_uniforms(camera, framebuffer.size().map(|x| x as f32)),
                parameters.clone(),
                &uniforms,
            );
            let quad = Self::get_quad(shader.parameters.vertices, &geng);
            ugli::draw(
                framebuffer,
                &program,
                ugli::DrawMode::TriangleStrip,
                ugli::instanced(&quad, &instances_arr),
                uniforms,
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::straight_alpha()),
                    ..default()
                },
            );
        }
    }

    fn flatten_shader_chain(
        mut shader: Shader,
        parameters: ShaderParameters,
    ) -> Vec<(Shader, ShaderParameters)> {
        match shader.chain {
            Some(chain) => {
                shader.chain = None;
                [
                    vec![(shader, parameters.clone())],
                    chain
                        .deref()
                        .into_iter()
                        .map(|shader| {
                            Self::flatten_shader_chain(
                                shader.clone(),
                                ShaderParameters {
                                    uniforms: shader
                                        .parameters
                                        .uniforms
                                        .merge(&parameters.uniforms),
                                    ..shader.parameters.clone()
                                },
                            )
                        })
                        .flatten()
                        .collect_vec(),
                ]
                .concat()
            }
            None => vec![(shader, parameters)],
        }
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
