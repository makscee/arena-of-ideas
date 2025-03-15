use bevy_egui::egui::menu::BarState;
use serde::de::DeserializeOwned;

use super::*;

pub struct DataFrameMut<'a, T> {
    data: &'a mut T,
    prefix: Option<&'a str>,
    header: Option<Box<dyn FnOnce(&mut T, &mut Ui) -> bool>>,
    body: Option<Box<dyn FnOnce(&mut T, &mut Ui) -> bool>>,
    name: Option<Box<dyn FnOnce(&mut T, &mut Ui) -> bool>>,
    context_actions: HashMap<&'static str, Box<dyn FnOnce(&mut T) -> bool>>,
    default_open: bool,
}

pub struct DataFrame<'a, T> {
    data: &'a T,
    prefix: Option<&'a str>,
    header: Option<Box<dyn FnOnce(&T, &mut Ui) + 'a>>,
    body: Option<Box<dyn FnOnce(&T, &mut Ui) + 'a>>,
    context_actions: HashMap<&'static str, Box<dyn FnOnce(&T) + 'a>>,
    default_open: bool,
}

const FRAME: Frame = Frame {
    inner_margin: Margin::ZERO,
    outer_margin: Margin::ZERO,
    corner_radius: ROUNDING,
    shadow: Shadow::NONE,
    fill: TRANSPARENT,
    stroke: STROKE_BG_DARK,
};

impl<'a, T> DataFrame<'a, T>
where
    T: ToCstr + Clone + std::fmt::Debug + StringData,
{
    pub fn new(data: &'a T) -> Self {
        let mut context_actions: HashMap<&str, Box<dyn FnOnce(&T)>> = default();
        context_actions.insert("Copy", Box::new(|d: &T| clipboard_set(d.get_data())));
        Self {
            data,
            header: None,
            body: None,
            prefix: None,
            context_actions,
            default_open: true,
        }
    }
    pub fn prefix(mut self, prefix: Option<&'a str>) -> Self {
        self.prefix = prefix;
        self
    }
    pub fn default_open(mut self, value: bool) -> Self {
        self.default_open = value;
        self
    }
    pub fn header(mut self, f: impl FnOnce(&T, &mut Ui) + 'a) -> Self {
        self.header = Some(Box::new(f));
        self
    }
    pub fn body(mut self, f: impl FnOnce(&T, &mut Ui) + 'a) -> Self {
        self.body = Some(Box::new(f));
        self
    }
    pub fn ui(self, ui: &mut Ui) -> bool {
        let data = RefCell::new(self.data.clone());
        let header = self.header.map(|f| {
            |ui: &mut Ui| {
                f(data.borrow().deref(), ui);
                false
            }
        });
        let body = self.body.map(|f| {
            |ui: &mut Ui| {
                f(data.borrow().deref(), ui);
                false
            }
        });
        let name = |ui: &mut Ui| {
            self.data.cstr().label(ui);
            false
        };
        let context_actions = HashMap::from_iter(self.context_actions.into_iter().map(|(k, v)| {
            (k, || {
                v(data.borrow().deref());
                false
            })
        }));
        let r = compose_ui(
            self.prefix,
            header,
            body,
            name,
            context_actions,
            self.default_open || ui.data_frame_is_force_open(),
            ui,
        );
        r
    }
}

