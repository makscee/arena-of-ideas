use super::*;
use geng::ui::*;

const SIZE: f64 = 100.0;
pub struct CornerButtonWidget<'a> {
    resources: &'a Resources,
    pub icon: Image,
    pub position: &'a mut Aabb2<f32>,
    pub sense: &'a mut geng::ui::Sense,
    clicked: bool,
    corner: vec2<f32>,
}

impl<'a> CornerButtonWidget<'a> {
    pub fn new(cx: &'a geng::ui::Controller, resources: &'a Resources, icon: Image) -> Self {
        let sense: &'a mut Sense = cx.get_state();
        Self {
            position: cx.get_state_with(|| Aabb2::point(vec2::ZERO)), // Specify default value for hidden state
            clicked: sense.take_clicked(),
            sense,
            resources,
            icon,
            corner: vec2::ZERO,
        }
    }

    pub fn place(mut self, corner: vec2<f64>) -> Box<dyn geng::ui::Widget + 'a> {
        self.corner = corner.map(|x| x as f32);
        self.flex_align(vec2(None, None), corner).boxed()
    }

    pub fn was_clicked(&self) -> bool {
        self.clicked
    }
}

impl geng::ui::Widget for CornerButtonWidget<'_> {
    fn calc_constraints(
        &mut self,
        _children: &geng::ui::ConstraintsContext,
    ) -> geng::ui::Constraints {
        geng::ui::Constraints {
            min_size: vec2(SIZE, SIZE),
            flex: vec2(0.0, 0.0),
        }
    }
    fn sense(&mut self) -> Option<&mut geng::ui::Sense> {
        Some(self.sense)
    }
    fn update(&mut self, delta_time: f64) {
        self.sense.update(delta_time);
    }
    fn draw(&mut self, cx: &mut geng::ui::DrawContext) {
        *self.position = cx.position.map(|x| x as f32);
        #[derive(ugli::Vertex)]
        struct Vertex {
            a_pos: vec2<f32>,
        }
        let button_color = self.resources.options.colors.button;
        let icon_color = self.resources.options.colors.primary;
        let scale = 1.0
            + match self.sense.hovered_time {
                Some(value) => (0.4 - value * value * 0.5).clamp_min(0.2) as f32,
                None => 0.0,
            };
        ugli::draw(
            cx.framebuffer,
            &self.resources.shader_programs.get_program(
                &static_path().join(self.resources.options.shaders.corner_button.path.clone()),
            ),
            ugli::DrawMode::TriangleFan,
            &ugli::VertexBuffer::new_dynamic(
                cx.geng.ugli(),
                vec![
                    Vertex {
                        a_pos: vec2(-1.0, -1.0),
                    },
                    Vertex {
                        a_pos: vec2(1.0, -1.0),
                    },
                    Vertex {
                        a_pos: vec2(1.0, 1.0),
                    },
                    Vertex {
                        a_pos: vec2(-1.0, 1.0),
                    },
                ],
            ),
            (
                ugli::uniforms! {
                    u_texture: self.resources.image_textures.get_texture(&self.icon),
                    u_pos: cx.position.center().map(|x| x as f32),
                    u_box: cx.position.size().map(|x| x as f32) * 0.5,
                    u_color: button_color,
                    u_icon_color: icon_color,
                    u_size: scale,
                    u_corner: self.corner,
                },
                geng::camera2d_uniforms(
                    &geng::PixelPerfectCamera,
                    cx.framebuffer.size().map(|x| x as f32),
                ),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                ..default()
            },
        );
    }
    fn handle_event(&mut self, _: &geng::Event) {}
}
