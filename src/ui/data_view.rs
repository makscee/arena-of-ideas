use std::any::{type_name, type_name_of_val};

use serde::de::DeserializeOwned;

use super::*;

#[derive(Clone, Copy)]
pub struct ViewContext {
    id: Id,
    collapsed: bool,
    parent_rect: Rect,
    can_delete: bool,
    non_interactible: bool,
}

#[derive(Clone, Copy, Default)]
pub struct ViewResponse {
    pub hovered: bool,
    pub changed: bool,
    pub delete_me: bool,
}

impl ViewResponse {
    pub fn merge(&mut self, other: Self) {
        *self = Self {
            hovered: self.hovered || other.hovered,
            changed: self.changed || other.changed,
            delete_me: self.delete_me || other.delete_me,
        }
    }
    pub fn take_delete_me(&mut self) -> bool {
        let v = self.delete_me;
        self.delete_me = false;
        v
    }
}

impl ViewContext {
    pub fn new(ui: &mut Ui) -> Self {
        Self {
            id: ui.id(),
            parent_rect: ui.min_rect(),
            collapsed: false,
            can_delete: false,
            non_interactible: false,
        }
    }
    pub fn with_id(mut self, h: impl Hash) -> Self {
        self.id = self.id.with(h);
        self
    }
    pub fn collapsed(mut self, value: bool) -> Self {
        self.collapsed = value;
        self
    }
    pub fn can_delete(mut self, value: bool) -> Self {
        self.can_delete = value;
        self
    }
    pub fn non_interactible(mut self, value: bool) -> Self {
        self.non_interactible = value;
        self
    }
    pub fn merge_state(mut self, view: &impl DataView, ui: &mut Ui) -> Self {
        self.id = self.id.with(type_name_of_val(view));
        if let Some(state) = ui.data(|r| r.get_temp::<ViewContext>(self.id)) {
            self.collapsed = state.collapsed;
        }
        self
    }
    pub fn save_state(self, ui: &mut Ui) {
        ui.data_mut(|w| w.insert_temp(self.id, self));
    }
}

fn show_parent_line(parent: Rect, child: Rect, hovered: bool, ui: &mut Ui) {
    if (child.left() - parent.right()).abs() < 30.0 {
        ui.painter().line_segment(
            [parent.right_center(), child.left_center()],
            if hovered {
                ui.visuals().widgets.hovered.fg_stroke
            } else {
                ui.visuals().weak_text_color().stroke()
            },
        );
    }
}