impl<'a, T> DataFrameMut<'a, T>
where
    T: ToCstr + Clone + std::fmt::Debug + StringData,
{
    pub fn new(data: &'a mut T) -> Self {
        let mut context_actions: HashMap<&str, Box<dyn FnOnce(&mut T) -> bool>> = default();
        context_actions.insert(
            "Copy",
            Box::new(|d| {
                clipboard_set(d.get_data());
                false
            }),
        );
        context_actions.insert(
            "Paste",
            Box::new(move |d| {
                if let Some(c) = clipboard_get() {
                    d.inject_data(&c).log();
                    true
                } else {
                    false
                }
            }),
        );
        Self {
            data,
            header: None,
            body: None,
            prefix: None,
            name: None,
            context_actions,
            default_open: true,
        }
    }
    pub fn new_inject(data: &'a mut T) -> Self
    where
        T: Inject,
    {
        let mut r = Self::new(data);
        r.context_actions.insert(
            "Wrap",
            Box::new(move |d| {
                d.wrap();
                true
            }),
        );
        r
    }
    pub fn new_selector(data: &'a mut T) -> Self
    where
        T: AsRef<str> + IntoEnumIterator + PartialEq + Inject,
    {
        let mut r = Self::new_inject(data);
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
    pub fn default_open(mut self, value: bool) -> Self {
        self.default_open = value;
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
                self.data.cstr();
                false
            }
        };
        let context_actions = HashMap::from_iter(
            self.context_actions
                .into_iter()
                .map(|(k, v)| (k, || v(data.borrow_mut().deref_mut()))),
        );
        let r = compose_ui(
            self.prefix,
            header,
            body,
            name,
            context_actions,
            self.default_open || ui.data_frame_is_force_open(),
            ui,
        );
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
    default_open: bool,
    ui: &mut Ui,
) -> bool {
    let mut changed = false;
    let id = ui.next_auto_id();
    let collapsed_id = id.with("collapsed");
    let collapse_inner_id = id.with("collapse_inner");
    let collapse_override_id = Id::new("collapse_override");
    let hovered_id = id.with("hovered");
    let mut collapsed = get_ctx_bool_id_default(ui.ctx(), collapsed_id, !default_open);
    let collapsed_inner = get_ctx_bool_id_default(ui.ctx(), collapse_inner_id, false);
    if let Some(collapse_override) = get_ctx_bool_id(ui.ctx(), collapse_override_id) {
        collapsed = collapse_override;
        set_ctx_bool_id(ui.ctx(), collapsed_id, collapse_override);
        set_ctx_bool_id(ui.ctx(), collapse_inner_id, collapse_override);
    }
    let openness = ui.ctx().animate_bool(collapsed_id, collapsed);
    let openness_inner = ui.ctx().animate_bool(collapse_inner_id, collapsed_inner);
    let hovered = get_ctx_bool_id_default(ui.ctx(), hovered_id, false);

    const R: u8 = 13;
    let header_rounding = CornerRadius {
        nw: R,
        ne: if header.is_none() || collapsed { R } else { 0 },
        sw: if body.is_none() || collapsed { R } else { 0 },
        se: R,
    };
    let mut header_rect = Rect::ZERO;
    let mut triangle_rect = Rect::ZERO;
    let resp = FRAME
        .stroke(if hovered { STROKE_DARK } else { STROKE_BG_DARK })
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    header_rect = FRAME
                        .fill(BG_DARK)
                        .inner_margin(Margin::symmetric(8, 4))
                        .corner_radius(header_rounding)
                        .show(ui, |ui| {
                            if let Some(prefix) = prefix {
                                format!("[vd [s {prefix}]]").label(ui);
                            }
                            changed |= name(ui);
                            if !context_actions.is_empty() {
                                let (_, resp) = ui.allocate_at_least(
                                    egui::vec2(8.0, ui.available_height()),
                                    Sense::click(),
                                );
                                ui.painter().circle(
                                    resp.rect.center(),
                                    4.0,
                                    if resp.hovered() { YELLOW } else { BG_DARK },
                                    STROKE_DARK,
                                );
                                let bar_id = ui.id();
                                let mut bar_state = BarState::load(ui.ctx(), bar_id);
                                bar_state.bar_menu(&resp, |ui| {
                                    for (name, action) in context_actions {
                                        if ui.button(name).clicked() {
                                            action();
                                            changed = true;
                                            ui.close_menu();
                                        }
                                    }
                                    if "[vd Close]".cstr().button(ui).clicked() {
                                        ui.close_menu();
                                    }
                                });
                                bar_state.store(ui.ctx(), bar_id);
                            }
                            if header.is_some() || body.is_some() {
                                let x = ui.available_height() - 4.0;
                                let (_, resp) =
                                    ui.allocate_at_least(egui::Vec2::splat(x), Sense::click());
                                show_triangle(openness, resp.rect, resp.hovered(), ui);
                                triangle_rect = resp.rect;
                                if resp.clicked() {
                                    set_ctx_bool_id(ui.ctx(), collapsed_id, !collapsed);
                                }
                            }
                        })
                        .response
                        .rect;
                    if !collapsed {
                        if let Some(f) = header {
                            changed |= f(ui);
                            ui.add_space(4.0);
                        }
                    }
                });
                if !collapsed {
                    if let Some(f) = body {
                        let x_shift = header_rect.right() - triangle_rect.min.x + 4.0;
                        let triangle_rect = triangle_rect.translate(egui::vec2(x_shift, 0.0));
                        let resp = ui.allocate_rect(
                            Rect::from_min_size(
                                header_rect.right_top(),
                                egui::vec2(header_rect.height(), header_rect.height()),
                            ),
                            Sense::click(),
                        );
                        show_triangle(openness_inner, triangle_rect, resp.hovered(), ui);
                        if resp.clicked() {
                            set_ctx_bool_id(ui.ctx(), collapse_override_id, !collapsed_inner);
                            set_ctx_bool_id(ui.ctx(), collapse_inner_id, !collapsed_inner);
                        }
                        Frame::new()
                            .inner_margin(Margin {
                                left: 8,
                                right: 8,
                                top: 0,
                                bottom: 4,
                            })
                            .show(ui, |ui| changed |= f(ui));
                        if resp.clicked() {
                            clear_ctx_bool_id(ui.ctx(), collapse_override_id);
                        }
                    }
                }
            });
        })
        .response;
    set_ctx_bool_id(ui.ctx(), hovered_id, resp.hovered());
    changed
}

