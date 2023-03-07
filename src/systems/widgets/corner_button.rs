use std::rc::Rc;

use super::*;

const SIZE: f32 = 200.0;
pub struct CornerButtonWidget<'a> {
    resources: &'a Resources,
    pub icon: Image,
    pub position: &'a mut Aabb2<f32>,         // Hidden
    pub animation_time: &'a mut f32,          // state
    pub sense: &'a mut geng::ui::Sense,       // Helper hidden state for interaction
    pub change: RefCell<&'a mut Option<f32>>, // Result of interaction is optionally a change that was made
    pub corner: vec2<f32>,
}

impl<'a> CornerButtonWidget<'a> {
    pub fn new(
        cx: &'a geng::ui::Controller,
        resources: &'a Resources,
        icon: Image,
        corner: vec2<f32>,
    ) -> Self {
        Self {
            position: cx.get_state_with(|| Aabb2::point(vec2::ZERO)), // Specify default value for hidden state
            animation_time: cx.get_state(),
            sense: cx.get_state(),
            change: RefCell::new(cx.get_state()),
            resources,
            icon,
            corner,
        }
    }

    // We had a RefCell so that this method doesn't need a mut reference to self
    pub fn get_change(&self) -> Option<f32> {
        self.change.borrow_mut().take()
    }
}

impl geng::ui::Widget for CornerButtonWidget<'_> {
    fn calc_constraints(
        &mut self,
        _children: &geng::ui::ConstraintsContext,
    ) -> geng::ui::Constraints {
        geng::ui::Constraints {
            min_size: vec2(1.0, 1.0),
            flex: vec2(0.0, 0.0),
        }
    }
    // If using Sense helper this method must be added
    fn sense(&mut self) -> Option<&mut geng::ui::Sense> {
        Some(self.sense)
    }
    fn update(&mut self, delta_time: f64) {
        #![allow(unused_variables)]
    }
    fn draw(&mut self, cx: &mut geng::ui::DrawContext) {
        let size = cx.position.size().map(|x| x as f32);
        *self.position = Aabb2::point(size * self.corner - vec2(SIZE, SIZE) * self.corner)
            .extend_uniform(SIZE * 0.5);
        #[derive(ugli::Vertex)]
        struct Vertex {
            a_pos: vec2<f32>,
        }
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
                        a_pos: vec2(0.0, 0.0),
                    },
                    Vertex {
                        a_pos: vec2(1.0, 0.0),
                    },
                    Vertex {
                        a_pos: vec2(1.0, 1.0),
                    },
                    Vertex {
                        a_pos: vec2(0.0, 1.0),
                    },
                ],
            ),
            (
                ugli::uniforms! {
                    u_texture: self.icon.get(self.resources).deref(),
                    u_pos: self.position.center(),
                    u_size: self.position.size(),
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
    fn handle_event(&mut self, event: &geng::Event) {
        // Use helper to determine if we should process interactions
        if self.sense.is_hovered() {
            if let geng::Event::MouseDown { position, .. } = &event {
                debug!("Click");
                let new_value = ((position.y as f32 - self.position.min.y)
                    / self.position.height().max(0.1))
                .clamp(0.0, 1.0);
                **self.change.borrow_mut() = Some(new_value);
            }
        }
    }
}
