use super::*;

#[derive(Clone)]
struct Text {
    position: Vec2<f32>,
    velocity: Vec2<f32>,
    time: f32,
    text: String,
    color: Color<f32>,
}

#[derive(Clone)]
pub struct RenderModel {
    texts: Vec<Text>,
}

impl RenderModel {
    pub fn new() -> Self {
        Self { texts: Vec::new() }
    }
    pub fn update(&mut self, delta_time: f32) {
        for text in &mut self.texts {
            text.time += delta_time;
            text.position += text.velocity * delta_time;
        }
        self.texts.retain(|text| text.time < 1.0);
    }
    pub fn add_text(&mut self, position: Vec2<Coord>, text: &str, color: Color<f32>) {
        let velocity = vec2(0.2, 0.0).rotate(global_rng().gen_range(0.0..2.0 * f32::PI));
        self.texts.push(Text {
            position: position.map(|x| x.as_f32()) + velocity,
            time: 0.0,
            velocity,
            text: text.to_owned(),
            color,
        });
    }
}

pub struct Render {
    geng: Geng,
    camera: geng::Camera2d,
    assets: Rc<Assets>,
    unit_render: UnitRender,
}

pub struct UnitRender {
    pub geng: Geng,
    pub assets: Rc<Assets>,
}

impl Render {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: Config) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            camera: geng::Camera2d {
                center: vec2(0.0, 0.0),
                rotation: 0.0,
                fov: config.fov,
            },
            unit_render: UnitRender::new(geng, assets),
        }
    }
    pub fn draw(
        &mut self,
        game_time: f32,
        model: &Model,
        render_model: &RenderModel,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        for unit in itertools::chain![&model.units, &model.spawning_units] {
            let template = &self.assets.units[&unit.unit_type];

            let render = self.assets.get_render(&unit.render); // TODO: move this into to an earlier phase perhaps
            self.draw_unit(unit, template, model, game_time, framebuffer);
        }
        for projectile in &model.projectiles {
            let render = self.assets.get_render(&projectile.render_config); // TODO: move this into to an earlier phase perhaps
            self.draw_projectile(projectile, &render, game_time, framebuffer);
        }
        for particle in &model.particles {
            let render = self.assets.get_render(&particle.render_config); // TODO: move this into to an earlier phase perhaps
            self.draw_particle(particle, &render, game_time, framebuffer);
        }
        for text in &render_model.texts {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Text::unit(&**self.geng.default_font(), &text.text, text.color)
                    .scale_uniform(0.2)
                    .translate(text.position),
            );
        }
    }

    fn draw_unit(
        &self,
        unit: &Unit,
        template: &UnitTemplate,
        model: &Model,
        game_time: f32,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.unit_render.draw_unit(
            unit,
            template,
            Some(model),
            game_time,
            &self.camera,
            framebuffer,
        );
    }

    fn draw_particle(
        &self,
        particle: &Particle,
        render_mode: &RenderMode,
        game_time: f32,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        match render_mode {
            RenderMode::Circle { color } => {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Ellipse::circle(
                        particle.position.map(|x| x.as_f32()),
                        particle.radius.as_f32(),
                        *color,
                    ),
                );
            }
            RenderMode::Texture { texture } => {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit(&**texture)
                        .scale_uniform(particle.radius.as_f32())
                        .translate(particle.position.map(|x| x.as_f32())),
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
                let model_matrix = Mat3::translate(particle.position.map(|x| x.as_f32()))
                    * Mat3::scale_uniform(particle.radius.as_f32());

                ugli::draw(
                    framebuffer,
                    program,
                    ugli::DrawMode::TriangleFan,
                    &quad,
                    (
                        ugli::uniforms! {
                            u_time: game_time,
                            u_unit_position: particle.position.map(|x| x.as_f32()),
                            u_unit_radius: particle.radius.as_f32(),
                            u_spawn: (particle.time_left / particle.duration).as_f32(),
                            u_action: 0.0,
                            u_clan_color_1: Color::WHITE,
                            u_clan_color_2: Color::WHITE,
                            u_clan_color_3: Color::WHITE,
                            u_clan_count: 0,
                        },
                        geng::camera2d_uniforms(&self.camera, framebuffer_size.map(|x| x as f32)),
                        parameters,
                    ),
                    ugli::DrawParameters {
                        blend_mode: Some(default()),
                        ..default()
                    },
                );
            }
        }
    }

    fn draw_projectile(
        &self,
        projectile: &Projectile,
        render_mode: &RenderMode,
        game_time: f32,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        const RADIUS: f32 = 0.35;
        match render_mode {
            RenderMode::Circle { color } => {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Ellipse::circle(
                        projectile.position.map(|x| x.as_f32()),
                        RADIUS,
                        *color,
                    ),
                );
            }
            RenderMode::Texture { texture } => {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit(&**texture)
                        .scale_uniform(RADIUS)
                        .translate(projectile.position.map(|x| x.as_f32())),
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
                let model_matrix = Mat3::translate(projectile.position.map(|x| x.as_f32()))
                    * Mat3::scale_uniform(RADIUS);
                let velocity = ((projectile.target_position - projectile.position)
                    .normalize_or_zero()
                    * projectile.speed)
                    .map(|x| x.as_f32());

                ugli::draw(
                    framebuffer,
                    program,
                    ugli::DrawMode::TriangleFan,
                    &quad,
                    (
                        ugli::uniforms! {
                            u_time: game_time,
                            u_unit_position: projectile.position.map(|x| x.as_f32()),
                            u_unit_radius: RADIUS,
                            u_velocity: velocity,
                        },
                        geng::camera2d_uniforms(&self.camera, framebuffer_size.map(|x| x as f32)),
                        parameters,
                    ),
                    ugli::DrawParameters {
                        blend_mode: Some(default()),
                        ..default()
                    },
                );
            }
        }
    }
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
        let attack_scale = match &unit.action_state {
            ActionState::Start { time, .. } => {
                1.0 - 0.25 * (*time / unit.action.animation_delay).as_f32()
            }
            _ => 1.0,
        };

        match render_mode {
            RenderMode::Circle { color } => {
                self.geng.draw_2d(
                    framebuffer,
                    camera,
                    &draw_2d::Ellipse::circle(
                        unit.position.map(|x| x.as_f32()),
                        unit.radius.as_f32() * attack_scale * spawn_scale,
                        {
                            let mut color = *color;
                            if unit
                                .all_statuses
                                .iter()
                                .any(|status| status.r#type() == StatusType::Freeze)
                            {
                                color = Color::CYAN;
                            }
                            if unit
                                .all_statuses
                                .iter()
                                .any(|status| matches!(status, Status::Slow { .. }))
                            {
                                color = Color::GRAY;
                            }
                            color
                        },
                    ),
                );
            }
            RenderMode::Texture { texture } => {
                self.geng.draw_2d(
                    framebuffer,
                    camera,
                    &draw_2d::TexturedQuad::unit(&**texture)
                        .scale_uniform(unit.radius.as_f32() * attack_scale * spawn_scale)
                        .translate(unit.position.map(|x| x.as_f32())),
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
                let model_matrix = Mat3::translate(unit.position.map(|x| x.as_f32()))
                    * Mat3::scale_uniform(unit.radius.as_f32() * attack_scale * spawn_scale);

                let mut clans: Vec<Clan> = unit.clans.iter().copied().collect();
                let clan_colors: Vec<Color<f32>> = clans
                    .iter()
                    .map(|clan| self.assets.options.clan_colors[clan])
                    .collect();

                let (action_time, target) = match &unit.action_state {
                    ActionState::Start { time, target } => (
                        (*time / unit.action.animation_delay).as_f32(),
                        model
                            .and_then(|model| model.units.get(&target))
                            .map(|unit| unit),
                    ),
                    _ => (0.0, None),
                };

                let target_dir = target
                    .map_or(Vec2::ZERO, |target| {
                        (target.position - unit.position).normalize_or_zero()
                    })
                    .map(|x| x.as_f32());

                let mut is_ability_ready = 0.0; // TODO: rewrite please
                if let Some(ability) = &template.ability {
                    is_ability_ready = match unit.ability_cooldown {
                        Some(time) if time > Time::new(0.0) => 0.0,
                        _ => 1.0,
                    };
                }

                // Actual render
                let texture_position = AABB::point(unit.position.map(|x| x.as_f32()))
                    .extend_uniform(unit.radius.as_f32() * 2.0); // TODO: configuring?
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
                        u_unit_position: unit.position.map(|x| x.as_f32()),
                        u_unit_radius: unit.radius.as_f32(),
                        u_spawn: spawn_scale,
                        u_action: action_time,
                        u_cooldown: unit.action.cooldown.as_f32(),
                        u_animation_delay: unit.action.animation_delay.as_f32(),
                        u_face_dir: unit.face_dir.map(|x| x.as_f32()),
                        u_random: unit.random_number.as_f32(),
                        u_action_time: unit.last_action_time.as_f32(),
                        u_injure_time: unit.last_injure_time.as_f32(),
                        u_clan_color_1: clan_colors.get(0).copied().unwrap_or(Color::WHITE),
                        u_clan_color_2: clan_colors.get(1).copied().unwrap_or(Color::WHITE),
                        u_clan_color_3: clan_colors.get(2).copied().unwrap_or(Color::WHITE),
                        u_clan_count: clan_colors.len(),
                        u_ability_ready: is_ability_ready,
                        u_health: unit.health.as_f32() / unit.max_hp.as_f32(),
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

                let mut statuses: std::vec::Vec<&StatusRender> = unit
                    .all_statuses
                    .iter()
                    .filter_map(|status| {
                        let status_type = status.r#type();
                        if let Some(program) = self.assets.statuses.get(&status_type) {
                            Some(program)
                        } else {
                            None
                        }
                    })
                    .collect();
                let status_count = statuses.len();
                for (
                    status_index,
                        StatusRender {
                            shader: program,
                            parameters,
                        },
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
                        ugli::clear(framebuffer, Some(Color::TRANSPARENT_WHITE), None);
                        ugli::draw(
                            framebuffer,
                            program,
                            ugli::DrawMode::TriangleFan,
                            &quad,
                            (
                                &uniforms,
                                ugli::uniforms! {
                                    u_previous_texture: &texture,
                                    u_status_count: status_count,
                                    u_status_index: status_index,
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
