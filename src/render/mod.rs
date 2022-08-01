use super::*;

mod field;
mod particle;
mod text;
mod unit;

use geng::Draw2d;
use text::*;
pub use unit::*;

const DESCRIPTION_WIDTH: f32 = 1.0;
const DESCRIPTION_MARGIN: f32 = 0.1;
const FONT_SIZE: f32 = 0.2;
const ARROW_SIZE: f32 = 0.15;

const STATUS_DESC_FOREGROUND: Color<f32> = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};
const STATUS_DESC_BACKGROUND: Color<f32> = Color {
    r: 0.2,
    g: 0.2,
    b: 0.2,
    a: 1.0,
};
const DAMAGE_DESC_FOREGROUND: Color<f32> = Color {
    r: 0.3,
    g: 0.3,
    b: 0.3,
    a: 1.0,
};

#[derive(Clone)]
pub struct RenderModel {
    text_blocks: HashMap<Position, TextBlock>,
    texts: Vec<Text>,
}

pub enum TextType {
    Damage,
    Heal,
    Status,
    Aoe,
}

impl RenderModel {
    pub fn new() -> Self {
        Self {
            text_blocks: HashMap::new(),
            texts: Vec::new(),
        }
    }
    pub fn update(&mut self, delta_time: f32) {
        for text_block in self.text_blocks.values_mut() {
            text_block.update(delta_time);
        }
        for text in &mut self.texts {
            text.update(delta_time);
        }
        self.texts.retain(Text::is_alive);
    }
    pub fn add_text(
        &mut self,
        position: Position,
        text: impl Into<String>,
        color: Color<f32>,
        text_type: TextType,
    ) {
        let text_block = self
            .text_blocks
            .entry(position)
            .or_insert_with(|| TextBlock::new(position.to_world_f32()));
        match text_type {
            TextType::Damage => text_block.add_text_top(text, color),
            TextType::Heal => text_block.add_text_bottom(text, color),
            TextType::Status | TextType::Aoe => {
                self.add_text_random(position.to_world_f32(), text, color)
            }
        }
    }
}

pub struct Render {
    geng: Geng,
    pub camera: geng::Camera2d,
    assets: Rc<Assets>,
    unit_render: UnitRender,
}