fn show_triangle(openness: f32, rect: Rect, hovered: bool, ui: &mut Ui) {
    let rect = Rect::from_center_size(rect.center(), egui::vec2(rect.width(), rect.height()) * 0.5);
    let mut points = vec![rect.left_top(), rect.right_top(), rect.center_bottom()];
    let rotation = emath::Rot2::from_angle(remap(1.0 - openness, 0.0..=1.0, -TAU / 4.0..=0.0));
    for p in &mut points {
        *p = rect.center() + rotation * (*p - rect.center());
    }
    ui.painter().add(egui::Shape::convex_polygon(
        points,
        TRANSPARENT,
        if hovered { STROKE_YELLOW } else { STROKE_DARK },
    ));
}

pub trait DataFramed: ToCstr + Clone + Debug + StringData + Inject {
    fn default_open(&self) -> bool {
        true
    }
    fn has_header(&self) -> bool;
    fn has_body(&self) -> bool;
    fn show_header(&self, context: &Context, ui: &mut Ui);
    fn show_header_mut(&mut self, ui: &mut Ui) -> bool;
    fn show_body(&self, context: &Context, ui: &mut Ui);
    fn show_body_mut(&mut self, ui: &mut Ui) -> bool;
    fn show_name(&self, ui: &mut Ui) {
        self.cstr().label(ui);
    }
    fn show_name_mut(&mut self, ui: &mut Ui) -> bool {
        self.show_name(ui);
        false
    }
    fn ui(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        let has_header = self.has_header();
        let has_body = self.has_body();
        let mut df = DataFrame::new(self)
            .prefix(prefix)
            .default_open(self.default_open());
        if has_header {
            let context = context.clone();
            df = df.header(move |d, ui| d.show_header(&context, ui));
        }
        if has_body {
            let context = context.clone();
            df = df.body(move |d, ui| d.show_body(&context, ui));
        }
        df.ui(ui);
    }
    fn ui_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        let has_header = self.has_header();
        let has_body = self.has_body();
        let default_open = self.default_open();
        let mut df = DataFrameMut::new_inject(self)
            .prefix(prefix)
            .default_open(default_open);
        df.name = Some(Box::new(|d, ui| d.show_name_mut(ui)));
        if has_header {
            df = df.header(move |d, ui| d.show_header_mut(ui));
        }
        if has_body {
            df = df.body(move |d, ui| d.show_body_mut(ui));
        }
        df.ui(ui)
    }
}

impl<T> Show for T
where
    T: ?Sized + DataFramed,
{
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        self.ui(prefix, context, ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        self.ui_mut(prefix, ui)
    }
}

fn show_mut_vec<T: Show + Default + Serialize + DeserializeOwned>(
    v: &mut Vec<T>,
    prefix: Option<&str>,
    ui: &mut Ui,
) -> bool {
    prefix.show(ui);
    let mut changed = false;
    let mut swap = None;
    let mut delete = None;
    let mut insert = None;
    let len = v.len();
    fn plus_btn(ui: &mut Ui) -> bool {
        "+".cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold)
            .button(ui)
            .clicked()
    }
    for (i, a) in v.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if i > 0 && "<".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                        swap = Some((i, i - 1));
                    }
                    if i + 1 < len && ">".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                        swap = Some((i, i + 1));
                    }
                });
                ui.horizontal(|ui| {
                    if "-".cstr_cs(RED, CstrStyle::Bold).button(ui).clicked() {
                        delete = Some(i);
                    }
                    if plus_btn(ui) {
                        insert = Some(i + 1);
                    }
                });
            });
            changed |= a.show_mut(Some(&i.to_string()), ui);
        });
    }
    if v.is_empty() && plus_btn(ui) {
        insert = Some(0);
    }
    if let Some(delete) = delete {
        changed = true;
        v.remove(delete);
    }
    if let Some(index) = insert {
        changed = true;
        v.insert(index, default());
    }
    if let Some((a, b)) = swap {
        changed = true;
        v.swap(a, b);
    }
    changed
}