pub trait DataView: Sized + Clone + Default + StringData + ToCstr + Debug {
    fn wrap(self) -> Option<Self> {
        None
    }
    fn replace_options() -> Vec<Self> {
        default()
    }
    fn move_inner(&mut self, _source: &mut Self) {}
    fn merge_state<'a>(
        &self,
        view_ctx: ViewContext,
        context: &Context<'a>,
        ui: &mut Ui,
    ) -> (ViewContext, Context<'a>) {
        (view_ctx.merge_state(self, ui), context.clone())
    }
    fn view(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        let mut view_resp = ViewResponse::default();
        let (view_ctx, context) = self.merge_state(view_ctx, context, ui);
        let parent_rect = view_ctx.parent_rect;
        let mut self_rect = Rect::ZERO;
        let mut show = |s: &Self, mut view_ctx: ViewContext, ui: &mut Ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let title_response = s.show_title(view_ctx, &context, ui);
                    self_rect = title_response.rect;
                    show_parent_line(
                        view_ctx.parent_rect,
                        self_rect,
                        title_response.hovered(),
                        ui,
                    );
                    view_ctx.parent_rect = self_rect;
                    if title_response.hovered() {
                        view_resp.hovered = true;
                    }
                    title_response.bar_menu(|ui| {
                        s.context_menu(view_ctx, &context, ui);
                    });
                    if ui
                        .ctx()
                        .rect_contains_pointer(ui.layer_id(), title_response.rect)
                    {
                        let size = 8.0;
                        if RectButton::new_rect(Rect::from_center_size(
                            title_response.rect.right_center() - egui::vec2(size, 0.0),
                            egui::Vec2::splat(size),
                        ))
                        .color(ui.visuals().weak_text_color())
                        .ui(ui, |color, rect, _, ui| {
                            ui.painter().line(
                                [
                                    rect.left_bottom(),
                                    rect.left_top(),
                                    rect.right_center(),
                                    rect.left_bottom(),
                                ]
                                .into(),
                                color.stroke(),
                            );
                        })
                        .clicked()
                        {
                            view_ctx.collapsed(true).save_state(ui);
                        }
                    }
                    s.show_value(view_ctx, &context, ui);
                });
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    view_resp.merge(s.view_children(view_ctx, &context, ui));
                });
            });
        };
        if view_ctx.collapsed {
            let r = self.show_collapsed(view_ctx, &context, ui);
            show_parent_line(view_ctx.parent_rect, r.rect, false, ui);
            if r.on_hover_ui(|ui| show(self, view_ctx.collapsed(false), ui))
                .clicked()
            {
                view_ctx.collapsed(false).save_state(ui);
            }
        } else {
            show(self, view_ctx, ui);
        }
        if view_resp.hovered {
            show_parent_line(parent_rect, self_rect, true, ui);
        }
        view_resp
    }
    fn view_mut(&mut self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        let mut view_resp = ViewResponse::default();
        let (view_ctx, context) = self.merge_state(view_ctx, context, ui);
        let parent_rect = view_ctx.parent_rect;
        let mut self_rect = Rect::ZERO;
        let mut show = |s: &mut Self, mut view_ctx: ViewContext, ui: &mut Ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let title_response = s.show_title(view_ctx, &context, ui);
                    self_rect = title_response.rect;
                    show_parent_line(
                        view_ctx.parent_rect,
                        self_rect,
                        title_response.hovered(),
                        ui,
                    );
                    view_ctx.parent_rect = self_rect;
                    if title_response.hovered() {
                        view_resp.hovered = true;
                    }
                    title_response.bar_menu(|ui| {
                        s.context_menu(view_ctx, &context, ui);
                        view_resp.merge(s.context_menu_mut(view_ctx, &context, ui));
                    });
                    if ui
                        .ctx()
                        .rect_contains_pointer(ui.layer_id(), title_response.rect)
                    {
                        let size = 8.0;
                        if RectButton::new_rect(Rect::from_center_size(
                            title_response.rect.right_center() - egui::vec2(size, 0.0),
                            egui::Vec2::splat(size),
                        ))
                        .color(ui.visuals().weak_text_color())
                        .ui(ui, |color, rect, _, ui| {
                            ui.painter().line(
                                [
                                    rect.left_bottom(),
                                    rect.left_top(),
                                    rect.right_center(),
                                    rect.left_bottom(),
                                ]
                                .into(),
                                color.stroke(),
                            );
                        })
                        .clicked()
                        {
                            view_ctx.collapsed(true).save_state(ui);
                        }
                    }
                    view_resp.changed |= s.show_value_mut(view_ctx, &context, ui);
                });
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    view_resp.merge(s.view_children_mut(view_ctx, &context, ui));
                });
            });
        };
        if view_ctx.collapsed {
            let r = self.show_collapsed(view_ctx, &context, ui);
            show_parent_line(view_ctx.parent_rect, r.rect, false, ui);
            if r.on_hover_ui(|ui| show(self, view_ctx.collapsed(false), ui))
                .clicked()
            {
                view_ctx.collapsed(false).save_state(ui);
            }
        } else {
            show(self, view_ctx, ui);
        }
        if view_resp.hovered {
            show_parent_line(parent_rect, self_rect, true, ui);
        }
        view_resp
    }
    fn show_collapsed(&self, _view_ctx: ViewContext, _context: &Context, ui: &mut Ui) -> Response {
        "([tw ...])".cstr().button(ui)
    }
    fn show_value(&self, _view_ctx: ViewContext, _context: &Context, _ui: &mut Ui) {}
    fn show_value_mut(&mut self, _view_ctx: ViewContext, _context: &Context, _ui: &mut Ui) -> bool {
        false
    }
    fn view_children(
        &self,
        _view_ctx: ViewContext,
        _context: &Context,
        _ui: &mut Ui,
    ) -> ViewResponse {
        default()
    }
    fn view_children_mut(
        &mut self,
        _view_ctx: ViewContext,
        _context: &Context,
        _ui: &mut Ui,
    ) -> ViewResponse {
        default()
    }
    fn title_cstr(&self, _view_ctx: ViewContext, _context: &Context) -> Cstr {
        self.cstr()
    }
    fn show_title(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> Response {
        if view_ctx.non_interactible {
            self.title_cstr(view_ctx, context).label(ui)
        } else {
            self.title_cstr(view_ctx, context).button(ui)
        }
    }
    fn context_menu(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) {
        if view_ctx.collapsed {
            if ui.button("expand").clicked() {
                view_ctx.collapsed(false).save_state(ui);
                ui.close_menu();
            }
        } else {
            if ui.button("collapse").clicked() {
                view_ctx.collapsed(true).save_state(ui);
                ui.close_menu();
            }
        }
        if ui.button("copy").clicked() {
            self.copy();
            ui.close_menu();
        }
        self.context_menu_extra(view_ctx, context, ui);
    }
    fn context_menu_extra(&self, _view_ctx: ViewContext, _context: &Context, _ui: &mut Ui) {}
    fn context_menu_extra_mut(
        &mut self,
        _view_ctx: ViewContext,
        _context: &Context,
        _ui: &mut Ui,
    ) -> ViewResponse {
        ViewResponse::default()
    }
    fn context_menu_mut(
        &mut self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        let mut view_resp = ViewResponse::default();
        let options = Self::replace_options();
        let lookup_id = Id::new("lookup text");
        if !options.is_empty() {
            if ui
                .menu_button("replace", |ui| {
                    let lookup =
                        if let Some(mut lookup) = ui.data(|r| r.get_temp::<String>(lookup_id)) {
                            let resp = Input::new("")
                                .desired_width(ui.available_width())
                                .ui_string(&mut lookup, ui);
                            if resp.changed() {
                                ui.data_mut(|w| w.insert_temp(lookup_id, lookup.clone()));
                            }
                            resp.request_focus();
                            lookup
                        } else {
                            String::new()
                        };
                    ScrollArea::vertical()
                        .min_scrolled_height(200.0)
                        .show(ui, |ui| {
                            'o: for mut opt in options {
                                let text = opt.cstr();
                                if !lookup.is_empty() {
                                    let text = text.get_text().to_lowercase();
                                    let mut text = text.chars();
                                    'c: for c in lookup.chars() {
                                        while let Some(text_c) = text.next() {
                                            if text_c == c {
                                                continue 'c;
                                            }
                                        }
                                        continue 'o;
                                    }
                                }
                                let resp = opt.cstr().button(ui);
                                if resp.clicked() || resp.gained_focus() {
                                    self.move_inner(&mut opt);
                                    mem::swap(self, &mut opt);
                                    view_resp.changed = true;
                                }
                            }
                        });
                })
                .response
                .clicked()
            {
                ui.data_mut(|w| w.insert_temp(lookup_id, String::new()));
            };
        }
        if Self::wrap(default()).is_some() {
            if ui.button("wrap").clicked() {
                view_resp.changed = true;
                *self = self.clone().wrap().unwrap();
            }
        }
        if let Some(data) = clipboard_get() {
            if ui
                .menu_button("paste", |ui| {
                    let mut d = Self::default();
                    if let Err(e) = d.inject_data(&data) {
                        ui.set_max_width(300.0);
                        Label::new(&data).wrap().ui(ui);
                        e.cstr().label_w(ui);
                    } else {
                        if d.view_mut(view_ctx, context, ui).changed {
                            clipboard_set(d.get_data());
                        }
                    }
                })
                .response
                .clicked()
            {
                view_resp.changed = true;
                self.paste();
            }
        }
        view_resp.merge(self.context_menu_extra_mut(view_ctx, context, ui));
        if view_ctx.can_delete && "[red delete]".cstr().button(ui).clicked() {
            view_resp.changed = true;
            view_resp.delete_me = true;
        }
        if view_resp.changed {
            ui.close_menu();
            ui.data_mut(|w| w.remove_temp::<String>(lookup_id));
        }
        view_resp
    }
    fn copy(&self) {
        clipboard_set(self.get_data());
    }
    fn paste(&mut self) {
        if let Some(data) = clipboard_get() {
            self.inject_data(&data).notify_op();
        } else {
            "Clipboard is empty".notify_error_op();
        }
    }
}

