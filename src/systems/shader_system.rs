use geng::prelude::{itertools::Itertools, ugli::SingleUniform};
use legion::EntityStore;

use super::*;

pub struct ShaderSystem {}

impl System for ShaderSystem {
    fn draw(
        &self,
        world: &legion::World,
        resources: &mut Resources,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.draw_all_shaders(world, resources, framebuffer);
    }

    fn update(&mut self, _: &mut legion::World, _: &mut Resources) {}
}

impl ShaderSystem {
    pub fn new() -> Self {
        Self {}
    }

    /// Get Shader component and merge Context into it's vars if any
    pub fn get_entity_shader(
        world: &legion::World,
        entity: legion::Entity,
        context: Option<&Context>,
    ) -> Option<Shader> {
        match world.entry_ref(entity).ok().and_then(|x| {
            x.get_component::<Shader>()
                .ok()
                .cloned()
                .and_then(|shader| Some((x.get_component::<EntityComponent>().unwrap().ts, shader)))
        }) {
            Some((ts, mut shader)) => Some({
                if let Some(context) = context {
                    shader
                        .parameters
                        .uniforms
                        .merge_mut(&context.vars.clone().into(), true);
                    shader.ts = ts;
                }
                shader
            }),
            None => None,
        }
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
                .filter(
                    !component::<UnitComponent>()
                        & !component::<CorpseComponent>()
                        & !component::<TapeEntityComponent>()
                        & component::<Shader>(),
                )
                .iter(world)
                .map(|entity| {
                    (
                        entity.entity,
                        Self::get_entity_shader(
                            world,
                            entity.entity,
                            ContextSystem::try_get_context(entity.entity, world)
                                .ok()
                                .as_ref(),
                        )
                        .unwrap(),
                    )
                }),
        );

        let shaders = TapePlayerSystem::get_shaders(world_shaders, resources)
            .into_iter()
            .chain(resources.frame_shaders.drain(..))
            .sorted_by_key(|x| (x.layer.index(), x.order, x.ts))
            .collect_vec();

        let game_time = match resources.tape_player.mode {
            TapePlayMode::Play => resources.tape_player.head,
            TapePlayMode::Stop { .. } => resources.global_time,
        };
        let aspect_ratio = {
            let size = resources.camera.framebuffer_size;
            size.x / size.y
        };
        for shader in shaders {
            let uniforms = ugli::uniforms!(
                u_game_time: game_time,
                u_global_time: resources.global_time,
                u_aspect_ratio: aspect_ratio,
            );
            Self::draw_shader(shader, framebuffer, resources, uniforms);
        }
    }

    pub fn draw_shader<U>(
        shader: Shader,
        framebuffer: &mut ugli::Framebuffer,
        resources: &mut Resources,
        uniforms: U,
    ) where
        U: ugli::Uniforms,
    {
        // todo: measure top avg execution time
        let chain = Self::flatten_shader_chain(shader);
        for shader in chain.into_iter() {
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
                if text.is_empty() {
                    continue;
                }
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

            Self::draw_shader_single(
                &shader,
                framebuffer,
                resources,
                (&uniforms, &texture_uniforms, &texture_size_uniforms),
            );
        }
    }

    pub fn draw_shader_single<U>(
        shader: &Shader,
        framebuffer: &mut ugli::Framebuffer,
        resources: &Resources,
        uniforms: U,
    ) where
        U: ugli::Uniforms,
    {
        let geng = resources.geng.as_ref().unwrap();
        let program = resources
            .shader_programs
            .get_program(&static_path().join(&shader.path));
        let mut instances_arr: ugli::VertexBuffer<Instance> =
            ugli::VertexBuffer::new_dynamic(geng.ugli(), Vec::new());
        instances_arr.resize(shader.parameters.instances, Instance {});
        let uniforms = (
            geng::camera2d_uniforms(
                &resources.camera.camera,
                framebuffer.size().map(|x| x as f32),
            ),
            shader.parameters.clone(),
            &uniforms,
        );
        let quad = Self::get_quad(shader.parameters.vertices, geng);
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

    pub fn flatten_shader_chain(mut shader: Shader) -> Vec<Shader> {
        let mut before = shader.chain_before.drain(..).collect_vec();
        before.iter_mut().for_each(|x| {
            x.parameters
                .uniforms
                .merge_mut(&shader.parameters.uniforms, false);
        });

        let mut after = shader.chain_after.drain(..).collect_vec();
        after.iter_mut().for_each(|x| {
            x.parameters
                .uniforms
                .merge_mut(&shader.parameters.uniforms, false);
        });

        [before, vec![shader], after].concat()
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