impl<T> DataFramed for Vec<T>
where
    T: Show + Default + Serialize + DeserializeOwned + Debug + Clone + ToCstr,
{
    fn has_header(&self) -> bool {
        false
    }
    fn has_body(&self) -> bool {
        true
    }
    fn show_header(&self, _: &Context, _: &mut Ui) {}
    fn show_header_mut(&mut self, ui: &mut Ui) -> bool {
        false
    }
    fn show_body(&self, context: &Context, ui: &mut Ui) {
        for (i, v) in self.into_iter().enumerate() {
            v.show(Some(&format!("[vd {i}:]")), context, ui);
        }
    }
    fn show_body_mut(&mut self, ui: &mut Ui) -> bool {
        show_mut_vec(self, None, ui)
    }
}

impl<T> DataFramed for Box<T>
where
    T: Show + Default + Serialize + DeserializeOwned + Debug + Clone + ToCstr,
{
    fn ui(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        self.deref().show(prefix, context, ui);
    }
    fn ui_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        self.deref_mut().show_mut(prefix, ui)
    }
    fn has_header(&self) -> bool {
        false
    }
    fn has_body(&self) -> bool {
        false
    }
    fn show_header(&self, _: &Context, _: &mut Ui) {}
    fn show_header_mut(&mut self, _: &mut Ui) -> bool {
        false
    }
    fn show_body(&self, _: &Context, _: &mut Ui) {}
    fn show_body_mut(&mut self, _: &mut Ui) -> bool {
        false
    }
}

