use serde::de::DeserializeOwned;

use super::*;

pub struct DataFrameMut<'a, T> {
    data: &'a mut T,
    prefix: Option<&'a str>,
    header: Option<Box<dyn FnOnce(&mut T, &mut Ui) -> bool>>,
    body: Option<Box<dyn FnOnce(&mut T, &mut Ui) -> bool>>,
    name: Option<Box<dyn FnOnce(&mut T, &mut Ui) -> DataFrameResponse>>,
    context_actions: HashMap<&'static str, Box<dyn FnOnce(&mut T) -> bool>>,
    settings: DataFrameSettings,
}

pub struct DataFrame<'a, T> {
    data: &'a T,
    prefix: Option<&'a str>,
    header: Option<Box<dyn FnOnce(&T, &mut Ui) + 'a>>,
    body: Option<Box<dyn FnOnce(&T, &mut Ui) + 'a>>,
    name: Option<Box<dyn FnOnce(&T, &mut Ui) -> DataFrameResponse + 'a>>,
    context_actions: HashMap<&'static str, Box<dyn FnOnce(&T)>>,
    settings: DataFrameSettings,
}

#[derive(Default)]
struct DataFrameSettings {
    default_open: bool,
    highlighted: bool,
}

impl DataFrameSettings {}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DataFrameResponse {
    None,
    NameClicked,
    Changed,
}

impl DataFrameResponse {
    pub fn changed(self) -> bool {
        self == Self::Changed
    }
    pub fn name_clicked(self) -> bool {
        self == Self::NameClicked
    }
}

