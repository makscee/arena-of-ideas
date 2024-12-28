use egui::{emath, remap};
use std::{
    cell::RefCell,
    f32::consts::TAU,
    ops::{Deref, DerefMut},
};

use super::*;

pub struct DataFrameMut<'a, T> {
    data: &'a mut T,
    prefix: Option<&'a str>,
    header: Option<Box<dyn FnOnce(&mut T, &mut Ui) -> bool>>,
    body: Option<Box<dyn FnOnce(&mut T, &mut Ui) -> bool>>,
    name: Option<Box<dyn FnOnce(&mut T, &mut Ui) -> bool>>,
    context_actions: HashMap<&'static str, Box<dyn FnOnce(&mut T) -> bool>>,
}

pub struct DataFrame<'a, T> {
    data: &'a T,
    prefix: Option<&'a str>,
    header: Option<Box<dyn FnOnce(&T, &mut Ui) -> bool + 'a>>,
    body: Option<Box<dyn FnOnce(&T, &mut Ui) -> bool + 'a>>,
    context_actions: HashMap<&'static str, Box<dyn FnOnce(&T) -> bool + 'a>>,
}

const FRAME: Frame = Frame {
    inner_margin: Margin::ZERO,
    outer_margin: Margin::ZERO,
    rounding: ROUNDING,
    shadow: Shadow::NONE,
    fill: EMPTINESS,
    stroke: STROKE_DARK,
};

impl<'a, T> DataFrame<'a, T>
where
    T: ToCstr + Clone + std::fmt::Debug,
{
    pub fn new(data: &'a T) -> Self {
        Self {
            data,
            header: None,
            body: None,
            prefix: None,
            context_actions: default(),
        }
    }
    pub fn prefix(mut self, prefix: Option<&'a str>) -> Self {
        self.prefix = prefix;
        self
    }
    pub fn header(mut self, f: impl FnOnce(&T, &mut Ui) -> bool + 'a) -> Self {
        self.header = Some(Box::new(f));
        self
    }
    pub fn body(mut self, f: impl FnOnce(&T, &mut Ui) -> bool + 'a) -> Self {
        self.body = Some(Box::new(f));
        self
    }
    pub fn copy(mut self, f: impl FnOnce(&T) + 'a) -> Self {
        self.context_actions.insert(
            "Copy",
            Box::new(move |d| {
                f(d);
                false
            }),
        );
        self
    }
    pub fn ui(self, ui: &mut Ui) -> bool {
        let data = RefCell::new(self.data.clone());
        let header = self
            .header
            .map(|f| |ui: &mut Ui| f(data.borrow().deref(), ui));
        let body = self
            .body
            .map(|f| |ui: &mut Ui| f(data.borrow().deref(), ui));
        let name = |ui: &mut Ui| {
            self.data.cstr_s(CstrStyle::Bold).label(ui);
            false
        };
        let context_actions = HashMap::from_iter(
            self.context_actions
                .into_iter()
                .map(|(k, v)| (k, || v(data.borrow().deref()))),
        );
        let r = compose_ui(self.prefix, header, body, name, context_actions, ui);
        r
    }
}