impl DataFramed for Expression {
    fn show_name_mut(&mut self, ui: &mut Ui) -> bool {
        Selector::from_mut(self, ui)
    }
    fn has_header(&self) -> bool {
        match self {
            Expression::Var(_)
            | Expression::V(_)
            | Expression::S(_)
            | Expression::F(_)
            | Expression::FSlider(_)
            | Expression::I(_)
            | Expression::B(_)
            | Expression::V2(_, _)
            | Expression::StateVar(_, _)
            | Expression::C(_) => true,
            Expression::One
            | Expression::Zero
            | Expression::PI
            | Expression::PI2
            | Expression::GT
            | Expression::Owner
            | Expression::Target
            | Expression::UnitSize
            | Expression::AllUnits
            | Expression::AllAllyUnits
            | Expression::AllOtherAllyUnits
            | Expression::AdjacentAllyUnits
            | Expression::AdjacentBack
            | Expression::AdjacentFront
            | Expression::AllEnemyUnits
            | Expression::Sin(..)
            | Expression::Cos(..)
            | Expression::Even(..)
            | Expression::Abs(..)
            | Expression::Floor(..)
            | Expression::Ceil(..)
            | Expression::Fract(..)
            | Expression::UnitVec(..)
            | Expression::Rand(..)
            | Expression::RandomUnit(..)
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
            | Expression::ToF(..)
            | Expression::Oklch(..)
            | Expression::Fallback(..)
            | Expression::If(..) => false,
        }
    }
    fn has_body(&self) -> bool {
        match self {
            Expression::One
            | Expression::Zero
            | Expression::PI
            | Expression::PI2
            | Expression::GT
            | Expression::Owner
            | Expression::Target
            | Expression::UnitSize
            | Expression::AllUnits
            | Expression::AllAllyUnits
            | Expression::AllOtherAllyUnits
            | Expression::AdjacentAllyUnits
            | Expression::AdjacentBack
            | Expression::AdjacentFront
            | Expression::AllEnemyUnits
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
            | Expression::UnitVec(..)
            | Expression::Rand(..)
            | Expression::RandomUnit(..)
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
            | Expression::FSlider(..)
            | Expression::ToF(..)
            | Expression::Oklch(..)
            | Expression::StateVar(..)
            | Expression::Fallback(..)
            | Expression::If(..) => true,
        }
    }
    fn show_header(&self, context: &Context, ui: &mut Ui) {
        match self {
            Expression::Var(v) | Expression::StateVar(_, v) => v.show(Some("x"), &context, ui),
            Expression::V(v) => v.show(Some("x"), &context, ui),
            Expression::S(v) => v.show(Some("x"), &context, ui),
            Expression::F(v) | Expression::FSlider(v) => v.show(Some("x"), &context, ui),
            Expression::I(v) => v.show(Some("x"), &context, ui),
            Expression::B(v) => v.show(Some("x"), &context, ui),
            Expression::V2(x, y) => {
                x.show(Some("x"), &context, ui);
                y.show(Some("y"), &context, ui);
            }
            Expression::C(v) => v.show(Some("c"), &context, ui),
            _ => {}
        }
    }
    fn show_header_mut(&mut self, ui: &mut Ui) -> bool {
        match self {
            Expression::Var(v) | Expression::StateVar(_, v) => v.show_mut(Some("x"), ui),
            Expression::V(v) => v.show_mut(Some("x"), ui),
            Expression::S(v) => v.show_mut(Some("x"), ui),
            Expression::F(v) => v.show_mut(Some("x"), ui),
            Expression::I(v) => v.show_mut(Some("x"), ui),
            Expression::B(v) => v.show_mut(Some("x"), ui),
            Expression::C(v) => match Color32::from_hex(v) {
                Ok(mut c) => {
                    v.cstr_c(c).label(ui);
                    let changed = c.show_mut(None, ui);
                    if changed {
                        *v = c.to_hex();
                    }
                    changed
                }
                Err(e) => {
                    error!("Hex color parse error: {e:?}");
                    *v = "#ffffff".into();
                    true
                }
            },
            Expression::V2(x, y) => {
                let x = x.show_mut(Some("x"), ui);
                y.show_mut(Some("y"), ui) || x
            }
            _ => false,
        }
    }
    fn show_body(&self, context: &Context, ui: &mut Ui) {
        match self {
            Expression::Sin(x)
            | Expression::Cos(x)
            | Expression::Even(x)
            | Expression::Abs(x)
            | Expression::Floor(x)
            | Expression::Ceil(x)
            | Expression::Fract(x)
            | Expression::UnitVec(x)
            | Expression::Rand(x)
            | Expression::RandomUnit(x)
            | Expression::ToF(x)
            | Expression::StateVar(x, _)
            | Expression::Sqr(x) => x.show(Some("x"), &context, ui),
            Expression::V2EE(a, b)
            | Expression::Macro(a, b)
            | Expression::Sum(a, b)
            | Expression::Sub(a, b)
            | Expression::Mul(a, b)
            | Expression::Div(a, b)
            | Expression::Max(a, b)
            | Expression::Min(a, b)
            | Expression::Mod(a, b)
            | Expression::And(a, b)
            | Expression::Or(a, b)
            | Expression::Equals(a, b)
            | Expression::GreaterThen(a, b)
            | Expression::LessThen(a, b) => {
                a.show(Some("a"), &context, ui);
                b.show(Some("b"), &context, ui);
            }
            Expression::Fallback(v, e) => {
                v.show(Some("v"), &context, ui);
                e.show(Some("on_err"), &context, ui);
            }
            Expression::Oklch(a, b, c) => {
                a.show(Some("lightness"), &context, ui);
                b.show(Some("chroma"), &context, ui);
                c.show(Some("hue"), &context, ui);
            }
            Expression::If(a, b, c) => {
                a.show(Some("if"), &context, ui);
                b.show(Some("then"), &context, ui);
                c.show(Some("else"), &context, ui);
            }
            _ => {}
        };
    }
    fn show_body_mut(&mut self, ui: &mut Ui) -> bool {
        match self {
            Expression::Sin(x)
            | Expression::Cos(x)
            | Expression::Even(x)
            | Expression::Abs(x)
            | Expression::Floor(x)
            | Expression::Ceil(x)
            | Expression::Fract(x)
            | Expression::UnitVec(x)
            | Expression::Rand(x)
            | Expression::RandomUnit(x)
            | Expression::ToF(x)
            | Expression::StateVar(x, _)
            | Expression::Sqr(x) => x.show_mut(Some("x"), ui),
            Expression::V2EE(a, b)
            | Expression::Macro(a, b)
            | Expression::Sum(a, b)
            | Expression::Sub(a, b)
            | Expression::Mul(a, b)
            | Expression::Div(a, b)
            | Expression::Max(a, b)
            | Expression::Min(a, b)
            | Expression::Mod(a, b)
            | Expression::And(a, b)
            | Expression::Or(a, b)
            | Expression::Equals(a, b)
            | Expression::GreaterThen(a, b)
            | Expression::LessThen(a, b) => {
                let a = a.show_mut(Some("a"), ui);
                b.show_mut(Some("b"), ui) || a
            }
            Expression::Fallback(v, e) => {
                let v = v.show_mut(Some("v"), ui);
                e.show_mut(Some("on_err"), ui) || v
            }
            Expression::Oklch(a, b, c) => {
                let a = a.show_mut(Some("lightness"), ui);
                let b = b.show_mut(Some("chroma"), ui);
                c.show_mut(Some("hue"), ui) || a || b
            }
            Expression::If(a, b, c) => {
                let a = a.show_mut(Some("if"), ui);
                let b = b.show_mut(Some("then"), ui);
                c.show_mut(Some("else"), ui) || a || b
            }
            Expression::FSlider(x) => Slider::new("x").full_width().ui(x, 0.0..=1.0, ui),
            _ => false,
        }
    }
}

