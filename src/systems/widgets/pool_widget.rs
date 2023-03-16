use super::*;
use geng::ui::*;

const MAX_HEIGHT: f64 = 300.0;
pub struct PoolWidget<'a> {
    resources: &'a Resources,
    pub position: &'a mut Aabb2<f32>,
    pub height: f32,
}

impl<'a> PoolWidget<'a> {
    pub fn new(cx: &'a geng::ui::Controller, resources: &'a Resources, height: f32) -> Self {
        Self {
            position: cx.get_state_with(|| Aabb2::point(vec2::ZERO)),
            resources,
            height,
        }
    }
    pub fn place(self) -> Box<dyn Widget + 'a> {
        self.flex_align(vec2(Some(1.0), None), vec2(0.0, 1.0))
            .fixed_size(vec2(0.0, 400.0))
            .boxed()
    }
}

impl geng::ui::Widget for PoolWidget<'_> {
    fn calc_constraints(&mut self, _children: &ConstraintsContext) -> Constraints {
        geng::ui::Constraints {
            min_size: vec2(0.0, self.height as f64 * MAX_HEIGHT),
            flex: vec2(1.0, 0.0),
        }
    }
    fn draw(&mut self, cx: &mut geng::ui::DrawContext) {
        *self.position = cx.position.map(|x| x as f32);
        if self.height < 0.001 {
            return;
        }
        #[derive(ugli::Vertex)]
        struct Vertex {
            a_pos: vec2<f32>,
        }

        let framebuffer_size = cx.framebuffer.size().map(|x| x as f32);
        let position = self.resources.camera.camera.screen_to_world(
            framebuffer_size,
            (cx.position.bottom_left() + cx.position.size() * vec2(0.2, 0.8)).map(|x| x as f32),
        );
        self.resources
            .shop
            .units
            .iter()
            .enumerate()
            .for_each(|(ind, (size, shader))| {
                let position = position + vec2(ind as f32 * 2.0, 0.0);
                ShaderSystem::draw_shader_single(
                    shader,
                    cx.framebuffer,
                    self.resources,
                    ugli::uniforms! {
                        u_global_time: self.resources.global_time,
                        u_game_time: self.resources.cassette.head,
                        u_position: position,
                        u_radius: 0.6 * self.height,
                    },
                );
                let text_texture = self.resources.fonts.get_texture(1, size).unwrap();
                ShaderSystem::draw_shader_single(
                    &self.resources.options.shaders.text,
                    cx.framebuffer,
                    self.resources,
                    ugli::uniforms! {
                        u_global_time: self.resources.global_time,
                        u_game_time: self.resources.cassette.head,
                        u_position: position,
                        u_offset: vec2(0.0,-1.0),
                        u_text: text_texture,
                        u_scale: 0.4 * self.height,
                        u_text_size: text_texture.size().map(|x| x as f32),
                        u_outline_color: self.resources.options.colors.corner_button_color,
                    },
                );
            });
    }
}
