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

    pub fn draw_unit_with_position(
        &self,
        unit: &Unit,
        template: &UnitTemplate,
        model: Option<&Model>,
        game_time: f64,
        camera: &geng::Camera2d,
        framebuffer: &mut ugli::Framebuffer,
        position: AABB<f32>,
    ) {
        let shader_program = &self.assets.get_render(&unit.render.shader_config); // TODO: move this into to an earlier phase perhaps
        let spawn_scale = match unit.spawn_animation_time_left {
            Some(time) if template.spawn_animation_time > Time::new(0.0) => {
                1.0 - (time / template.spawn_animation_time).as_f32()
            }
            _ => 1.0,
        };
        let quad = shader_program.get_vertices(&self.geng);

        let clan_colors: Vec<Color<f32>> = unit
            .clans
            .iter()
            .map(|clan| self.assets.options.clan_colors[clan])
            .collect();

        // Actual render
        let texture_position = position;
        let texture_size = (texture_position.height() * framebuffer.size().y as f32 / camera.fov
            * 2.0)
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
                u_unit_position: position.center(),
                u_unit_radius: unit.render.radius.as_f32(),
                u_spawn: spawn_scale,
                u_face_dir: vec2(0.0, 0.0),
                u_random: unit.random_number.as_f32(),
                u_action_time: unit.render.last_action_time.as_f32(),
                u_injure_time: unit.render.last_injure_time.as_f32(),
                u_heal_time: unit.render.last_heal_time.as_f32(),
                u_parent_faction: match unit.faction {
                        Faction::Player => 1.0,
                        Faction::Enemy => -1.0,
                    },
                u_clan_color_1: clan_colors.get(0).copied().unwrap_or(Color::WHITE),
                u_clan_color_2: clan_colors.get(1).copied().unwrap_or(Color::WHITE),
                u_clan_color_3: clan_colors.get(2).copied().unwrap_or(Color::WHITE),
                u_clan_count: clan_colors.len(),
                u_ability_ready: 1.0,
                u_health: 1.0,
            },
            geng::camera2d_uniforms(&texture_camera, texture_size.map(|x| x as f32)),
            shader_program.parameters.clone(),
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
                &shader_program.program,
                ugli::DrawMode::TriangleStrip,
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
                    .filter(|config| config.render.is_some())
                    // .and_then(|config| config.render.as_ref())
                    .map(|config| {
                        (
                            self.assets.get_render(config.render.as_ref().unwrap()),
                            status.time,
                            status.status.duration,
                            config.get_color(&self.assets.options),
                        )
                    })
            })
            .collect();
        let status_count = statuses.len();
        for (status_index, (program, status_time, status_duration, status_color)) in
            statuses.into_iter().enumerate()
        {
            let mut new_texture = ugli::Texture::new_uninitialized(self.geng.ugli(), texture_size);
            {
                let mut framebuffer = ugli::Framebuffer::new_color(
                    self.geng.ugli(),
                    ugli::ColorAttachment::Texture(&mut new_texture),
                );
                let framebuffer = &mut framebuffer;
                let status_time = status_time.unwrap_or(0);
                let status_duration = status_duration.unwrap_or_else(|| 1.try_into().unwrap());
                ugli::clear(framebuffer, Some(Color::TRANSPARENT_WHITE), None);
                ugli::draw(
                    framebuffer,
                    &program.program,
                    ugli::DrawMode::TriangleStrip,
                    &quad,
                    (
                        &uniforms,
                        ugli::uniforms! {
                            u_previous_texture: &texture,
                            u_status_count: status_count,
                            u_status_index: status_index,
                            u_status_time: status_time as f32,
                            u_status_duration: u64::from(status_duration) as f32,
                            u_time: game_time,
                            u_color: status_color,
                        },
                        program.parameters,
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

    pub fn draw_unit(
        &self,
        unit: &Unit,
        template: &UnitTemplate,
        model: Option<&Model>,
        game_time: f64,
        camera: &geng::Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let position = AABB::point(unit.render.render_position.map(|x| x.as_f32()))
            .extend_uniform(unit.render.radius.as_f32() * 2.0); // TODO: configuring?
        self.draw_unit_with_position(
            unit,
            template,
            model,
            game_time,
            camera,
            framebuffer,
            position,
        )
    }
}