impl DataFramed for PainterAction {
    fn show_name_mut(&mut self, ui: &mut Ui) -> bool {
        Selector::from_mut(self, ui)
    }
    fn has_header(&self) -> bool {
        false
    }
    fn has_body(&self) -> bool {
        match self {
            PainterAction::Paint => false,
            PainterAction::Circle(..)
            | PainterAction::Rectangle(..)
            | PainterAction::Curve { .. }
            | PainterAction::Text(..)
            | PainterAction::Hollow(..)
            | PainterAction::Translate(..)
            | PainterAction::Rotate(..)
            | PainterAction::ScaleMesh(..)
            | PainterAction::ScaleRect(..)
            | PainterAction::Color(..)
            | PainterAction::Feathering(..)
            | PainterAction::Alpha(..)
            | PainterAction::Repeat(..)
            | PainterAction::List(..) => true,
        }
    }
    fn show_header(&self, _: &Context, _: &mut Ui) {}
    fn show_header_mut(&mut self, _: &mut Ui) -> bool {
        false
    }
    fn show_body(&self, context: &Context, ui: &mut Ui) {
        match self {
            PainterAction::Paint => {}
            PainterAction::Circle(x)
            | PainterAction::Rectangle(x)
            | PainterAction::Text(x)
            | PainterAction::Hollow(x)
            | PainterAction::Translate(x)
            | PainterAction::Rotate(x)
            | PainterAction::ScaleMesh(x)
            | PainterAction::ScaleRect(x)
            | PainterAction::Color(x)
            | PainterAction::Feathering(x)
            | PainterAction::Alpha(x) => x.show(Some("x"), context, ui),
            PainterAction::Curve {
                thickness,
                curvature,
            } => {
                thickness.show(Some("thickness"), context, ui);
                curvature.show(Some("curvature"), context, ui);
            }
            PainterAction::Repeat(x, painter_action) => {
                x.show(Some("cnt"), context, ui);
                painter_action.show(Some("action"), context, ui);
            }
            PainterAction::List(vec) => vec.show(None, context, ui),
        }
    }
    fn show_body_mut(&mut self, ui: &mut Ui) -> bool {
        match self {
            PainterAction::Paint => false,
            PainterAction::Circle(x)
            | PainterAction::Rectangle(x)
            | PainterAction::Text(x)
            | PainterAction::Hollow(x)
            | PainterAction::Translate(x)
            | PainterAction::Rotate(x)
            | PainterAction::ScaleMesh(x)
            | PainterAction::ScaleRect(x)
            | PainterAction::Color(x)
            | PainterAction::Feathering(x)
            | PainterAction::Alpha(x) => x.show_mut(Some("x"), ui),
            PainterAction::Repeat(x, painter_action) => {
                let x = x.show_mut(Some("cnt"), ui);
                painter_action.show_mut(Some("action"), ui) || x
            }
            PainterAction::Curve {
                thickness,
                curvature,
            } => {
                let thickness = thickness.show_mut(Some("thickness"), ui);
                curvature.show_mut(Some("curvature"), ui) || thickness
            }
            PainterAction::List(vec) => vec.show_mut(None, ui),
        }
    }
}
impl DataFramed for Action {
    fn default_open(&self) -> bool {
        match self {
            Action::UseAbility => true,
            _ => false,
        }
    }
    fn show_name_mut(&mut self, ui: &mut Ui) -> bool {
        Selector::from_mut(self, ui)
    }
    fn has_header(&self) -> bool {
        match self {
            Action::UseAbility => true,
            _ => false,
        }
    }
    fn has_body(&self) -> bool {
        match self {
            Action::Noop | Action::DealDamage | Action::HealDamage | Action::UseAbility => false,
            Action::Debug(..)
            | Action::SetValue(..)
            | Action::AddValue(..)
            | Action::SubtractValue(..)
            | Action::AddTarget(..)
            | Action::Repeat(..) => true,
        }
    }
    fn show_header(&self, context: &Context, ui: &mut Ui) {
        match self {
            Action::UseAbility => {
                let Ok(entity) = context.get_caster().or_else(|_| context.get_owner()) else {
                    return;
                };
                if let Some(color) = context
                    .find_parent_component::<HouseColor>(entity)
                    .map(|h| h.color.c32())
                {
                    if let Some(name) = context
                        .find_parent_component::<ActionAbility>(entity)
                        .map(|a| a.name.clone())
                        .or_else(|| {
                            context
                                .find_parent_component::<StatusAbility>(entity)
                                .map(|a| a.name.clone())
                        })
                    {
                        name.cstr_c(color).label(ui);
                    }
                }
            }
            _ => {}
        }
    }
    fn show_header_mut(&mut self, _ui: &mut Ui) -> bool {
        false
    }
    fn show_body(&self, context: &Context, ui: &mut Ui) {
        match self {
            Action::DealDamage | Action::HealDamage | Action::UseAbility | Action::Noop => {}
            Action::Debug(x)
            | Action::SetValue(x)
            | Action::AddValue(x)
            | Action::SubtractValue(x)
            | Action::AddTarget(x) => {
                x.show(Some("x"), context, ui);
            }
            Action::Repeat(x, vec) => {
                x.show(Some("x"), context, ui);
                vec.show(None, context, ui);
            }
        }
    }

