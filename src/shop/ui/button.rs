use super::*;

pub struct Button<'a> {
    sense: &'a mut Sense,
    clicked: bool,
    inner: Box<dyn Widget + 'a>,
}

impl<'a> Button<'a> {
    pub fn new(cx: &'a Controller, text: &str) -> Self {
        let sense: &'a mut Sense = cx.get_state();
        let text = Text::new(
            text.to_owned(),
            cx.theme().font.clone(),
            cx.theme().text_size,
            if sense.is_hovered() {
                cx.theme().hover_color
            } else {
                cx.theme().usable_color
            },
        )
        .shrink(if sense.is_captured() {
            cx.theme().press_ratio as f64
        } else {
            0.0
        })
        .uniform_padding(5.0); // TODO: dehardcode
        let background_color = Color::BLUE;
        let background = ColorBox::new(background_color).constraints_override(Constraints {
            min_size: vec2(0.0, 0.0),
            flex: vec2(0.0, 0.0),
        });
        let ui = ui::stack![background, text];
        Self {
            clicked: sense.take_clicked(),
            sense,
            inner: Box::new(ui),
        }
    }
    pub fn was_clicked(&self) -> bool {
        self.clicked
    }
}

impl Widget for Button<'_> {
    fn sense(&mut self) -> Option<&mut Sense> {
        Some(self.sense)
    }
    fn calc_constraints(&mut self, cx: &ConstraintsContext) -> Constraints {
        cx.get_constraints(&self.inner)
    }
    fn walk_children_mut(&mut self, mut f: Box<dyn FnMut(&mut dyn Widget) + '_>) {
        f(&mut self.inner);
    }
    fn layout_children(&mut self, cx: &mut LayoutContext) {
        cx.set_position(&self.inner, cx.position);
    }
}
