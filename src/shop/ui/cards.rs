use super::*;

pub struct CardsRow<'a> {
    children: Vec<Box<dyn Widget + 'a>>,
    space_in: f64,
    space_out: f64,
}

impl<'a> CardsRow<'a> {
    pub fn new(cards: Vec<Box<dyn Widget + 'a>>, space_in: f64, space_out: f64) -> Self {
        Self {
            children: cards,
            space_in,
            space_out,
        }
    }
}

impl<'a> Widget for CardsRow<'a> {
    fn calc_constraints(&mut self, children: &ConstraintsContext) -> Constraints {
        Constraints {
            min_size: Vec2 {
                x: self
                    .children
                    .iter()
                    .map(|child| children.get_constraints(child.deref()).min_size.x)
                    .sum::<f64>()
                    + self.space_out * 2.0
                    + self.space_in * (self.children.len().max(1) - 1) as f64,
                y: self
                    .children
                    .iter()
                    .map(|child| children.get_constraints(child.deref()).min_size.y)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0)
                    + self.space_out * 2.0,
            },
            flex: Vec2 {
                x: self
                    .children
                    .iter()
                    .map(|child| children.get_constraints(child.deref()).flex.x)
                    .sum(),
                y: self
                    .children
                    .iter()
                    .map(|child| children.get_constraints(child.deref()).flex.y)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0),
            },
        }
    }
    fn layout_children(&mut self, cx: &mut LayoutContext) {
        let total_flex = self
            .children
            .iter()
            .map(|child| cx.get_constraints(child.deref()).flex.x)
            .sum::<f64>();
        let size_per_flex = if total_flex == 0.0 {
            0.0
        } else {
            (cx.position.width()
                - self
                    .children
                    .iter()
                    .map(|child| cx.get_constraints(child.deref()).min_size.x)
                    .sum::<f64>())
                / total_flex
        };
        let mut pos = cx.position.x_min + self.space_out;
        let child_height = cx.position.height() - self.space_out * 2.0;
        for child in &self.children {
            let child = child.deref();
            let width = cx.get_constraints(child).min_size.x
                + cx.get_constraints(child).flex.x * size_per_flex;
            cx.set_position(
                child,
                AABB::point(vec2(pos, cx.position.y_min))
                    .extend_positive(vec2(width, child_height)),
            );
            pos += width + self.space_in;
        }
    }
    fn walk_children_mut<'b>(&mut self, mut f: Box<dyn FnMut(&mut dyn Widget) + 'b>) {
        for child in &mut self.children {
            f(child.deref_mut());
        }
    }
    fn draw(&mut self, cx: &mut geng::ui::DrawContext) {
        let pixel_camera = &geng::PixelPerfectCamera;
        draw_2d::Quad::new(
            cx.position.map(|x| x as f32),
            Color::rgba(0.3, 0.3, 0.3, 1.0),
        )
        .draw_2d(cx.geng, cx.framebuffer, pixel_camera);
    }
}
