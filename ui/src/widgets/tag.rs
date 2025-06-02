use std::sync::Arc;

use super::*;

#[derive(Clone)]
pub struct TagWidget {
    name: String,
    color: Color32,
    value: Option<Cstr>,
    text_galley: Option<Arc<egui::Galley>>,
    number_galley: Option<Arc<egui::Galley>>,
}

const INNER_MARGIN: Margin = Margin::symmetric(4, 1);
const OUTER_MARGIN: Margin = Margin::symmetric(4, 1);
const NUMBER_MARGIN: Margin = Margin {
    left: 8,
    right: 2,
    top: 0,
    bottom: 0,
};
impl TagWidget {
    #[must_use]
    pub fn new_name(name: impl ToString, color: Color32) -> Self {
        Self {
            name: name.to_string(),
            color,
            text_galley: None,
            number_galley: None,
            value: None,
        }
    }
    #[must_use]
    pub fn new_var_value(var: VarName, value: VarValue) -> Self {
        Self {
            name: var.to_string(),
            color: var.color(),
            value: Some(value.cstr()),
            text_galley: None,
            number_galley: None,
        }
    }
    #[must_use]
    pub fn new_name_value(name: impl ToString, color: Color32, value: Cstr) -> Self {
        Self {
            name: name.to_string(),
            color,
            value: Some(value),
            text_galley: None,
            number_galley: None,
        }
    }
    fn name_size(&mut self, ui: &mut Ui) -> egui::Vec2 {
        let galley = self
            .name
            .cstr_cs(tokens_global().app_background(), CstrStyle::Bold)
            .galley(1.0, ui);
        let size = galley.size();
        self.text_galley = Some(galley);
        size
    }
    fn value_size(&mut self, ui: &mut Ui) -> egui::Vec2 {
        let Some(number) = &self.value else {
            return default();
        };
        let galley = number
            .cstr_cs(tokens_global().high_contrast_text(), CstrStyle::Bold)
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
    pub fn ui(mut self, ui: &mut Ui) -> Response {
        let mut frame = Frame {
            corner_radius: ROUNDING,
            shadow: Shadow::NONE,
            fill: self.color,
            stroke: Stroke::new(1.0, self.color),
            ..default()
        };
        let text_size = self.name_size(ui);
        let number_size = self.value_size(ui);
        let margin_size = Self::margin_size();
        let (rect, response) =
            ui.allocate_exact_size(text_size + number_size + margin_size, Sense::click());
        let hovered = response.hovered() && !response.is_pointer_button_down_on();
        if hovered {
            frame.stroke = ui.visuals().widgets.hovered.bg_stroke;
        }
        if self.value.is_some() {
            ui.painter().add(
                frame
                    .fill(if hovered {
                        ui.visuals().widgets.hovered.bg_fill
                    } else {
                        tokens_global().app_background()
                    })
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
            tokens_global().high_contrast_text(),
        );
        if let Some(number) = self.number_galley {
            ui.painter().galley(
                rect.shrink2(margin_size * 0.5).right_top()
                    - egui::vec2(number_size.x - NUMBER_MARGIN.left as f32, 0.0),
                number,
                tokens_global().high_contrast_text(),
            );
        }
        response
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
    pub fn add_name(&mut self, text: impl ToString, color: Color32) {
        self.tags.push(TagWidget::new_name(text, color));
    }
    pub fn add_number(&mut self, text: impl ToString, color: Color32, number: i32) {
        self.tags.push(TagWidget::new_name_value(
            text,
            color,
            number
                .to_string()
                .cstr_cs(tokens_global().high_contrast_text(), CstrStyle::Bold),
        ));
    }
    pub fn add_name_value(&mut self, text: impl ToString, color: Color32, number: Cstr) {
        self.tags
            .push(TagWidget::new_name_value(text, color, number));
    }
    pub fn ui(self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for tag in self.tags {
                tag.ui(ui);
            }
        });
    }
}
