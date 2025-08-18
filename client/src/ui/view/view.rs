use std::any::type_name_of_val;

use serde::de::DeserializeOwned;

use super::*;

#[derive(Copy, Clone)]
pub struct ViewContext {
    pub id: Id,
    pub collapsed: bool,
    pub selected: bool,
    pub non_interactible: bool,
    pub one_line: bool,
    pub separate_contex_menu_btn: bool,
    pub can_delete: bool,
    pub parent_rect: Option<Rect>,
    pub link_rating: Option<(bool, u64)>,
}

impl ViewContext {
    pub fn new(ui: &mut Ui) -> Self {
        Self {
            id: ui.id(),

            collapsed: false,
            selected: false,
            non_interactible: false,
            one_line: false,
            separate_contex_menu_btn: false,
            can_delete: false,

            parent_rect: None,
            link_rating: None,
        }
    }
    pub fn merge_state(mut self, view: &impl ViewFns, ui: &mut Ui) -> Self {
        self.id = self.id.with(type_name_of_val(view));
        if let Some(state) = ui.data(|r| r.get_temp::<ViewContext>(self.id)) {
            self.collapsed = state.collapsed;
        }
        self
    }
    pub fn save_state(self, ui: &mut Ui) {
        ui.data_mut(|w| w.insert_temp(self.id, self));
    }
    pub fn with_id(mut self, h: impl Hash) -> Self {
        self.id = self.id.with(h);
        self
    }
    pub fn collapsed(mut self, value: bool) -> Self {
        self.collapsed = value;
        self
    }
    pub fn non_interactible(mut self, value: bool) -> Self {
        self.non_interactible = value;
        self
    }
    pub fn one_line(mut self, value: bool) -> Self {
        self.one_line = value;
        self
    }
    pub fn selected(mut self, value: bool) -> Self {
        self.selected = value;
        self
    }
    pub fn context_btn(mut self) -> Self {
        self.separate_contex_menu_btn = true;
        self
    }
    pub fn with_parent_rect(mut self, rect: Rect) -> Self {
        self.parent_rect = Some(rect);
        self
    }
    pub fn can_delete(mut self) -> Self {
        self.can_delete = true;
        self
    }
    pub fn link_rating(mut self, parent: bool, id: u64) -> Self {
        self.link_rating = Some((parent, id));
        self
    }
}

#[derive(Copy, Clone, Default)]
pub struct ViewResponse {
    pub changed: bool,
    pub title_clicked: bool,
    pub delete_me: bool,
}

impl ViewResponse {
    pub fn merge(&mut self, other: Self) {
        self.changed |= other.changed;
        self.delete_me |= other.delete_me;
    }
    pub fn take_delete_me(&mut self) -> bool {
        let v = self.delete_me;
        self.delete_me = false;
        v
    }
}

fn circle_btn(r: &Response, ui: &mut Ui) -> Response {
    let rect = Rect::from_min_max(r.rect.left_bottom(), r.rect.left_bottom() + 10.0.v2());
    RectButton::new_rect(rect)
        .color(ui.visuals().weak_text_color())
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
}
fn collapse_btn(r: &Response, ui: &mut Ui) -> Response {
    let rect = Rect::from_min_max(
        r.rect.right_top() - egui::vec2(LINE_HEIGHT, 0.0),
        r.rect.right_top() + egui::vec2(0.0, LINE_HEIGHT),
    );
    RectButton::new_rect(rect).ui(ui, |color, mut rect, _, ui| {
        rect.min.y += LINE_HEIGHT * 0.6;
        ui.painter().rect_filled(rect.shrink(2.0), 0, color);
    })
}