fn view_children_mut<T: DataView + Inject + Injector<I>, I: DataView + Inject>(
    s: &mut T,
    view_ctx: ViewContext,
    context: &Context,
    ui: &mut Ui,
) -> ViewResponse {
    let mut view_resp = ViewResponse::default();
    let names = <T as Injector<I>>::get_inner_names(s);
    for (i, e) in <T as Injector<I>>::get_inner_mut(s).into_iter().enumerate() {
        ui.horizontal(|ui| {
            if let Some(name) = names.get(i) {
                format!("[tw {name}]").cstr().label(ui);
            }
            view_resp.merge(e.view_mut(view_ctx.with_id(i), context, ui));
        });
    }
    view_resp
}
fn view_children<T: DataView + Inject + Injector<I>, I: DataView + Inject>(
    s: &T,
    view_ctx: ViewContext,
    context: &Context,
    ui: &mut Ui,
) -> ViewResponse {
    let mut view_resp = ViewResponse::default();
    let names = <T as Injector<I>>::get_inner_names(s);
    for (i, e) in <T as Injector<I>>::get_inner(s).into_iter().enumerate() {
        ui.horizontal(|ui| {
            if let Some(name) = names.get(i) {
                format!("[tw {name}]").cstr().label(ui);
            }
            view_resp.merge(e.view(view_ctx.with_id(i), context, ui));
        });
    }
    view_resp
}
fn show_children_mut<T: DataView + Inject + Injector<I>, I: Show>(
    s: &mut T,
    context: &Context,
    ui: &mut Ui,
) -> bool {
    let mut changed = false;
    let names = <T as Injector<I>>::get_inner_names(s);
    for (i, e) in <T as Injector<I>>::get_inner_mut(s).into_iter().enumerate() {
        ui.horizontal(|ui| {
            if let Some(name) = names.get(i) {
                format!("[tw {name}]").cstr().label(ui);
            }
            changed |= e.show_mut(context, ui);
        });
    }
    changed
}
fn show_children<T: DataView + Inject + Injector<I>, I: Show>(
    s: &T,
    context: &Context,
    ui: &mut Ui,
) {
    let names = <T as Injector<I>>::get_inner_names(s);
    for (i, e) in <T as Injector<I>>::get_inner(s).into_iter().enumerate() {
        ui.horizontal(|ui| {
            if let Some(name) = names.get(i) {
                format!("[tw {name}]").cstr().label(ui);
            }
            e.show(context, ui);
        });
    }
}

