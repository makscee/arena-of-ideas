use egui::Shadow;

use super::*;

pub struct NodeFrame {
    type_name: String,
    color: Color32,
    title: Option<String>,
    depth: usize,
}

impl NodeFrame {
    pub fn new(type_name: String, color: Color32) -> Self {
        Self {
            type_name,
            color,
            title: None,
            depth: 0,
        }
    }
    pub fn title(mut self, title: Option<String>) -> Self {
        self.title = title;
        self
    }
    pub fn depth(mut self, value: usize) -> Self {
        self.depth = value;
        self
    }
    pub fn ui(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        const FRAME: Frame = Frame {
            inner_margin: Margin::same(0.0),
            outer_margin: Margin::same(4.0),
            rounding: Rounding::same(13.0),
            shadow: Shadow::NONE,
            fill: BG_DARK,
            stroke: Stroke::NONE,
        };
        let fill = if self.depth % 2 == 0 {
            BG_LIGHT
        } else {
            BG_DARK
        };
        ui.style_mut().spacing.item_spacing.y = 0.0;
        FRAME.fill(fill).show(ui, |ui| {
            let mut trounding = Rounding {
                nw: 13.0,
                ne: 0.0,
                sw: 0.0,
                se: 13.0,
            };
            const MARGIN: Margin = Margin {
                left: 4.0,
                right: 4.0,
                top: 0.0,
                bottom: 0.0,
            };
            if let Some(title) = self.title {
                Frame::none()
                    .stroke(Stroke::new(1.0, self.color))
                    .rounding(Rounding {
                        nw: 13.0,
                        ne: 13.0,
                        sw: 0.0,
                        se: 0.0,
                    })
                    .inner_margin(MARGIN)
                    .show(ui, |ui| {
                        ui.vertical_centered_justified(|ui| {
                            title.cstr_cs(self.color, CstrStyle::Bold).label(ui);
                        });
                    });
                trounding.nw = 0.0;
            }
            Frame::none()
                .fill(self.color)
                .inner_margin(MARGIN)
                .rounding(trounding)
                .show(ui, |ui| {
                    self.type_name.cstr_cs(fill, CstrStyle::Small).label(ui);
                });
            ui.set_min_width(ui.available_width());
            Frame::none()
                .inner_margin(Margin::same(8.0))
                .show(ui, |ui| {
                    content(ui);
                });
        });
    }
}