fn frame() -> Frame {
    Frame {
        inner_margin: Margin::ZERO,
        outer_margin: Margin::ZERO,
        corner_radius: ROUNDING,
        shadow: Shadow::NONE,
        fill: TRANSPARENT,
        stroke: Stroke::new(1.0, tokens_global().subtle_borders_and_separators()),
    }
}

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
            name: None,
            body: None,
            prefix: None,
            context_actions,
            settings: default(),
        }
    }
    pub fn prefix(mut self, prefix: Option<&'a str>) -> Self {
        self.prefix = prefix;
        self
    }
    pub fn highlighted(mut self, value: bool) -> Self {
        self.settings.highlighted = value;
        self
    }
    pub fn default_open(mut self, value: bool) -> Self {
        self.settings.default_open = value;
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
    pub fn ui(self, ui: &mut Ui) -> DataFrameResponse {
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
            if let Some(name) = self.name {
                name(data.borrow_mut().deref_mut(), ui)
            } else {
                if self.data.cstr().button(ui).clicked() {
                    DataFrameResponse::NameClicked
                } else {
                    DataFrameResponse::None
                }
            }
        };
        let context_actions = HashMap::from_iter(self.context_actions.into_iter().map(|(k, v)| {
            (k, || {
                v(data.borrow().deref());
                false
            })
        }));
        let mut settings = self.settings;
        if ui.data_frame_is_force_open() {
            settings.default_open = true;
        }
        let r = compose_ui(
            self.prefix,
            header,
            body,
            name,
            context_actions,
            settings,
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
            settings: default(),
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
                DataFrameResponse::Changed
            } else {
                DataFrameResponse::None
            }
        }));
        r
    }
    pub fn prefix(mut self, prefix: Option<&'a str>) -> Self {
        self.prefix = prefix;
        self
    }
    pub fn highlighted(mut self, value: bool) -> Self {
        self.settings.highlighted = value;
        self
    }
    pub fn default_open(mut self, value: bool) -> Self {
        self.settings.default_open = value;
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
    pub fn ui(self, ui: &mut Ui) -> DataFrameResponse {
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
                if self.data.cstr().button(ui).clicked() {
                    DataFrameResponse::NameClicked
                } else {
                    DataFrameResponse::None
                }
            }
        };
        let context_actions = HashMap::from_iter(
            self.context_actions
                .into_iter()
                .map(|(k, v)| (k, || v(data.borrow_mut().deref_mut()))),
        );
        let mut settings = self.settings;
        if ui.data_frame_is_force_open() {
            settings.default_open = true;
        }
        let r = compose_ui(
            self.prefix,
            header,
            body,
            name,
            context_actions,
            settings,
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
    name: impl FnOnce(&mut Ui) -> DataFrameResponse,
    context_actions: HashMap<&'static str, impl FnOnce() -> bool>,
    settings: DataFrameSettings,
    ui: &mut Ui,
) -> DataFrameResponse {
    let mut changed = false;
    let mut df_response = DataFrameResponse::None;
    let id = ui.next_auto_id();
    let collapsed_id = id.with("collapsed");
    let collapse_inner_id = id.with("collapse_inner");
    let collapse_override_id = Id::new("collapse_override");
    let hovered_id = id.with("hovered");
    let mut collapsed = get_ctx_bool_id_default(ui.ctx(), collapsed_id, !settings.default_open);
    let collapsed_inner = get_ctx_bool_id_default(ui.ctx(), collapse_inner_id, true);
    if let Some(collapse_override) = get_ctx_bool_id(ui.ctx(), collapse_override_id) {
        collapsed = collapse_override;
        set_ctx_bool_id(ui.ctx(), collapsed_id, collapse_override);
        set_ctx_bool_id(ui.ctx(), collapse_inner_id, collapse_override);
    }
    let openness = ui.ctx().animate_bool(collapsed_id, collapsed);
    let openness_inner = ui.ctx().animate_bool(collapse_inner_id, collapsed_inner);
    let hovered = get_ctx_bool_id_default(ui.ctx(), hovered_id, false);

    const R: u8 = ROUNDING.ne;
    let header_rounding = CornerRadius {
        nw: R,
        ne: if header.is_none() || collapsed { R } else { 0 },
        sw: if body.is_none() || collapsed { R } else { 0 },
        se: R,
    };
    let mut header_rect = Rect::ZERO;
    let mut triangle_rect = Rect::ZERO;
    let resp = frame()
        .stroke(Stroke::new(
            1.0,
            if settings.highlighted {
                YELLOW
            } else if hovered {
                tokens_global().hovered_ui_element_border()
            } else {
                tokens_global().subtle_borders_and_separators()
            },
        ))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    header_rect = frame()
                        .fill(tokens_global().subtle_background())
                        .inner_margin(Margin::symmetric(8, 4))
                        .corner_radius(header_rounding)
                        .show(ui, |ui| {
                            if let Some(prefix) = prefix {
                                format!("[tl [s {prefix}]]").label(ui);
                            }
                            df_response = name(ui);
                            if !context_actions.is_empty() {
                                let (_, resp) = ui.allocate_at_least(
                                    egui::vec2(8.0, ui.available_height()),
                                    Sense::click(),
                                );
                                ui.painter().circle(
                                    resp.rect.center(),
                                    4.0,
                                    if resp.hovered() {
                                        tokens_global().hovered_solid_backgrounds()
                                    } else {
                                        tokens_global().subtle_background()
                                    },
                                    Stroke::new(
                                        1.0,
                                        tokens_global().subtle_borders_and_separators(),
                                    ),
                                );
                                resp.bar_menu(|ui| {
                                    for (name, action) in context_actions {
                                        if ui.button(name).clicked() {
                                            action();
                                            changed = true;
                                            ui.close_menu();
                                        }
                                    }
                                    if "[tl Close]".cstr().button(ui).clicked() {
                                        ui.close_menu();
                                    }
                                });
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
    if changed {
        DataFrameResponse::Changed
    } else {
        df_response
    }
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
        if hovered {
            Stroke::new(1.0, tokens_global().hovered_ui_element_border())
        } else {
            Stroke::new(1.0, tokens_global().subtle_borders_and_separators())
        },
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
    fn show_name(&self, _context: &Context, ui: &mut Ui) -> DataFrameResponse {
        if self.cstr().button(ui).clicked() {
            DataFrameResponse::NameClicked
        } else {
            DataFrameResponse::None
        }
    }
    fn show_name_mut(&mut self, ui: &mut Ui) -> DataFrameResponse {
        self.show_name(&default(), ui)
    }
    fn add_context_actions_mut(
        _map: &mut HashMap<&'static str, Box<dyn FnOnce(&mut Self) -> bool>>,
    ) {
    }
    fn add_context_actions(_map: &mut HashMap<&'static str, Box<dyn FnOnce(&Self)>>) {}
    fn df<'a>(&'a self, context: &'a Context) -> DataFrame<'a, Self> {
        let has_header = self.has_header();
        let has_body = self.has_body();
        let mut df = DataFrame::new(self).default_open(self.default_open());
        df.name = Some(Box::new(|d, ui| d.show_name(context, ui)));
        Self::add_context_actions(&mut df.context_actions);
        if has_header {
            df = df.header(move |d, ui| d.show_header(&context, ui));
        }
        if has_body {
            let context = context.clone();
            df = df.body(move |d, ui| d.show_body(&context, ui));
        }
        df
    }
    fn df_mut<'a>(&'a mut self) -> DataFrameMut<'a, Self> {
        let has_header = self.has_header();
        let has_body = self.has_body();
        let default_open = self.default_open();
        let mut df = DataFrameMut::new_inject(self).default_open(default_open);
        df.name = Some(Box::new(|d, ui| d.show_name_mut(ui)));
        Self::add_context_actions_mut(&mut df.context_actions);
        if has_header {
            df = df.header(move |d, ui| d.show_header_mut(ui));
        }
        if has_body {
            df = df.body(move |d, ui| d.show_body_mut(ui));
        }
        df
    }
    fn ui(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) -> DataFrameResponse {
        self.df(context).prefix(prefix).ui(ui)
    }
    fn ui_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> DataFrameResponse {
        self.df_mut().prefix(prefix).ui(ui)
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
        self.ui_mut(prefix, ui).changed()
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
        "+".cstr_cs(tokens_global().high_contrast_text(), CstrStyle::Bold)
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
            v.show(Some(&format!("[tl {i}:]")), context, ui);
        }
    }
    fn show_body_mut(&mut self, ui: &mut Ui) -> bool {
        show_mut_vec(self, None, ui)
    }
}

impl DataFramed for Expression {
    fn show_name_mut(&mut self, ui: &mut Ui) -> DataFrameResponse {
        if Selector::from_mut(self, ui) {
            DataFrameResponse::Changed
        } else {
            DataFrameResponse::None
        }
    }
    fn has_header(&self) -> bool {
        match self {
            Expression::var(_)
            | Expression::value(_)
            | Expression::string(_)
            | Expression::f32(_)
            | Expression::f32_slider(_)
            | Expression::i32(_)
            | Expression::bool(_)
            | Expression::vec2(_, _)
            | Expression::state_var(_, _)
            | Expression::color(_) => true,
            Expression::one
            | Expression::zero
            | Expression::pi
            | Expression::pi2
            | Expression::gt
            | Expression::owner
            | Expression::target
            | Expression::unit_size
            | Expression::all_units
            | Expression::all_ally_units
            | Expression::all_other_ally_units
            | Expression::adjacent_ally_units
            | Expression::adjacent_back
            | Expression::adjacent_front
            | Expression::all_enemy_units
            | Expression::sin(..)
            | Expression::cos(..)
            | Expression::even(..)
            | Expression::abs(..)
            | Expression::floor(..)
            | Expression::ceil(..)
            | Expression::fract(..)
            | Expression::unit_vec(..)
            | Expression::rand(..)
            | Expression::random_unit(..)
            | Expression::sqr(..)
            | Expression::vec2_ee(..)
            | Expression::str_macro(..)
            | Expression::sum(..)
            | Expression::sub(..)
            | Expression::mul(..)
            | Expression::div(..)
            | Expression::max(..)
            | Expression::min(..)
            | Expression::r#mod(..)
            | Expression::and(..)
            | Expression::or(..)
            | Expression::equals(..)
            | Expression::greater_then(..)
            | Expression::less_then(..)
            | Expression::to_f32(..)
            | Expression::oklch(..)
            | Expression::fallback(..)
            | Expression::r#if(..) => false,
        }
    }
    fn has_body(&self) -> bool {
        match self {
            Expression::one
            | Expression::zero
            | Expression::pi
            | Expression::pi2
            | Expression::gt
            | Expression::owner
            | Expression::target
            | Expression::unit_size
            | Expression::all_units
            | Expression::all_ally_units
            | Expression::all_other_ally_units
            | Expression::adjacent_ally_units
            | Expression::adjacent_back
            | Expression::adjacent_front
            | Expression::all_enemy_units
            | Expression::var(..)
            | Expression::value(..)
            | Expression::string(..)
            | Expression::f32(..)
            | Expression::i32(..)
            | Expression::bool(..)
            | Expression::vec2(..)
            | Expression::color(..) => false,
            Expression::sin(..)
            | Expression::cos(..)
            | Expression::even(..)
            | Expression::abs(..)
            | Expression::floor(..)
            | Expression::ceil(..)
            | Expression::fract(..)
            | Expression::unit_vec(..)
            | Expression::rand(..)
            | Expression::random_unit(..)
            | Expression::sqr(..)
            | Expression::vec2_ee(..)
            | Expression::str_macro(..)
            | Expression::sum(..)
            | Expression::sub(..)
            | Expression::mul(..)
            | Expression::div(..)
            | Expression::max(..)
            | Expression::min(..)
            | Expression::r#mod(..)
            | Expression::and(..)
            | Expression::or(..)
            | Expression::equals(..)
            | Expression::greater_then(..)
            | Expression::less_then(..)
            | Expression::f32_slider(..)
            | Expression::to_f32(..)
            | Expression::oklch(..)
            | Expression::state_var(..)
            | Expression::fallback(..)
            | Expression::r#if(..) => true,
        }
    }
    fn show_header(&self, context: &Context, ui: &mut Ui) {
        match self {
            Expression::var(v) | Expression::state_var(_, v) => v.show(Some("x"), &context, ui),
            Expression::value(v) => v.show(Some("x"), &context, ui),
            Expression::string(v) => v.show(Some("x"), &context, ui),
            Expression::f32(v) | Expression::f32_slider(v) => v.show(Some("x"), &context, ui),
            Expression::i32(v) => v.show(Some("x"), &context, ui),
            Expression::bool(v) => v.show(Some("x"), &context, ui),
            Expression::vec2(x, y) => {
                x.show(Some("x"), &context, ui);
                y.show(Some("y"), &context, ui);
            }
            Expression::color(v) => v.show(Some("c"), &context, ui),
            _ => {}
        }
    }
    fn show_header_mut(&mut self, ui: &mut Ui) -> bool {
        match self {
            Expression::var(v) | Expression::state_var(_, v) => v.show_mut(Some("x"), ui),
            Expression::value(v) => v.show_mut(Some("x"), ui),
            Expression::string(v) => v.show_mut(Some("x"), ui),
            Expression::f32(v) => v.show_mut(Some("x"), ui),
            Expression::i32(v) => v.show_mut(Some("x"), ui),
            Expression::bool(v) => v.show_mut(Some("x"), ui),
            Expression::color(v) => false,
            Expression::vec2(x, y) => {
                let x = x.show_mut(Some("x"), ui);
                y.show_mut(Some("y"), ui) || x
            }
            _ => false,
        }
    }
    fn show_body(&self, context: &Context, ui: &mut Ui) {
        match self {
            Expression::sin(x)
            | Expression::cos(x)
            | Expression::even(x)
            | Expression::abs(x)
            | Expression::floor(x)
            | Expression::ceil(x)
            | Expression::fract(x)
            | Expression::unit_vec(x)
            | Expression::rand(x)
            | Expression::random_unit(x)
            | Expression::to_f32(x)
            | Expression::state_var(x, _)
            | Expression::sqr(x) => x.show(Some("x"), &context, ui),
            Expression::vec2_ee(a, b)
            | Expression::str_macro(a, b)
            | Expression::sum(a, b)
            | Expression::sub(a, b)
            | Expression::mul(a, b)
            | Expression::div(a, b)
            | Expression::max(a, b)
            | Expression::min(a, b)
            | Expression::r#mod(a, b)
            | Expression::and(a, b)
            | Expression::or(a, b)
            | Expression::equals(a, b)
            | Expression::greater_then(a, b)
            | Expression::less_then(a, b) => {
                a.show(Some("a"), &context, ui);
                b.show(Some("b"), &context, ui);
            }
            Expression::fallback(v, e) => {
                v.show(Some("v"), &context, ui);
                e.show(Some("on_err"), &context, ui);
            }
            Expression::oklch(a, b, c) => {
                a.show(Some("lightness"), &context, ui);
                b.show(Some("chroma"), &context, ui);
                c.show(Some("hue"), &context, ui);
            }
            Expression::r#if(a, b, c) => {
                a.show(Some("if"), &context, ui);
                b.show(Some("then"), &context, ui);
                c.show(Some("else"), &context, ui);
            }
            _ => {}
        };
    }
    fn show_body_mut(&mut self, ui: &mut Ui) -> bool {
        match self {
            Expression::sin(x)
            | Expression::cos(x)
            | Expression::even(x)
            | Expression::abs(x)
            | Expression::floor(x)
            | Expression::ceil(x)
            | Expression::fract(x)
            | Expression::unit_vec(x)
            | Expression::rand(x)
            | Expression::random_unit(x)
            | Expression::to_f32(x)
            | Expression::state_var(x, _)
            | Expression::sqr(x) => x.show_mut(Some("x"), ui),
            Expression::vec2_ee(a, b)
            | Expression::str_macro(a, b)
            | Expression::sum(a, b)
            | Expression::sub(a, b)
            | Expression::mul(a, b)
            | Expression::div(a, b)
            | Expression::max(a, b)
            | Expression::min(a, b)
            | Expression::r#mod(a, b)
            | Expression::and(a, b)
            | Expression::or(a, b)
            | Expression::equals(a, b)
            | Expression::greater_then(a, b)
            | Expression::less_then(a, b) => {
                let a = a.show_mut(Some("a"), ui);
                b.show_mut(Some("b"), ui) || a
            }
            Expression::fallback(v, e) => {
                let v = v.show_mut(Some("v"), ui);
                e.show_mut(Some("on_err"), ui) || v
            }
            Expression::oklch(a, b, c) => {
                let a = a.show_mut(Some("lightness"), ui);
                let b = b.show_mut(Some("chroma"), ui);
                c.show_mut(Some("hue"), ui) || a || b
            }
            Expression::r#if(a, b, c) => {
                let a = a.show_mut(Some("if"), ui);
                let b = b.show_mut(Some("then"), ui);
                c.show_mut(Some("else"), ui) || a || b
            }
            Expression::f32_slider(x) => Slider::new("x").full_width().ui(x, 0.0..=1.0, ui),
            _ => false,
        }
    }
}

