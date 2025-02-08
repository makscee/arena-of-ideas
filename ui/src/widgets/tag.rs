use std::sync::Arc;

use super::*;

#[derive(Clone)]
pub struct TagWidget {
    text: String,
    color: Color32,
    number: Option<Cstr>,
    text_galley: Option<Arc<egui::Galley>>,
    number_galley: Option<Arc<egui::Galley>>,
}

const INNER_MARGIN: Margin = Margin::symmetric(4.0, 1.0);
const OUTER_MARGIN: Margin = Margin::symmetric(4.0, 4.0);
const NUMBER_MARGIN: Margin = Margin {
    left: 8.0,
    right: 2.0,
    top: 0.0,
    bottom: 0.0,
};
impl TagWidget {
    pub fn new_text(text: impl ToString, color: Color32) -> Self {
        Self {
            text: text.to_string(),
            color,
            text_galley: None,
            number_galley: None,
            number: None,
        }
    }
    pub fn new_number(text: impl ToString, color: Color32, number: Cstr) -> Self {
        Self {
            text: text.to_string(),
            color,
            number: Some(number),
            text_galley: None,
            number_galley: None,
        }
    }
    fn text_size(&mut self, ui: &mut Ui) -> egui::Vec2 {
        let galley = self
            .text
            .cstr_cs(EMPTINESS, CstrStyle::Bold)
            .galley(1.0, ui);
        let size = galley.size();
        self.text_galley = Some(galley);
        size
    }
    fn number_size(&mut self, ui: &mut Ui) -> egui::Vec2 {
        let Some(number) = &self.number else {
            return default();
        };
        let galley = number
            .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold)
            .galley(1.0, ui);
        let mut size = galley.size();
        size += NUMBER_MARGIN.sum();
        size.y = 0.0;
        self.number_galley = Some(galley);
        size
    }
    fn margin_size() -> egui::Vec2 {
        INNER_MARGIN.sum() + OUTER_MARGIN.sum()
    }
    pub fn size(&mut self, ui: &mut Ui) -> egui::Vec2 {
        self.text_size(ui) + self.number_size(ui) + Self::margin_size()
    }
    pub fn ui(mut self, ui: &mut Ui) {
        let frame = Frame {
            rounding: Rounding::same(13.0),
            shadow: Shadow::NONE,
            fill: self.color,
            stroke: Stroke::new(1.0, self.color),
            ..default()
        };
        let text_size = self.text_size(ui);
        let number_size = self.number_size(ui);
        let margin_size = Self::margin_size();
        let (rect, _) =
            ui.allocate_exact_size(text_size + number_size + margin_size, Sense::hover());
        if self.number.is_some() {
            ui.painter().add(
                frame
                    .fill(EMPTINESS)
                    .paint(rect.shrink2(OUTER_MARGIN.sum() * 0.5)),
            );
        }
        ui.painter().add(
            frame.paint(
                rect.with_max_x(rect.max.x - number_size.x)
                    .shrink2(OUTER_MARGIN.sum() * 0.5),
            ),
        );
        ui.painter().galley(
            rect.shrink2(margin_size * 0.5).left_top(),
            self.text_galley.unwrap(),
            VISIBLE_BRIGHT,
        );
        if let Some(number) = self.number_galley {
            ui.painter().galley(
                rect.shrink2(margin_size * 0.5).right_top()
                    - egui::vec2(number_size.x - NUMBER_MARGIN.left, 0.0),
                number,
                VISIBLE_BRIGHT,
            );
        }
    }
}

#[derive(Clone)]
pub struct TagsWidget {
    tags: Vec<TagWidget>,
}

impl TagsWidget {
    pub fn new() -> Self {
        Self { tags: default() }
    }
    pub fn add_text(&mut self, text: impl ToString, color: Color32) {
        self.tags.push(TagWidget::new_text(text, color));
    }
    pub fn add_number(&mut self, text: impl ToString, color: Color32, number: i32) {
        self.tags.push(TagWidget::new_number(
            text,
            color,
            number.to_string().cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold),
        ));
    }
    pub fn add_number_cstr(&mut self, text: impl ToString, color: Color32, number: Cstr) {
        self.tags.push(TagWidget::new_number(text, color, number));
    }
    pub fn ui(mut self, ui: &mut Ui) {
        let mut size = egui::Vec2::ZERO;
        for tag in &mut self.tags {
            let tag_size = tag.size(ui);
            size.y = size.y.max(tag_size.y);
            size.x += tag_size.x;
        }
        let right_bottom = ui.cursor().center_top() + egui::vec2(size.x * 0.5, size.y);
        let rect = Rect::from_min_max(right_bottom - size, right_bottom);
        ui.allocate_ui_at_rect(rect, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                for tag in self.tags {
                    tag.ui(ui);
                }
            })
        });
    }
}
