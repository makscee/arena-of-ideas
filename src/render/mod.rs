use super::*;

mod particle;
mod projectile;
mod unit;
mod field;

pub use unit::*;

#[derive(Clone)]
struct Text {
    position: Vec2<f32>,
    velocity: Vec2<f32>,
    time: f32,
    text: String,
    color: Color<f32>,
    scale: f32,
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
            text.time += delta_time * 0.8;
            text.position += text.velocity * delta_time;
            text.scale = 1.0 - text.time;
        }
        self.texts.retain(|text| text.time < 1.0);
    }
    pub fn add_text(&mut self, position: Position, text: &str, color: Color<f32>) {
        let velocity = vec2(0.7, 0.0).rotate(global_rng().gen_range(0.0..2.0 * f32::PI));
        self.texts.push(Text {
            position: position.to_world_f32() + velocity,
            time: 0.0,
            velocity,
            text: text.to_owned(),
            color,
            scale: 1.0,
        });
    }
}

pub struct Render {
    geng: Geng,
    camera: geng::Camera2d,
    assets: Rc<Assets>,
    unit_render: UnitRender,
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
                    .scale_uniform(0.35 * text.scale)
                    .translate(text.position),
            );
        }
    }
}