impl Render {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: &Config) -> Self {
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
    pub fn draw(&mut self, game_time: f64, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        self.draw_field(
            &self.assets.renders_config.field,
            game_time,
            model,
            framebuffer,
        );
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
        let mouse_world_pos = self.camera.screen_to_world(
            framebuffer_size,
            self.geng.window().mouse_pos().map(|x| x as f32),
        );

        let mut hovered_unit = None;
        for unit in &model.units {
            let template = &self.assets.units[&unit.unit_type];

            let render = self.assets.get_render(&unit.render); // TODO: move this into to an earlier phase perhaps
            self.unit_render.draw_unit(
                unit,
                template,
                Some(model),
                game_time,
                &self.camera,
                framebuffer,
            );

            // Draw damage and health
            let unit_aabb = AABB::point(unit.position.to_world_f32())
                .extend_uniform(unit.stats.radius.as_f32() / 2.3);
            let size = unit.stats.radius.as_f32() * 0.3;
            let damage = AABB::point(unit_aabb.bottom_left())
                .extend_right(size)
                .extend_up(size);
            let health = AABB::point(unit_aabb.bottom_right())
                .extend_left(size)
                .extend_up(size);

            draw_2d::TexturedQuad::new(damage, self.assets.swords_emblem.clone()).draw_2d(
                &self.geng,
                framebuffer,
                &self.camera,
            );
            draw_2d::TexturedQuad::new(health, self.assets.hearts.clone()).draw_2d(
                &self.geng,
                framebuffer,
                &self.camera,
            );
            let text_color = Color::try_from("#e6e6e6").unwrap();
            draw_2d::Text::unit(
                self.geng.default_font().clone(),
                format!("{:.0}", unit.stats.base_damage),
                text_color,
            )
            .fit_into(damage)
            .draw_2d(&self.geng, framebuffer, &self.camera);
            draw_2d::Text::unit(
                self.geng.default_font().clone(),
                format!("{:.0}", unit.stats.health),
                text_color,
            )
            .fit_into(health)
            .draw_2d(&self.geng, framebuffer, &self.camera);

            // On unit hover
            if (mouse_world_pos - unit.render_position.map(|x| x.as_f32())).len()
                < unit.stats.radius.as_f32() / 2.0
            {
                // Draw extra ui: statuses descriptions, damage/heal descriptions
                hovered_unit = Some(unit);
            }
        }

        // Draw slots
        let factions = vec![Faction::Player, Faction::Enemy];
        let shader_program = &self.assets.renders_config.slot;
        for faction in factions {
            for i in 0..SIDE_SLOTS {
                let quad = shader_program.get_vertices(&self.geng);
                let framebuffer_size = framebuffer.size();
                let position = Position {
                    x: i as i64,
                    side: faction,
                    height: 0,
                }
                .to_world_f32();
                let empty = model
                    .units
                    .iter()
                    .any(|unit| unit.position.x == i as i64 && unit.faction == faction);

                ugli::draw(
                    framebuffer,
                    &shader_program.program,
                    ugli::DrawMode::TriangleStrip,
                    &quad,
                    (
                        ugli::uniforms! {
                            u_time: game_time,
                            u_unit_position: position,
                            u_parent_faction: match faction {
                                Faction::Player => 1.0,
                                Faction::Enemy => -1.0,
                            },
                            u_empty: if empty { 1.0 } else { 0.0 },
                        },
                        geng::camera2d_uniforms(&self.camera, framebuffer_size.map(|x| x as f32)),
                        &shader_program.parameters,
                    ),
                    ugli::DrawParameters {
                        blend_mode: Some(default()),
                        ..default()
                    },
                );
            }
        }

        for particle in &model.particles {
            if particle.delay <= Time::new(0.0) {
                let render = self.assets.get_render(&particle.render_config); // TODO: move this into to an earlier phase perhaps
                self.draw_particle(particle, &render, game_time, framebuffer);
            }
        }

        if let Some(unit) = hovered_unit {
            self.draw_statuses_desc(unit, framebuffer);
            self.draw_damage_heal_desc(unit.position, framebuffer);
        }

        for text in model
            .render_model
            .text_blocks
            .values()
            .flat_map(|text_block| text_block.texts())
        {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Text::unit(&**self.geng.default_font(), &text.text, text.color)
                    .scale_uniform(0.15 * text.scale)
                    .translate(text.position),
            );
        }

