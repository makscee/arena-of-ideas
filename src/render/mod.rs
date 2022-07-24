use super::*;

mod field;
mod particle;
mod text;
mod unit;

use text::*;
pub use unit::*;

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
    camera: geng::Camera2d,
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
    pub fn draw(&mut self, game_time: f32, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        self.draw_field(&self.assets.field_render, game_time, framebuffer);
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
        }
        for particle in &model.particles {
            if particle.delay <= Time::new(0.0) {
                let render = self.assets.get_render(&particle.render_config); // TODO: move this into to an earlier phase perhaps
                self.draw_particle(particle, &render, game_time, framebuffer);
            }
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
        let tick_text = model.current_tick_num.to_string();
        let text_scale = f32::max(
            1.1 - (model.current_tick.tick_time.as_f32()) / TICK_TIME,
            1.0,
        );
        self.geng.draw_2d(
            framebuffer,
            &self.camera,
            &draw_2d::Text::unit(&**self.geng.default_font(), &tick_text, Color::WHITE)
                .scale_uniform(0.3 * text_scale)
                .translate(vec2(0.0, self.camera.fov * 0.35)),
        );
    }
}