impl<'a, T> DataFrameMut<'a, T>
where
    T: ToCstr + Clone + std::fmt::Debug,
{
    pub fn new(data: &'a mut T) -> Self {
        Self {
            data,
            header: None,
            body: None,
            prefix: None,
            name: None,
            context_actions: default(),
        }
    }
    pub fn new_selector(data: &'a mut T) -> Self
    where
        T: AsRef<str> + IntoEnumIterator + PartialEq + Inject,
    {
        let mut r = Self::new(data);
        r.name = Some(Box::new(|d, ui| {
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
        let data = RefCell::new(self.data.clone());
        let header = self
            .header
            .map(|f| |ui: &mut Ui| f(data.borrow_mut().deref_mut(), ui));
        let body = self
            .body
            .map(|f| |ui: &mut Ui| f(data.borrow_mut().deref_mut(), ui));
        let name = |ui: &mut Ui| {
            if let Some(name) = self.name {
                name(data.borrow_mut().deref_mut(), ui)
            } else {
                self.data.cstr_s(CstrStyle::Bold);
                false
            }
        };
        let context_actions = HashMap::from_iter(
            self.context_actions
                .into_iter()
                .map(|(k, v)| (k, || v(data.borrow_mut().deref_mut()))),
        );
        let r = compose_ui(self.prefix, header, body, name, context_actions, ui);
        *self.data = data.into_inner();
        r
    }
}

fn compose_ui(
    prefix: Option<&str>,
    header: Option<impl FnOnce(&mut Ui) -> bool>,
    body: Option<impl FnOnce(&mut Ui) -> bool>,
    name: impl FnOnce(&mut Ui) -> bool,
    context_actions: HashMap<&'static str, impl FnOnce() -> bool>,
    ui: &mut Ui,
) -> bool {
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
        ne: if header.is_none() || collapsed {
            r
        } else {
            0.0
        },
        sw: if body.is_none() || collapsed { r } else { 0.0 },
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
                            if let Some(prefix) = prefix {
                                format!("[vd {prefix}]").label(ui);
                            }
                            changed |= name(ui);
                            if !context_actions.is_empty() {
                                ui.menu_button("+", |ui| {
                                    for (name, action) in context_actions {
                                        if ui.button(name).clicked() {
                                            action();
                                            changed = true;
                                            ui.close_menu();
                                        }
                                    }
                                    if ui.button("Close").clicked() {
                                        ui.close_menu();
                                    }
                                });
                            }
                            if header.is_some() || body.is_some() {
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
                        if let Some(f) = header {
                            changed |= f(ui);
                            ui.add_space(4.0);
                        }
                    }
                });
                if !collapsed {
                    if let Some(f) = body {
                        Frame::none()
                            .inner_margin(Margin {
                                left: 8.0,
                                right: 8.0,
                                top: 0.0,
                                bottom: 4.0,
                            })
                            .show(ui, |ui| changed |= f(ui));
                    }
                }
            });
        })
        .response;
    set_ctx_bool_id(ui.ctx(), hovered_id, resp.hovered());
    changed
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

pub trait DataFramed {
    fn has_header(&self) -> bool;
    fn has_body(&self) -> bool;
}

impl DataFramed for Expression {
    fn has_header(&self) -> bool {
        match self {
            Expression::Var(_)
            | Expression::V(_)
            | Expression::S(_)
            | Expression::F(_)
            | Expression::I(_)
            | Expression::B(_)
            | Expression::V2(_, _)
            | Expression::C(_) => true,
            Expression::One
            | Expression::Zero
            | Expression::GT
            | Expression::Owner
            | Expression::Target
            | Expression::UnitSize
            | Expression::Sin(..)
            | Expression::Cos(..)
            | Expression::Even(..)
            | Expression::Abs(..)
            | Expression::Floor(..)
            | Expression::Ceil(..)
            | Expression::Fract(..)
            | Expression::Sqr(..)
            | Expression::V2EE(..)
            | Expression::Macro(..)
            | Expression::Sum(..)
            | Expression::Sub(..)
            | Expression::Mul(..)
            | Expression::Div(..)
            | Expression::Max(..)
            | Expression::Min(..)
            | Expression::Mod(..)
            | Expression::And(..)
            | Expression::Or(..)
            | Expression::Equals(..)
            | Expression::GreaterThen(..)
            | Expression::LessThen(..)
            | Expression::If(..) => false,
        }
    }
    fn has_body(&self) -> bool {
        match self {
            Expression::One
            | Expression::Zero
            | Expression::GT
            | Expression::Owner
            | Expression::Target
            | Expression::UnitSize
            | Expression::Var(..)
            | Expression::V(..)
            | Expression::S(..)
            | Expression::F(..)
            | Expression::I(..)
            | Expression::B(..)
            | Expression::V2(..)
            | Expression::C(..) => false,
            Expression::Sin(..)
            | Expression::Cos(..)
            | Expression::Even(..)
            | Expression::Abs(..)
            | Expression::Floor(..)
            | Expression::Ceil(..)
            | Expression::Fract(..)
            | Expression::Sqr(..)
            | Expression::V2EE(..)
            | Expression::Macro(..)
            | Expression::Sum(..)
            | Expression::Sub(..)
            | Expression::Mul(..)
            | Expression::Div(..)
            | Expression::Max(..)
            | Expression::Min(..)
            | Expression::Mod(..)
            | Expression::And(..)
            | Expression::Or(..)
            | Expression::Equals(..)
            | Expression::GreaterThen(..)
            | Expression::LessThen(..)
            | Expression::If(..) => true,
        }
    }
}