impl DataView for VarName {
    fn replace_options() -> Vec<Self> {
        Self::iter().collect_vec()
    }
}

impl DataView for Trigger {
    fn replace_options() -> Vec<Self> {
        Self::iter().collect()
    }
    fn view_children(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        match self {
            Trigger::BattleStart | Trigger::TurnEnd | Trigger::BeforeDeath => {
                ViewResponse::default()
            }
            Trigger::ChangeStat(var) => var.view(view_ctx, context, ui),
        }
    }
    fn view_children_mut(
        &mut self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        match self {
            Trigger::BattleStart | Trigger::TurnEnd | Trigger::BeforeDeath => {
                ViewResponse::default()
            }
            Trigger::ChangeStat(var) => var.view_mut(view_ctx, context, ui),
        }
    }
}

impl DataView for Expression {
    fn replace_options() -> Vec<Self> {
        Self::iter().collect_vec()
    }
    fn wrap(self) -> Option<Self> {
        Some(Self::abs(Box::new(self)))
    }
    fn move_inner(&mut self, source: &mut Self) {
        <Expression as Injector<Expression>>::inject_inner(self, source);
        <Expression as Injector<f32>>::inject_inner(self, source);
        <Expression as Injector<VarName>>::inject_inner(self, source);
    }
    fn view_children_mut(
        &mut self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        let mut view_resp = ViewResponse::default();
        view_resp.merge(view_children_mut::<_, Self>(self, view_ctx, context, ui));
        view_resp.changed |= show_children_mut::<_, f32>(self, context, ui);
        view_resp.changed |= show_children_mut::<_, VarName>(self, context, ui);
        view_resp.changed |= show_children_mut::<_, HexColor>(self, context, ui);
        view_resp
    }
    fn view_children(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        show_children::<_, f32>(self, context, ui);
        show_children::<_, HexColor>(self, context, ui);
        view_children::<_, Self>(self, view_ctx, context, ui)
    }
    fn show_value(&self, _: ViewContext, context: &Context, ui: &mut Ui) {
        match self.get_value(context) {
            Ok(v) => v.cstr_expanded(),
            Err(e) => e.cstr(),
        }
        .label(ui);
    }
}
fn material_view(m: &Material, context: &Context, ui: &mut Ui) {
    let size_id = ui.id().with("view size");
    let mut size = ui.ctx().data_mut(|w| *w.get_temp_mut_or(size_id, 60.0));
    if DragValue::new(&mut size).ui(ui).changed() {
        ui.ctx().data_mut(|w| w.insert_temp(size_id, size));
    }
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
    RepresentationPlugin::paint_rect(rect, context, m, ui).ui(ui);
    ui.painter().rect_stroke(
        rect,
        0,
        Stroke::new(1.0, tokens_global().subtle_borders_and_separators()),
        egui::StrokeKind::Middle,
    );
}
impl DataView for Material {
    fn view_children_mut(
        &mut self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        ui.vertical(|ui| {
            material_view(self, context, ui);
            self.0.view_mut(view_ctx, context, ui)
        })
        .inner
    }
    fn view_children(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        ui.vertical(|ui| {
            material_view(self, context, ui);
            self.0.view(view_ctx, context, ui)
        })
        .inner
    }
}