pub trait View: Sized + ViewFns {
    fn view_mut(&mut self, vctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        let mut vr = ViewResponse::default();
        let vctx = vctx.merge_state(self, ui);
        ui.horizontal(|ui| {
            let mut r = self.view_title(vctx, context, ui);
            if ui.rect_contains_pointer(r.rect) && collapse_btn(&r, ui).clicked() {
                vctx.collapsed(!vctx.collapsed).save_state(ui);
            }
            if Self::fn_view_context_menu().is_some() || Self::fn_view_context_menu_mut().is_some()
            {
                if vctx.separate_contex_menu_btn {
                    circle_btn(&r, ui)
                } else {
                    r.clone()
                }
                .bar_menu(|ui| {
                    if let Some(f) = Self::fn_view_context_menu_mut() {
                        vr.merge(f(self, vctx, context, ui));
                    }
                    if let Some(f) = Self::fn_view_context_menu() {
                        f(self, vctx, context, ui);
                    }
                });
            }
            if let Some(f) = Self::fn_view_type() {
                f(self, vctx, context, ui);
            }
            if let Some(f) = Self::fn_view_value() {
                r = r.on_hover_ui(|ui| {
                    context.with_layers_ref(default(), |context| {
                        f(self, vctx, context, ui);
                    });
                });
            }
            if r.clicked() {
                vr.title_clicked = true;
            }
        });
        if vctx.collapsed {
            return vr;
        }
        if let Some(f) = Self::fn_view_data_mut() {
            vr.merge(f(self, vctx, context, ui));
        } else if let Some(f) = Self::fn_view_data() {
            f(self, vctx, context, ui);
        }
        vr
    }
    fn view(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        let mut vr = ViewResponse::default();
        let vctx = vctx.merge_state(self, ui);
        ui.horizontal(|ui| {
            let mut r = self.view_title(vctx, context, ui);
            if ui.rect_contains_pointer(r.rect) && collapse_btn(&r, ui).clicked() {
                vctx.collapsed(!vctx.collapsed).save_state(ui);
            }
            if Self::fn_view_context_menu().is_some() {
                if vctx.separate_contex_menu_btn {
                    circle_btn(&r, ui)
                } else {
                    r.clone()
                }
                .bar_menu(|ui| {
                    if let Some(f) = Self::fn_view_context_menu() {
                        f(self, vctx, context, ui);
                    }
                });
            }
            if let Some(f) = Self::fn_view_type() {
                f(self, vctx, context, ui);
            }
            if let Some(f) = Self::fn_view_value() {
                r = r.on_hover_ui(|ui| {
                    context.with_layers_ref(default(), |context| {
                        f(self, vctx, context, ui);
                    });
                });
            }
            if r.clicked() {
                vr.title_clicked = true;
            }
        });
        if vctx.collapsed {
            return vr;
        }
        if let Some(f) = Self::fn_view_data() {
            f(self, vctx, context, ui);
        }
        vr
    }
}

impl<T> View for T where T: ViewFns {}

pub trait ViewChildren: View {
    fn view_with_children_mut(
        &mut self,
        vctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        ui.horizontal(|ui| {
            let mut vr = self.view_mut(vctx, context, ui);
            let vctx = vctx.merge_state(self, ui);
            if vctx.collapsed {
                if "[tw ...]".cstr().button(ui).clicked() {
                    vctx.collapsed(false).save_state(ui);
                }
                return vr;
            }
            if let Some(rect) = vctx.parent_rect {
                const OFFSET: egui::Vec2 = egui::vec2(0.0, LINE_HEIGHT * 0.5);
                ui.painter().line_segment(
                    [rect.right_top() + OFFSET, ui.min_rect().left_top() + OFFSET],
                    ui.visuals().weak_text_color().stroke(),
                );
            }
            let vctx = vctx.with_parent_rect(ui.min_rect()).can_delete();
            ui.vertical(|ui| {
                vr.merge(self.view_children_mut(vctx, context, ui));
            });
            vr
        })
        .inner
    }
    fn view_with_children(
        &self,
        vctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        ui.horizontal(|ui| {
            let r = self.view(vctx, context, ui);
            ui.vertical(|ui| {
                self.view_children(vctx, context, ui);
            });
            r
        })
        .inner
    }
    fn view_children(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse;
    fn view_children_mut(
        &mut self,
        vctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse;
}

impl<T> ViewChildren for Box<T>
where
    T: ViewChildren + ViewFns + StringData + Serialize + DeserializeOwned,
{
    fn view_children(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        self.as_ref().view_children(vctx, context, ui)
    }
    fn view_children_mut(
        &mut self,
        vctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        self.as_mut().view_children_mut(vctx, context, ui)
    }
}

impl<T> ViewChildren for Vec<T>
where
    T: ViewChildren + StringData + Serialize + DeserializeOwned,
{
    fn view_children(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        for (i, v) in self.into_iter().enumerate() {
            v.view_with_children(vctx.with_id(i), context, ui);
        }
        default()
    }
    fn view_children_mut(
        &mut self,
        mut vctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        let mut vr = ViewResponse::default();
        let mut to_remove = None;
        let mut swap = None;
        let len = self.len();
        let size = egui::Vec2::splat(8.0);
        vctx.parent_rect = None;
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
                ui.vertical(|ui| {
                    ui.push_id(i, |ui| {
                        vr.merge(v.view_with_children_mut(vctx.with_id(i), context, ui));
                    });
                    if vr.take_delete_me() {
                        to_remove = Some(i);
                    }
                });
            });
        }

