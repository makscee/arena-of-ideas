use super::*;

pub struct UnitRender {
    pub geng: Geng,
    pub assets: Rc<Assets>,
}

impl UnitRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
        }
    }

    pub fn draw_unit(
        &self,
        unit: &Unit,
        template: &UnitTemplate,
        model: Option<&Model>,
        game_time: f32,
        camera: &geng::Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let render_mode = &self.assets.get_render(&unit.render); // TODO: move this into to an earlier phase perhaps
        let spawn_scale = match unit.spawn_animation_time_left {
            Some(time) if template.spawn_animation_time > Time::new(0.0) => {
                1.0 - (time / template.spawn_animation_time).as_f32()
            }
            _ => 1.0,
        };

        match render_mode {
            RenderMode::Circle { color } => {
                let unit_pos = unit.position.to_world_f32();
                self.geng.draw_2d(
                    framebuffer,
                    camera,
                    &draw_2d::Ellipse::circle(
                        unit_pos,
                        unit.stats.radius.as_f32() * spawn_scale,
                        {
                            let mut color = *color;
                            // TODO: reimplement
                            // if unit
                            //     .all_statuses
                            //     .iter()
                            //     .any(|status| status.r#type() == StatusType::Freeze)
                            // {
                            //     color = Color::CYAN;
                            // }
                            // if unit
                            //     .all_statuses
                            //     .iter()
                            //     .any(|status| matches!(status, StatusOld::Slow { .. }))
                            // {
                            //     color = Color::GRAY;
                            // }
                            color
                        },
                    ),
                );
            }
            RenderMode::Texture { texture } => {
                let unit_pos = unit.position.to_world_f32();
                self.geng.draw_2d(
                    framebuffer,
                    camera,
                    &draw_2d::TexturedQuad::unit(&**texture)
                        .scale_uniform(unit.stats.radius.as_f32() * spawn_scale)
                        .translate(unit_pos),
                );
            }
            RenderMode::Shader {
                program,
                parameters,
            } => {
                let quad = ugli::VertexBuffer::new_dynamic(
                    self.geng.ugli(),
                    vec![
                        draw_2d::Vertex {
                            a_pos: vec2(-1.0, -1.0),
                        },
                        draw_2d::Vertex {
                            a_pos: vec2(1.0, -1.0),
                        },
                        draw_2d::Vertex {
                            a_pos: vec2(1.0, 1.0),
                        },
                        draw_2d::Vertex {
                            a_pos: vec2(-1.0, 1.0),
                        },
                    ],
                );
                let framebuffer_size = framebuffer.size();
                let unit_pos = unit.position.to_world_f32();
                let model_matrix = Mat3::translate(unit_pos)
                    * Mat3::scale_uniform(unit.stats.radius.as_f32() * spawn_scale);

                let mut clans: Vec<Clan> = unit.clans.iter().copied().collect();
                let clan_colors: Vec<Color<f32>> = clans
                    .iter()
                    .map(|clan| self.assets.options.clan_colors[clan])
                    .collect();

                let target = match &unit.action_state {
                    ActionState::Start { target } => model
                        .and_then(|model| model.units.get(&target))
                        .map(|unit| unit),

                    _ => None,
                };

                let mut is_ability_ready: f32 = 0.0; // TODO: rewrite please
                if let Some(ability) = &template.ability {
                    is_ability_ready = match unit.ability_cooldown {
                        Some(time) if time > Time::new(0.0) => 0.0,
                        _ => 1.0,
                    };
                }

                // Actual render
                let texture_position =
                    AABB::point(unit_pos).extend_uniform(unit.stats.radius.as_f32() * 2.0); // TODO: configuring?
                let texture_size =
                    (texture_position.height() * framebuffer.size().y as f32 / camera.fov * 2.0)
                        .max(1.0) as usize;
                let texture_size = vec2(texture_size, texture_size);
                let texture_camera = geng::Camera2d {
                    center: texture_position.center(),
                    rotation: 0.0,
                    fov: texture_position.height(),
                };
                let uniforms = (
                    ugli::uniforms! {
                        u_time: game_time,
                        u_unit_position: unit_pos,
                        u_unit_radius: unit.stats.radius.as_f32(),
                        u_spawn: spawn_scale,
                        u_cooldown: unit.action.cooldown as f32,
                        u_face_dir: unit.face_dir.map(|x| x.as_f32()),
                        u_random: unit.random_number.as_f32(),
                        u_action_time: unit.last_action_time.as_f32(),
                        u_injure_time: unit.last_injure_time.as_f32(),
                        u_clan_color_1: clan_colors.get(0).copied().unwrap_or(Color::WHITE),
                        u_clan_color_2: clan_colors.get(1).copied().unwrap_or(Color::WHITE),
                        u_clan_color_3: clan_colors.get(2).copied().unwrap_or(Color::WHITE),
                        u_clan_count: clan_colors.len(),
                        u_ability_ready: is_ability_ready,
                        u_health: unit.stats.health.as_f32() / unit.stats.max_hp.as_f32(),
                    },
                    geng::camera2d_uniforms(&texture_camera, texture_size.map(|x| x as f32)),
                    parameters,
                );

                let mut texture = ugli::Texture::new_uninitialized(self.geng.ugli(), texture_size);
                {
                    let mut framebuffer = ugli::Framebuffer::new_color(
                        self.geng.ugli(),
                        ugli::ColorAttachment::Texture(&mut texture),
                    );
                    let framebuffer = &mut framebuffer;
                    ugli::clear(framebuffer, Some(Color::TRANSPARENT_WHITE), None);
                    ugli::draw(
                        framebuffer,
                        program,
                        ugli::DrawMode::TriangleFan,
                        &quad,
                        &uniforms,
                        ugli::DrawParameters {
                            // blend_mode: Some(default()),
                            ..default()
                        },
                    );
                }

                let mut statuses: Vec<_> = unit
                    .all_statuses
                    .iter()
                    .filter_map(|status| {
                        self.assets
                            .statuses
                            .get(&status.status.name)
                            .and_then(|config| config.render.as_ref())
                            .map(|render| {
                                (
                                    self.assets.get_status_render(render),
                                    status.time,
                                    status.status.duration,
                                )
                            })
                    })
                    .collect();
                let status_count = statuses.len();
                for (
                    status_index,
                    (
                        ShaderConfig {
                            shader: program,
                            parameters,
                        },
                        status_time,
                        status_duration,
                    ),
                ) in statuses.into_iter().enumerate()
                {
                    let mut new_texture =
                        ugli::Texture::new_uninitialized(self.geng.ugli(), texture_size);
                    {
                        let mut framebuffer = ugli::Framebuffer::new_color(
                            self.geng.ugli(),
                            ugli::ColorAttachment::Texture(&mut new_texture),
                        );
                        let framebuffer = &mut framebuffer;
                        let status_time = match status_time {
                            Some(status_time) => status_time,
                            None => r32(0.0),
                        };
                        let status_duration = match status_duration {
                            Some(status_duration) => status_duration,
                            None => r32(0.0),
                        };
                        ugli::clear(framebuffer, Some(Color::TRANSPARENT_WHITE), None);
                        ugli::draw(
                            framebuffer,
                            &*program,
                            ugli::DrawMode::TriangleFan,
                            &quad,
                            (
                                &uniforms,
                                ugli::uniforms! {
                                    u_previous_texture: &texture,
                                    u_status_count: status_count,
                                    u_status_index: status_index,
                                    u_status_time: status_time.as_f32(),
                                    u_status_duration: status_duration.as_f32(),
                                    u_time: game_time,
                                },
                                parameters,
                            ),
                            ugli::DrawParameters {
                                // blend_mode: Some(default()),
                                ..default()
                            },
                        );
                    }
                    texture = new_texture;
                }
                self.geng.draw_2d(
                    framebuffer,
                    camera,
                    &draw_2d::TexturedQuad::new(texture_position, texture),
                );
            }
        }
    }
}