impl DataView for PainterAction {
    fn replace_options() -> Vec<Self> {
        Self::iter().collect_vec()
    }
    fn wrap(self) -> Option<Self> {
        Some(Self::list([Box::new(self)].to_vec()))
    }
    fn move_inner(&mut self, source: &mut Self) {
        <Self as Injector<Self>>::inject_inner(self, source);
        <Self as Injector<Expression>>::inject_inner(self, source);
    }
    fn view_children_mut(
        &mut self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        let mut view_resp = ViewResponse::default();
        match self {
            PainterAction::list(vec) => {
                return vec.view_mut(view_ctx, context, ui);
            }
            _ => {}
        }
        view_resp.merge(view_children_mut::<_, Self>(self, view_ctx, context, ui));
        view_resp.merge(view_children_mut::<_, Expression>(
            self, view_ctx, context, ui,
        ));
        view_resp
    }
    fn view_children(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        let mut view_resp = ViewResponse::default();
        match self {
            PainterAction::list(vec) => {
                return vec.view(view_ctx, context, ui);
            }
            _ => {}
        }
        view_resp.merge(view_children::<_, Self>(self, view_ctx, context, ui));
        view_resp.merge(view_children::<_, Expression>(self, view_ctx, context, ui));
        view_resp
    }
}

impl DataView for Action {
    fn wrap(self) -> Option<Self> {
        Some(Self::repeat(
            Box::new(Expression::i32(1)),
            [Box::new(self)].to_vec(),
        ))
    }
    fn replace_options() -> Vec<Self> {
        Self::iter().collect()
    }
    fn view_children(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        let mut view_resp = ViewResponse::default();
        view_resp.merge(view_children::<_, Self>(self, view_ctx, context, ui));
        view_resp.merge(view_children::<_, Expression>(self, view_ctx, context, ui));
        view_resp
    }
    fn view_children_mut(
        &mut self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        let mut view_resp = ViewResponse::default();
        view_resp.merge(view_children_mut::<_, Self>(self, view_ctx, context, ui));
        view_resp.merge(view_children_mut::<_, Expression>(
            self, view_ctx, context, ui,
        ));
        view_resp
    }
    fn title_cstr(&self, _: ViewContext, context: &Context) -> Cstr {
        match self {
            Action::use_ability => {
                let mut r = self.cstr();
                if let Ok(ability) = context.get_string(VarName::ability_name) {
                    if let Ok(color) = context.get_color(VarName::color) {
                        r += " ";
                        r += &ability.cstr_cs(color, CstrStyle::Bold);
                    }
                }
                r
            }
            Action::apply_status => {
                let mut r = self.cstr();
                if let Ok(status) = context.get_string(VarName::status_name) {
                    if let Ok(color) = context.get_color(VarName::color) {
                        r += " ";
                        r += &status.cstr_cs(color, CstrStyle::Bold);
                    }
                }
                r
            }
            _ => self.cstr(),
        }
    }
}

impl DataView for Reaction {
    fn view_children(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        let mut view_resp = ViewResponse::default();
        view_resp.merge(self.trigger.view(view_ctx, context, ui));
        view_resp.merge(self.actions.0.view(view_ctx, context, ui));
        view_resp
    }
    fn view_children_mut(
        &mut self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        let mut view_resp = self.trigger.view_mut(view_ctx, context, ui);
        view_resp.merge(self.actions.0.view_mut(view_ctx, context, ui));
        view_resp
    }
}