        if let Some(i) = to_remove {
            self.remove(i);
            vr.changed = true;
        }
        if let Some((a, b)) = swap {
            self.swap(a, b);
            vr.changed = true;
        }
        if "[b +]".cstr().button(ui).clicked() {
            self.push(default());
            vr.changed = true;
        }
        vr
    }
}

pub trait ViewFns: Sized + Clone + StringData + Default {
    fn title_cstr(&self, vctx: ViewContext, context: &Context) -> Cstr;
    fn view_title(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) -> Response {
        if vctx.non_interactible {
            self.title_cstr(vctx, context).label_w(ui)
        } else {
            self.title_cstr(vctx, context)
                .as_button()
                .min_width(50.0)
                .ui(ui)
        }
    }
    fn fn_view_data() -> Option<fn(&Self, ViewContext, &Context, &mut Ui)> {
        None
    }
    fn fn_view_data_mut() -> Option<fn(&mut Self, ViewContext, &Context, &mut Ui) -> ViewResponse> {
        None
    }
    fn fn_view_type() -> Option<fn(&Self, ViewContext, &Context, &mut Ui)> {
        None
    }
    fn fn_view_value() -> Option<fn(&Self, ViewContext, &mut Context, &mut Ui)> {
        None
    }
    fn fn_wrap() -> Option<fn(Self) -> Self> {
        None
    }
    fn fn_replace_options() -> Option<fn() -> Vec<Self>> {
        None
    }
    fn fn_move_inner() -> Option<fn(&mut Self, &mut Self)> {
        None
    }
    fn fn_view_context_menu() -> Option<fn(&Self, ViewContext, &Context, &mut Ui)> {
        Some(|s, _, _, ui| {
            if ui.button("copy").clicked() {
                clipboard_set(s.get_data());
                ui.close_menu();
            }
        })
    }
    fn fn_paste_preview(
        &mut self,
        vctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        self.view_mut(vctx, context, ui)
    }
    fn fn_view_context_menu_extra_mut()
    -> Option<fn(&mut Self, ViewContext, &Context, &mut Ui) -> ViewResponse> {
        None
    }
    fn fn_view_context_menu_mut()
    -> Option<fn(&mut Self, ViewContext, &Context, &mut Ui) -> ViewResponse> {
        Some(|s, vctx, context, ui| {
            let mut vr = ViewResponse::default();
            if let Some(f) = Self::fn_replace_options() {
                let lookup_id = Id::new("lookup text");
                if ui
                    .menu_button("replace", |ui| {
                        let lookup = if let Some(mut lookup) =
                            ui.data(|r| r.get_temp::<String>(lookup_id))
                        {
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
                                let opts = if lookup.is_empty() {
                                    f()
                                } else {
                                    f().into_iter()
                                        .filter(|o| {
                                            let text = o.title_cstr(vctx, context).get_text();
                                            let mut text = text.chars();
                                            'c: for c in lookup.chars() {
                                                while let Some(text_c) = text.next() {
                                                    if text_c == c {
                                                        continue 'c;
                                                    }
                                                }
                                                return false;
                                            }
                                            true
                                        })
                                        .sorted_by_cached_key(|o| {
                                            let text = o.title_cstr(vctx, context).get_text();
                                            !text.starts_with(&lookup)
                                        })
                                        .collect()
                                };
                                for mut opt in opts {
                                    let resp = opt.title_cstr(vctx, context).button(ui);
                                    if resp.clicked() || resp.gained_focus() {
                                        if let Some(f) = Self::fn_move_inner() {
                                            f(s, &mut opt);
                                            mem::swap(s, &mut opt);
                                        } else {
                                            *s = opt;
                                        }
                                        vr.changed = true;
                                        ui.close_menu();
                                    }
                                }
                            });
                    })
                    .response
                    .clicked()
                    || vr.changed
                {
                    ui.data_mut(|w| w.insert_temp(lookup_id, String::new()));
                }
            }
            if let Some(f) = Self::fn_wrap() {
                if ui.button("wrap").clicked() {
                    *s = f(s.clone());
                    vr.changed = true;
                    ui.close_menu();
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
                            if Self::fn_paste_preview(&mut d, vctx, context, ui).changed {
                                clipboard_set(d.get_data());
                            }
                        }
                    })
                    .response
                    .clicked()
                {
                    s.inject_data(&data).notify_op();
                    vr.changed = true;
                    ui.close_menu();
                }
            }
            if vctx.can_delete {
                if "[red delete]".cstr().button(ui).clicked() {
                    vr.delete_me = true;
                    ui.close_menu();
                }
            }
            if let Some(f) = Self::fn_view_context_menu_extra_mut() {
                vr.merge(f(s, vctx, context, ui));
            }
            vr
        })
    }
}

