use geng::TextAlign;
use tween::{QuartInOut, Tween};

use super::*;

impl Render {
    pub(super) fn draw_panel(
        &self,
        panel: &Panel,
        game_time: f32,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !panel.visible {
            return;
        }
        const HEIGHT: f32 = 1.5;
        const FONT_SIZE: f32 = 1.0;
        const ANIMATION_PART: f32 = 0.2;
        let t = (panel.time_passed.as_f32() / panel.duration.as_f32()) * 2.0 - 1.0;
        let t = t * t * t * t * t * t;

        let aabb = AABB::point(Vec2::ZERO)
            .extend_symmetric(vec2(self.camera.fov, HEIGHT * (1.0 - t.abs())))
            .translate(vec2(0.0, 2.0));
        draw_2d::Quad::new(aabb, panel.color).draw_2d(&self.geng, framebuffer, &self.camera);
        draw_2d::Quad::new(aabb.extend_symmetric(vec2(0.0, -0.2)), Rgba::BLACK).draw_2d(
            &self.geng,
            framebuffer,
            &self.camera,
        );

        let font_size = 1.0;
        let font = self.geng.default_font();
        draw_text(
            &**font,
            framebuffer,
            &self.camera,
            &panel.text,
            vec2(t * 15.0, 2.0),
            TextAlign::CENTER,
            font_size,
            panel.color,
        );
    }
}
