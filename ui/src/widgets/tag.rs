use std::sync::Arc;

use super::*;

#[derive(Clone)]
pub struct TagWidget {
    text: String,
    color: Color32,
    galley: Option<Arc<egui::Galley>>,
}

const INNER_MARGIN: Margin = Margin::symmetric(4.0, 1.0);
const OUTER_MARGIN: Margin = Margin::same(0.0);
impl TagWidget {
    pub fn new(text: impl ToString, color: Color32) -> Self {
        Self {
            text: text.to_string(),
            color,
            galley: None,
        }
    }
    fn text_size(&mut self, ui: &mut Ui) -> egui::Vec2 {
        let galley = self.text.cstr_cs(BG_DARK, CstrStyle::Bold).galley(1.0, ui);
        let size = galley.size();
        self.galley = Some(galley);
        size
    }
    fn margin_size() -> egui::Vec2 {
        INNER_MARGIN.sum() + OUTER_MARGIN.sum()
    }
    pub fn size(&mut self, ui: &mut Ui) -> egui::Vec2 {
        self.text_size(ui) + Self::margin_size()
    }
    pub fn ui(mut self, ui: &mut Ui) {
        if self.galley.is_none() {
            self.text_size(ui);
        }
        let frame = Frame {
            inner_margin: INNER_MARGIN,
            outer_margin: OUTER_MARGIN,
            rounding: Rounding::same(13.0),
            shadow: Shadow::NONE,
            fill: self.color,
            stroke: Stroke::default(),
        };
        let galley = self.galley.unwrap();
        let (rect, _) = ui.allocate_exact_size(galley.size() + Self::margin_size(), Sense::hover());
        ui.painter()
            .add(frame.paint(rect.shrink2(frame.outer_margin.sum() * 0.5)));
        ui.painter().galley(
            rect.shrink2(Self::margin_size() * 0.5).left_top(),
            galley,
            VISIBLE_BRIGHT,
        );
    }
}