        // Tick indicator
        let tick_text = model.current_tick.tick_num.to_string();
        let text_scale = f32::max(1.1 - (model.current_tick.tick_time.as_f32()) / 2.0, 1.0);
        self.geng.draw_2d(
            framebuffer,
            &self.camera,
            &draw_2d::Text::unit(&**self.geng.default_font(), &tick_text, Color::WHITE)
                .scale_uniform(0.3 * text_scale)
                .translate(vec2(0.0, self.camera.fov * 0.35)),
        );
    }

    fn draw_statuses_desc(&self, unit: &Unit, framebuffer: &mut ugli::Framebuffer) {
        let font_size = FONT_SIZE;
        let descriptions: Vec<_> = unit
            .all_statuses
            .iter()
            .filter_map(|status| {
                self.assets
                    .statuses
                    .get(&status.status.name)
                    .filter(|config| !config.hidden)
                    .map(|config| {
                        let lines = wrap_text(
                            self.geng.default_font().clone(),
                            &config.description,
                            font_size,
                            DESCRIPTION_WIDTH,
                        )
                        .expect("Failed to measure text");
                        let height = (lines.len() as f32 + 1.5) * font_size;
                        (status, config, lines, height)
                    })
            })
            .collect();
        if descriptions.is_empty() {
            return;
        }
        let total_height = descriptions.iter().map(|(_, _, _, h)| *h).sum::<f32>()
            + (descriptions.len() + 1) as f32 * DESCRIPTION_MARGIN;
        let top_left = vec2(
            unit.render_position.x.as_f32()
                + unit.stats.radius.as_f32() / 2.0
                + DESCRIPTION_MARGIN
                + ARROW_SIZE,
            unit.render_position.y.as_f32() + total_height / 2.0,
        );
        let bottom_right =
            top_left + vec2(DESCRIPTION_WIDTH + DESCRIPTION_MARGIN * 2.0, -total_height);

        draw_2d::Quad::new(
            AABB::from_corners(top_left, bottom_right),
            STATUS_DESC_BACKGROUND,
        )
        .draw_2d(&self.geng, framebuffer, &self.camera);
        let left_mid = vec2(top_left.x, unit.render_position.y.as_f32());
        draw_2d::Polygon::new(
            vec![
                left_mid - vec2(ARROW_SIZE, 0.0),
                left_mid - vec2(0.0, ARROW_SIZE),
                left_mid + vec2(0.0, ARROW_SIZE),
            ],
            STATUS_DESC_BACKGROUND,
        )
        .draw_2d(&self.geng, framebuffer, &self.camera);

        let mut text_pos = top_left
            + vec2(
                DESCRIPTION_MARGIN + DESCRIPTION_WIDTH / 2.0,
                -DESCRIPTION_MARGIN - font_size,
            );
        for (status, config, description, height) in descriptions {
            draw_2d::Quad::new(
                AABB::point(text_pos)
                    .extend_symmetric(vec2(DESCRIPTION_WIDTH / 2.0, 0.0))
                    .extend_up(font_size)
                    .extend_down(height - font_size),
                STATUS_DESC_FOREGROUND,
            )
            .draw_2d(&self.geng, framebuffer, &self.camera);

            let color = config.color.unwrap_or_else(|| {
                *self
                    .assets
                    .options
                    .clan_colors
                    .get(&config.clan_origin)
                    .unwrap_or_else(|| panic!("Failed to find clan ({}) color", config.clan_origin))
            });
            let font = self.geng.default_font().clone();
            font.draw(
                framebuffer,
                &self.camera,
                &status.status.name,
                text_pos,
                geng::TextAlign::CENTER,
                font_size,
                color,
            );
            draw_lines(
                font,
                &description,
                font_size,
                text_pos,
                Color::WHITE,
                framebuffer,
                &self.camera,
            );

            text_pos.y -= height + DESCRIPTION_MARGIN;
        }
    }

    fn draw_damage_heal_desc(&self, position: Position, framebuffer: &mut ugli::Framebuffer) {
        // TODO
    }
}

pub fn wrap_text(
    font: impl std::borrow::Borrow<geng::Font>,
    text: impl AsRef<str>,
    font_size: f32,
    target_width: f32,
) -> Option<Vec<String>> {
    let max_width = target_width;
    let font = font.borrow();
    let text = text.as_ref();

    let measure = |text| font.measure_at(text, Vec2::ZERO, font_size);

    let mut lines = Vec::new();
    for line in text.lines() {
        let mut words = line.split_whitespace();
        let mut line = String::new();
        let mut line_width = 0.0;
        if let Some(word) = words.next() {
            let width = measure(word)?.width();
            line_width += width;
            line += word;
        }
        for word in words {
            let width = measure(word)?.width();
            if line_width + width <= max_width {
                line_width += width;
                line += " ";
                line += word;
            } else {
                lines.push(line);
                line = String::new();
                line_width = width;
                line += word;
                continue;
            }
        }
        lines.push(line);
    }
    Some(lines)
}

pub fn draw_lines(
    font: impl std::borrow::Borrow<geng::Font>,
    lines: &[impl AsRef<str>],
    font_size: f32,
    top_anchor: Vec2<f32>,
    color: Color<f32>,
    framebuffer: &mut ugli::Framebuffer,
    camera: &impl geng::AbstractCamera2d,
) {
    let font = font.borrow();
    let mut pos = vec2(top_anchor.x, top_anchor.y - font_size);
    for line in lines {
        font.draw(
            framebuffer,
            camera,
            line.as_ref(),
            pos,
            geng::TextAlign::CENTER,
            font_size,
            color,
        );
        pos.y -= font_size;
    }
}

pub fn draw_text_wrapped(
    font: impl std::borrow::Borrow<geng::Font>,
    text: impl AsRef<str>,
    font_size: f32,
    target: AABB<f32>,
    color: Color<f32>,
    framebuffer: &mut ugli::Framebuffer,
    camera: &impl geng::AbstractCamera2d,
) -> Option<()> {
    let font = font.borrow();
    let lines = wrap_text(font, text, font_size, target.width())?;
    draw_lines(
        font,
        &lines,
        font_size,
        vec2(target.center().x, target.y_max),
        color,
        framebuffer,
        camera,
    );
    Some(())
}