impl<T> DataView for Vec<T>
where
    T: DataView
        + Sized
        + Clone
        + Default
        + StringData
        + ToCstr
        + Hash
        + Serialize
        + DeserializeOwned,
{
    fn title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        let name = type_name::<T>();
        let name = name.split("::").last().unwrap_or_default();
        format!("[tw {name}] ({})", self.len())
    }
    fn show_collapsed(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.add_enabled_ui(false, |ui| {
                self.show_title(view_ctx, context, ui);
            });
            "([tw ...])".cstr().button(ui)
        })
        .inner
    }
    fn show_value(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) {
        for (i, v) in self.into_iter().enumerate() {
            v.view(view_ctx.with_id(i), context, ui);
        }
    }
    fn show_value_mut(&mut self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        let mut to_remove = None;
        let mut swap = None;
        let len = self.len();
        let size = egui::Vec2::splat(8.0);
        for (i, v) in self.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                if RectButton::new_size(size)
                    .enabled(i > 0)
                    .ui(ui, |color, rect, _, ui| {
                        ui.painter().line(
                            [
                                rect.left_bottom(),
                                rect.center_top(),
                                rect.right_bottom(),
                                rect.left_bottom(),
                            ]
                            .into(),
                            color.stroke(),
                        );
                    })
                    .clicked()
                {
                    swap = Some((i, i - 1));
                }
                if RectButton::new_size(size)
                    .enabled(i + 1 < len)
                    .ui(ui, |color, rect, _, ui| {
                        ui.painter().line(
                            [
                                rect.left_top(),
                                rect.right_top(),
                                rect.center_bottom(),
                                rect.left_top(),
                            ]
                            .into(),
                            color.stroke(),
                        );
                    })
                    .clicked()
                {
                    swap = Some((i, i + 1));
                }
                if RectButton::new_size(size)
                    .color(RED)
                    .ui(ui, |color, rect, _, ui| {
                        ui.painter().rect_filled(
                            rect.shrink2(egui::vec2(0.0, size.y * 0.3)),
                            0,
                            color,
                        );
                    })
                    .clicked()
                {
                    to_remove = Some(i);
                }
                ui.vertical(|ui| {
                    changed |= v.view_mut(view_ctx.with_id(i), context, ui).changed;
                });
            });
        }

        if let Some(i) = to_remove {
            self.remove(i);
            changed = true;
        }
        if let Some((a, b)) = swap {
            self.swap(a, b);
            changed = true;
        }
        if "[b +]".cstr().button(ui).clicked() {
            self.push(default());
            changed = true;
        }
        changed
    }
}
impl<T> DataView for Box<T>
where
    T: DataView
        + Sized
        + Clone
        + Default
        + StringData
        + ToCstr
        + Hash
        + Serialize
        + DeserializeOwned,
{
    fn wrap(self) -> Option<Self> {
        T::wrap(*self).map(|v| Box::new(v))
    }
    fn replace_options() -> Vec<Self> {
        T::replace_options()
            .into_iter()
            .map(|v| Box::new(v))
            .collect()
    }
    fn move_inner(&mut self, source: &mut Self) {
        self.as_mut().move_inner(source.as_mut());
    }
    fn view(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        self.as_ref().view(view_ctx, context, ui)
    }
    fn view_mut(&mut self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        self.as_mut().view_mut(view_ctx, context, ui)
    }
    fn show_value(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) {
        self.as_ref().show_value(view_ctx, context, ui);
    }
    fn show_value_mut(&mut self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> bool {
        self.as_mut().show_value_mut(view_ctx, context, ui)
    }
    fn view_children(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        self.as_ref().view_children(view_ctx, context, ui)
    }
    fn view_children_mut(
        &mut self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        self.as_mut().view_children_mut(view_ctx, context, ui)
    }
    fn title_cstr(&self, view_ctx: ViewContext, context: &Context) -> Cstr {
        self.as_ref().title_cstr(view_ctx, context)
    }
    fn show_title(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> Response {
        T::show_title(self.as_ref(), view_ctx, context, ui)
    }
    fn context_menu(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) {
        self.as_ref().context_menu(view_ctx, context, ui);
    }
    fn context_menu_mut(
        &mut self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        self.as_mut().context_menu_mut(view_ctx, context, ui)
    }
    fn copy(&self) {
        self.as_ref().copy();
    }
    fn paste(&mut self) {
        self.as_mut().paste();
    }
}