    fn show_body_mut(&mut self, ui: &mut Ui) -> bool {
        match self {
            Action::DealDamage | Action::HealDamage | Action::UseAbility | Action::Noop => false,
            Action::Debug(x)
            | Action::SetValue(x)
            | Action::AddValue(x)
            | Action::SubtractValue(x)
            | Action::AddTarget(x) => x.show_mut(Some("x"), ui),
            Action::Repeat(x, vec) => {
                let x = x.show_mut(Some("x"), ui);
                vec.show_mut(None, ui) || x
            }
        }
    }
}
impl DataFramed for Trigger {
    fn show_name_mut(&mut self, ui: &mut Ui) -> bool {
        Selector::from_mut(self, ui)
    }
    fn has_header(&self) -> bool {
        match self {
            Trigger::BattleStart | Trigger::TurnEnd | Trigger::BeforeDeath => false,
            Trigger::ChangeStat(..) => true,
        }
    }
    fn has_body(&self) -> bool {
        false
    }
    fn show_header(&self, context: &Context, ui: &mut Ui) {
        match self {
            Trigger::BattleStart | Trigger::TurnEnd | Trigger::BeforeDeath => {}
            Trigger::ChangeStat(var) => var.show(None, context, ui),
        }
    }
    fn show_header_mut(&mut self, ui: &mut Ui) -> bool {
        match self {
            Trigger::BattleStart | Trigger::TurnEnd | Trigger::BeforeDeath => false,
            Trigger::ChangeStat(var) => var.show_mut(None, ui),
        }
    }
    fn show_body(&self, _context: &Context, _ui: &mut Ui) {}
    fn show_body_mut(&mut self, _ui: &mut Ui) -> bool {
        false
    }
}

const FORCE_OPEN_ID: &str = "dataframe_force_open";
pub trait DataFrameUiExt {
    fn data_frame_force_open(&mut self);
    fn data_frame_is_force_open(&self) -> bool;
}

impl DataFrameUiExt for Ui {
    fn data_frame_force_open(&mut self) {
        self.ctx().set_frame_flag(FORCE_OPEN_ID);
    }
    fn data_frame_is_force_open(&self) -> bool {
        self.ctx().get_frame_flag(FORCE_OPEN_ID)
    }
}