impl<T> ViewFns for Vec<T>
where
    T: ViewFns + StringData + Serialize + DeserializeOwned,
{
    fn title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        format!("{} ({})", type_name_short::<T>(), self.len())
    }
}

impl<T> ViewFns for Box<T>
where
    T: ViewFns + StringData + Serialize + DeserializeOwned,
{
    fn title_cstr(&self, vctx: ViewContext, context: &Context) -> Cstr {
        self.as_ref().title_cstr(vctx, context)
    }
    fn fn_replace_options() -> Option<fn() -> Vec<Self>> {
        if T::fn_replace_options().is_some() {
            Some(|| {
                T::fn_replace_options().unwrap()()
                    .into_iter()
                    .map(|o| Box::new(o))
                    .collect()
            })
        } else {
            None
        }
    }
    fn fn_move_inner() -> Option<fn(&mut Self, &mut Self)> {
        if T::fn_move_inner().is_some() {
            Some(|s, source| T::fn_move_inner().unwrap()(s.as_mut(), source.as_mut()))
        } else {
            None
        }
    }
    fn view_title(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) -> Response {
        self.as_ref().view_title(vctx, context, ui)
    }
    fn fn_view_data() -> Option<fn(&Self, ViewContext, &Context, &mut Ui)> {
        if T::fn_view_data().is_some() {
            Some(|s, vctx, context, ui| T::fn_view_data().unwrap()(s.as_ref(), vctx, context, ui))
        } else {
            None
        }
    }
    fn fn_view_data_mut() -> Option<fn(&mut Self, ViewContext, &Context, &mut Ui) -> ViewResponse> {
        if T::fn_view_data_mut().is_some() {
            Some(|s, vctx, context, ui| {
                T::fn_view_data_mut().unwrap()(s.as_mut(), vctx, context, ui)
            })
        } else {
            None
        }
    }
    fn fn_view_type() -> Option<fn(&Self, ViewContext, &Context, &mut Ui)> {
        if T::fn_view_type().is_some() {
            Some(|s, vctx, context, ui| T::fn_view_type().unwrap()(s.as_ref(), vctx, context, ui))
        } else {
            None
        }
    }
    fn fn_view_value() -> Option<fn(&Self, ViewContext, &mut Context, &mut Ui)> {
        if T::fn_view_value().is_some() {
            Some(|s, vctx, context, ui| T::fn_view_value().unwrap()(s, vctx, context, ui))
        } else {
            None
        }
    }
    fn fn_wrap() -> Option<fn(Self) -> Self> {
        if T::fn_wrap().is_some() {
            Some(|s| Box::new(T::fn_wrap().unwrap()(*s)))
        } else {
            None
        }
    }
    fn fn_view_context_menu() -> Option<fn(&Self, ViewContext, &Context, &mut Ui)> {
        if T::fn_view_context_menu().is_some() {
            Some(|s, vctx, context, ui| T::fn_view_context_menu().unwrap()(s, vctx, context, ui))
        } else {
            None
        }
    }
    fn fn_paste_preview(
        &mut self,
        vctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        self.as_mut().fn_paste_preview(vctx, context, ui)
    }
    fn fn_view_context_menu_extra_mut()
    -> Option<fn(&mut Self, ViewContext, &Context, &mut Ui) -> ViewResponse> {
        if T::fn_view_context_menu_extra_mut().is_some() {
            Some(|s, vctx, context, ui| {
                T::fn_view_context_menu_extra_mut().unwrap()(s, vctx, context, ui)
            })
        } else {
            None
        }
    }
    fn fn_view_context_menu_mut()
    -> Option<fn(&mut Self, ViewContext, &Context, &mut Ui) -> ViewResponse> {
        if T::fn_view_context_menu_mut().is_some() {
            Some(|s, vctx, context, ui| {
                T::fn_view_context_menu_mut().unwrap()(s, vctx, context, ui)
            })
        } else {
            None
        }
    }
}
