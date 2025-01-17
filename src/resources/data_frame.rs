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
    header: Option<Box<dyn FnOnce(&T, &mut Ui) + 'a>>,
    body: Option<Box<dyn FnOnce(&T, &mut Ui) + 'a>>,
    context_actions: HashMap<&'static str, Box<dyn FnOnce(&T) + 'a>>,
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
        }
    }
    pub fn prefix(mut self, prefix: Option<&'a str>) -> Self {
        self.prefix = prefix;
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
            self.data.cstr_s(CstrStyle::Bold).label(ui);
            false
        };
        let context_actions = HashMap::from_iter(self.context_actions.into_iter().map(|(k, v)| {
            (k, || {
                v(data.borrow().deref());
                false
            })
        }));
        let r = compose_ui(self.prefix, header, body, name, context_actions, ui);
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
                    d.inject_data(&c);
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

impl<T> Show for T
where
    T: DataFramed,
{
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        let has_header = self.has_header();
        let has_body = self.has_body();
        let mut df = DataFrame::new(self).prefix(prefix);
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
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        let has_header = self.has_header();
        let has_body = self.has_body();
        let mut df = DataFrameMut::new_inject(self).prefix(prefix);
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

pub trait DataFramed: ToCstr + Clone + Debug + StringData + Inject {
    fn has_header(&self) -> bool;
    fn has_body(&self) -> bool;
    fn show_header(&self, context: &Context, ui: &mut Ui);
    fn show_header_mut(&mut self, ui: &mut Ui) -> bool;
    fn show_body(&self, context: &Context, ui: &mut Ui);
    fn show_body_mut(&mut self, ui: &mut Ui) -> bool;
    fn show_name(&self, ui: &mut Ui) {
        self.cstr_s(CstrStyle::Bold).label(ui);
    }
    fn show_name_mut(&mut self, ui: &mut Ui) -> bool {
        self.show_name(ui);
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
            | Expression::I(_)
            | Expression::B(_)
            | Expression::V2(_, _)
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
            | Expression::If(..) => true,
        }
    }
    fn show_header(&self, context: &Context, ui: &mut Ui) {
        match self {
            Expression::Var(v) => v.show(Some("x:"), &context, ui),
            Expression::V(v) => v.show(Some("x:"), &context, ui),
            Expression::S(v) => v.show(Some("x:"), &context, ui),
            Expression::F(v) => v.show(Some("x:"), &context, ui),
            Expression::I(v) => v.show(Some("x:"), &context, ui),
            Expression::B(v) => v.show(Some("x:"), &context, ui),
            Expression::V2(x, y) => {
                x.show(Some("x:"), &context, ui);
                y.show(Some("y:"), &context, ui);
            }
            Expression::C(v) => v.show(Some("c:"), &context, ui),
            _ => {}
        }
    }
    fn show_header_mut(&mut self, ui: &mut Ui) -> bool {
        match self {
            Expression::Var(v) => v.show_mut(Some("x:"), ui),
            Expression::V(v) => v.show_mut(Some("x:"), ui),
            Expression::S(v) => v.show_mut(Some("x:"), ui),
            Expression::F(v) => v.show_mut(Some("x:"), ui),
            Expression::I(v) => v.show_mut(Some("x:"), ui),
            Expression::B(v) => v.show_mut(Some("x:"), ui),
            Expression::C(v) => match Color32::from_hex(v) {
                Ok(mut c) => {
                    v.cstr_cs(c, CstrStyle::Bold).label(ui);
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
                let x = x.show_mut(Some("x:"), ui);
                y.show_mut(Some("y:"), ui) || x
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
            | Expression::Sqr(x) => x.show(Some("x:"), &context, ui),
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
                a.show(Some("a:"), &context, ui);
                b.show(Some("b:"), &context, ui);
            }
            Expression::If(a, b, c) => {
                a.show(Some("if:"), &context, ui);
                b.show(Some("then:"), &context, ui);
                c.show(Some("else:"), &context, ui);
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
            | Expression::Sqr(x) => x.show_mut(Some("x:"), ui),
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
                let a = a.show_mut(Some("a:"), ui);
                b.show_mut(Some("b:"), ui) || a
            }
            Expression::If(a, b, c) => {
                let a = a.show_mut(Some("if:"), ui);
                let b = b.show_mut(Some("then:"), ui);
                c.show_mut(Some("else:"), ui) || a || b
            }
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
            | PainterAction::Text(..)
            | PainterAction::Hollow(..)
            | PainterAction::Translate(..)
            | PainterAction::Rotate(..)
            | PainterAction::Scale(..)
            | PainterAction::Color(..)
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
            | PainterAction::Scale(x)
            | PainterAction::Color(x)
            | PainterAction::Alpha(x) => x.show(Some("x:"), context, ui),
            PainterAction::Repeat(x, painter_action) => {
                x.show(Some("cnt:"), context, ui);
                painter_action.show(Some("action:"), context, ui);
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
            | PainterAction::Scale(x)
            | PainterAction::Color(x)
            | PainterAction::Alpha(x) => x.show_mut(Some("x:"), ui),
            PainterAction::Repeat(x, painter_action) => {
                let x = x.show_mut(Some("cnt:"), ui);
                painter_action.show_mut(Some("action:"), ui) || x
            }
            PainterAction::List(vec) => vec.show_mut(None, ui),
        }
    }
}