impl DataFramed for PainterAction {
    fn show_name_mut(&mut self, ui: &mut Ui) -> DataFrameResponse {
        if Selector::from_mut(self, ui) {
            DataFrameResponse::Changed
        } else {
            DataFrameResponse::None
        }
    }
    fn has_header(&self) -> bool {
        false
    }
    fn has_body(&self) -> bool {
        match self {
            PainterAction::paint => false,
            PainterAction::circle(..)
            | PainterAction::rectangle(..)
            | PainterAction::curve { .. }
            | PainterAction::text(..)
            | PainterAction::hollow(..)
            | PainterAction::translate(..)
            | PainterAction::rotate(..)
            | PainterAction::scale_mesh(..)
            | PainterAction::scale_rect(..)
            | PainterAction::color(..)
            | PainterAction::feathering(..)
            | PainterAction::alpha(..)
            | PainterAction::repeat(..)
            | PainterAction::list(..) => true,
        }
    }
    fn show_header(&self, _: &Context, _: &mut Ui) {}
    fn show_header_mut(&mut self, _: &mut Ui) -> bool {
        false
    }
    fn show_body(&self, context: &Context, ui: &mut Ui) {
        match self {
            PainterAction::paint => {}
            PainterAction::circle(x)
            | PainterAction::rectangle(x)
            | PainterAction::text(x)
            | PainterAction::hollow(x)
            | PainterAction::translate(x)
            | PainterAction::rotate(x)
            | PainterAction::scale_mesh(x)
            | PainterAction::scale_rect(x)
            | PainterAction::color(x)
            | PainterAction::feathering(x)
            | PainterAction::alpha(x) => x.show(Some("x"), context, ui),
            PainterAction::curve {
                thickness,
                curvature,
            } => {
                thickness.show(Some("thickness"), context, ui);
                curvature.show(Some("curvature"), context, ui);
            }
            PainterAction::repeat(x, painter_action) => {
                x.show(Some("cnt"), context, ui);
                painter_action.show(Some("action"), context, ui);
            }
            PainterAction::list(vec) => {}
        }
    }
    fn show_body_mut(&mut self, ui: &mut Ui) -> bool {
        match self {
            PainterAction::paint => false,
            PainterAction::circle(x)
            | PainterAction::rectangle(x)
            | PainterAction::text(x)
            | PainterAction::hollow(x)
            | PainterAction::translate(x)
            | PainterAction::rotate(x)
            | PainterAction::scale_mesh(x)
            | PainterAction::scale_rect(x)
            | PainterAction::color(x)
            | PainterAction::feathering(x)
            | PainterAction::alpha(x) => x.show_mut(Some("x"), ui),
            PainterAction::repeat(x, painter_action) => {
                let x = x.show_mut(Some("cnt"), ui);
                painter_action.show_mut(Some("action"), ui) || x
            }
            PainterAction::curve {
                thickness,
                curvature,
            } => {
                let thickness = thickness.show_mut(Some("thickness"), ui);
                curvature.show_mut(Some("curvature"), ui) || thickness
            }
            PainterAction::list(vec) => false,
        }
    }
}
impl DataFramed for Action {
    fn default_open(&self) -> bool {
        match self {
            Action::apply_status | Action::use_ability => true,
            _ => false,
        }
    }
    fn show_name_mut(&mut self, ui: &mut Ui) -> DataFrameResponse {
        if Selector::from_mut(self, ui) {
            DataFrameResponse::Changed
        } else {
            DataFrameResponse::None
        }
    }
    fn has_header(&self) -> bool {
        match self {
            Action::apply_status | Action::use_ability => true,
            _ => false,
        }
    }
    fn has_body(&self) -> bool {
        match self {
            Action::noop
            | Action::deal_damage
            | Action::heal_damage
            | Action::apply_status
            | Action::use_ability => false,
            Action::debug(..)
            | Action::set_value(..)
            | Action::add_value(..)
            | Action::subtract_value(..)
            | Action::add_target(..)
            | Action::repeat(..) => true,
        }
    }
    fn show_header(&self, context: &Context, ui: &mut Ui) {
        match self {
            Action::use_ability => {
                if let Some(ability) = context
                    .get_owner()
                    .ok()
                    .and_then(|entity| context.find_parent_component::<AbilityMagic>(entity))
                {
                    ability.view(ViewContext::compact().hide_buttons(), context, ui);
                }
            }
            Action::apply_status => {
                if let Some(status) = context
                    .get_owner()
                    .ok()
                    .and_then(|entity| context.find_parent_component::<StatusMagic>(entity))
                {
                    status.view(ViewContext::compact().hide_buttons(), context, ui);
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
            Action::deal_damage
            | Action::heal_damage
            | Action::apply_status
            | Action::use_ability
            | Action::noop => {}
            Action::debug(x)
            | Action::set_value(x)
            | Action::add_value(x)
            | Action::subtract_value(x)
            | Action::add_target(x) => {
                x.show(Some("x"), context, ui);
            }
            Action::repeat(x, vec) => {
                x.show(Some("x"), context, ui);
            }
        }
    }

    fn show_body_mut(&mut self, ui: &mut Ui) -> bool {
        match self {
            Action::deal_damage
            | Action::heal_damage
            | Action::apply_status
            | Action::use_ability
            | Action::noop => false,
            Action::debug(x)
            | Action::set_value(x)
            | Action::add_value(x)
            | Action::subtract_value(x)
            | Action::add_target(x) => x.show_mut(Some("x"), ui),
            Action::repeat(x, vec) => x.show_mut(Some("x"), ui),
        }
    }
}
impl DataFramed for Trigger {
    fn show_name_mut(&mut self, ui: &mut Ui) -> DataFrameResponse {
        if Selector::from_mut(self, ui) {
            DataFrameResponse::Changed
        } else {
            DataFrameResponse::None
        }
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
