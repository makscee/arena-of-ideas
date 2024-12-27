use std::f32::consts::TAU;

use egui::{emath, remap};

use super::*;

pub struct CollapsingFrame<'a, T> {
    data: &'a mut T,
    prefix: Option<&'a str>,
    header: Option<Box<dyn FnOnce(&mut T, &mut Ui) -> bool>>,
    body: Option<Box<dyn FnOnce(&mut T, &mut Ui) -> bool>>,
    ui_name: Option<Box<dyn FnOnce(&mut T, &mut Ui) -> bool>>,
    context_actions: HashMap<&'static str, Box<dyn FnOnce(&mut T) -> bool>>,
}

impl<'a, T> CollapsingFrame<'a, T>
where
    T: ToCstr + Clone + std::fmt::Debug,
{
    pub fn new(data: &'a mut T) -> Self {
        Self {
            data,
            header: None,
            body: None,
            prefix: None,
            ui_name: None,
            context_actions: default(),
        }
    }
    pub fn new_selector(data: &'a mut T) -> Self
    where
        T: AsRef<str> + IntoEnumIterator + PartialEq + Inject,
    {
        let mut r = Self::new(data);
        r.ui_name = Some(Box::new(|d, ui| {
            let mut new_value = d.clone();
            if Selector::new("").ui_enum(&mut new_value, ui) {
                new_value.move_inner(d);
                *d = new_value;
                true
            } else {
                false
            }
        }));
        r
    }
    pub fn prefix(mut self, prefix: Option<&'a str>) -> Self {
        self.prefix = prefix;
        self
    }
    pub fn header(mut self, f: impl FnOnce(&mut T, &mut Ui) -> bool + 'static) -> Self {
        self.header = Some(Box::new(f));
        self
    }
    pub fn body(mut self, f: impl FnOnce(&mut T, &mut Ui) -> bool + 'static) -> Self {
        self.body = Some(Box::new(f));
        self
    }
    pub fn wrapper(mut self, f: impl FnOnce(&mut T) + 'static) -> Self {
        self.context_actions.insert(
            "Wrap",
            Box::new(move |d| {
                f(d);
                true
            }),
        );
        self
    }
    pub fn copy(mut self, f: impl FnOnce(&mut T) + 'static) -> Self {
        self.context_actions.insert(
            "Copy",
            Box::new(move |d| {
                f(d);
                false
            }),
        );
        self
    }
    pub fn paste(mut self, f: impl FnOnce(&mut T) + 'static) -> Self {
        self.context_actions.insert(
            "Paste",
            Box::new(move |d| {
                f(d);
                true
            }),
        );
        self
    }
    pub fn ui(self, ui: &mut Ui) -> bool {
        const FRAME: Frame = Frame {
            inner_margin: Margin::ZERO,
            outer_margin: Margin::ZERO,
            rounding: ROUNDING,
            shadow: Shadow::NONE,
            fill: EMPTINESS,
            stroke: STROKE_DARK,
        };

        let mut changed = false;
        let id = ui.next_auto_id();
        let collapsed_id = id.with("collapsed");
        let hovered_id = id.with("hovered");
        let collapsed = get_ctx_bool_id(ui.ctx(), collapsed_id);
        let openness = ui.ctx().animate_bool(id, collapsed);
        let hovered = get_ctx_bool_id(ui.ctx(), hovered_id);

        let r = 13.0;
        let header_rounding = Rounding {
            nw: r,
            ne: if self.header.is_none() || collapsed {
                r
            } else {
                0.0
            },
            sw: if self.body.is_none() || collapsed {
                r
            } else {
                0.0
            },
            se: r,
        };
        let resp = FRAME
            .stroke(if hovered { STROKE_LIGHT } else { STROKE_DARK })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        FRAME
                            .fill(BG_DARK)
                            .inner_margin(Margin::symmetric(8.0, 4.0))
                            .rounding(header_rounding)
                            .show(ui, |ui| {
                                if let Some(prefix) = self.prefix {
                                    format!("[vd {prefix}]").label(ui);
                                }
                                if let Some(name) = self.ui_name {
                                    changed |= name(self.data, ui);
                                } else {
                                    let name = self.data.cstr_s(CstrStyle::Bold).as_label(ui);
                                    name.ui(ui);
                                }
                                if !self.context_actions.is_empty() {
                                    ui.menu_button("+", |ui| {
                                        for (name, action) in self.context_actions {
                                            if ui.button(name).clicked() {
                                                action(self.data);
                                                ui.close_menu();
                                            }
                                        }
                                        if ui.button("Close").clicked() {
                                            ui.close_menu();
                                        }
                                    });
                                }
                                if self.header.is_some() || self.body.is_some() {
                                    let x = ui.available_height() - 4.0;
                                    let (_, resp) =
                                        ui.allocate_at_least(egui::Vec2::splat(x), Sense::click());
                                    show_triangle(openness, &resp, ui);
                                    if resp.clicked() {
                                        set_ctx_bool_id(ui.ctx(), collapsed_id, !collapsed);
                                    }
                                }
                            });
                        if !collapsed {
                            if let Some(content) = self.header {
                                changed |= content(self.data, ui);
                                ui.add_space(4.0);
                            }
                        }
                    });
                    if !collapsed {
                        if let Some(content) = self.body {
                            Frame::none()
                                .inner_margin(Margin {
                                    left: 8.0,
                                    right: 8.0,
                                    top: 0.0,
                                    bottom: 4.0,
                                })
                                .show(ui, |ui| changed |= content(self.data, ui));
                        }
                    }
                });
            })
            .response;
        set_ctx_bool_id(ui.ctx(), hovered_id, resp.hovered());
        changed
    }
}

fn show_triangle(openness: f32, resp: &Response, ui: &mut Ui) {
    let rect = resp.rect;
    let rect = Rect::from_center_size(rect.center(), egui::vec2(rect.width(), rect.height()) * 0.5);
    let mut points = vec![rect.left_top(), rect.right_top(), rect.center_bottom()];
    let rotation = emath::Rot2::from_angle(remap(1.0 - openness, 0.0..=1.0, -TAU / 4.0..=0.0));
    for p in &mut points {
        *p = rect.center() + rotation * (*p - rect.center());
    }
    ui.painter().add(egui::Shape::convex_polygon(
        points,
        TRANSPARENT,
        if resp.hovered() {
            STROKE_YELLOW
        } else {
            STROKE_DARK
        },
    ));
}
