use super::*;

#[derive(Copy, Clone)]
pub struct ViewContextNew {
    pub id: Id,
    pub non_interactible: bool,
    pub one_line: bool,
    pub separate_contex_menu_btn: bool,
}

impl ViewContextNew {
    pub fn new(ui: &mut Ui) -> Self {
        Self {
            id: ui.id(),
            non_interactible: false,
            one_line: false,
            separate_contex_menu_btn: false,
        }
    }
    pub fn with_id(mut self, h: impl Hash) -> Self {
        self.id = self.id.with(h);
        self
    }
    pub fn context_btn(mut self) -> Self {
        self.separate_contex_menu_btn = true;
        self
    }
}

#[derive(Copy, Clone, Default)]
pub struct ViewResponseNew {
    pub changed: bool,
    pub title_clicked: bool,
}

impl ViewResponseNew {
    fn merge(&mut self, other: Self) {
        self.changed |= other.changed;
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

pub trait View: Sized + ViewFns {
    fn view_mut_new(
        &mut self,
        vctx: ViewContextNew,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponseNew {
        let mut vr = ViewResponseNew::default();
        ui.horizontal(|ui| {
            let mut r = self.view_title(vctx, context, ui);
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
                    f(self, vctx, context, ui);
                });
            }
            if r.clicked() {
                vr.title_clicked = true;
            }
        });
        if let Some(f) = Self::fn_view_data_mut() {
            vr.merge(f(self, vctx, context, ui));
        } else if let Some(f) = Self::fn_view_data() {
            f(self, vctx, context, ui);
        }
        vr
    }
    fn view_new(&self, vctx: ViewContextNew, context: &Context, ui: &mut Ui) -> ViewResponseNew {
        let mut response = ViewResponseNew::default();
        ui.horizontal(|ui| {
            let r = self.view_title(vctx, context, ui);
            if let Some(f) = Self::fn_view_context_menu() {
                if vctx.separate_contex_menu_btn {
                    circle_btn(&r, ui)
                } else {
                    r.clone()
                }
                .bar_menu(|ui| {
                    f(self, vctx, context, ui);
                });
            }
            if let Some(f) = Self::fn_view_type() {
                f(self, vctx, context, ui);
            }
            response.title_clicked |= r.clicked();
        });
        if let Some(f) = Self::fn_view_data() {
            f(self, vctx, context, ui);
        }
        response
    }
}

impl<T> View for T where T: ViewFns {}

pub trait ViewChildren: View {
    fn view_with_children_mut(
        &mut self,
        vctx: ViewContextNew,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponseNew {
        ui.horizontal(|ui| {
            let mut vr = self.view_mut_new(vctx, context, ui);
            ui.vertical(|ui| {
                vr.merge(self.view_children_mut(vctx, context, ui));
            });
            vr
        })
        .inner
    }
    fn view_with_children(
        &self,
        vctx: ViewContextNew,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponseNew {
        ui.horizontal(|ui| {
            let r = self.view_new(vctx, context, ui);
            ui.vertical(|ui| {
                self.view_children(vctx, context, ui);
            });
            r
        })
        .inner
    }
    fn view_children(
        &self,
        vctx: ViewContextNew,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponseNew;
    fn view_children_mut(
        &mut self,
        vctx: ViewContextNew,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponseNew;
}

pub trait ViewFns: Sized + Clone + StringData + Default {
    fn title_cstr(&self, vctx: ViewContextNew, context: &Context) -> Cstr;
    fn view_title(&self, vctx: ViewContextNew, context: &Context, ui: &mut Ui) -> Response {
        if vctx.non_interactible {
            self.title_cstr(vctx, context).label(ui)
        } else {
            self.title_cstr(vctx, context).button(ui)
        }
    }
    fn fn_view_data() -> Option<fn(&Self, ViewContextNew, &Context, &mut Ui)> {
        None
    }
    fn fn_view_data_mut(
    ) -> Option<fn(&mut Self, ViewContextNew, &Context, &mut Ui) -> ViewResponseNew> {
        None
    }
    fn fn_view_type() -> Option<fn(&Self, ViewContextNew, &Context, &mut Ui)> {
        None
    }
    fn fn_view_value() -> Option<fn(&Self, ViewContextNew, &Context, &mut Ui)> {
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
    fn fn_view_context_menu() -> Option<fn(&Self, ViewContextNew, &Context, &mut Ui)> {
        Some(|s, vctx, context, ui| {
            if ui.button("copy").clicked() {
                clipboard_set(s.get_data());
                ui.close_menu();
            }
        })
    }
    fn fn_paste_preview(
        &mut self,
        vctx: ViewContextNew,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponseNew {
        self.view_mut_new(vctx, context, ui)
    }
    fn fn_view_context_menu_mut(
    ) -> Option<fn(&mut Self, ViewContextNew, &Context, &mut Ui) -> ViewResponseNew> {
        if Self::fn_wrap().is_some() {
            Some(|s, vctx, context, ui| {
                let mut vr = ViewResponseNew::default();
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
                                        let text = opt.title_cstr(vctx, context);
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
                vr
            })
        } else {
            None
        }
    }
}

impl ViewFns for Expression {
    fn title_cstr(&self, vctx: ViewContextNew, context: &Context) -> Cstr {
        self.cstr()
    }
    fn fn_wrap() -> Option<fn(Self) -> Self> {
        Some(|s| Self::abs(Box::new(s)))
    }
    fn fn_replace_options() -> Option<fn() -> Vec<Self>> {
        Some(|| Self::iter().collect())
    }
    fn fn_move_inner() -> Option<fn(&mut Self, &mut Self)> {
        Some(|s, source| {
            <Expression as Injector<Expression>>::inject_inner(s, source);
            <Expression as Injector<f32>>::inject_inner(s, source);
            <Expression as Injector<VarName>>::inject_inner(s, source);
        })
    }
    fn fn_view_value() -> Option<fn(&Self, ViewContextNew, &Context, &mut Ui)> {
        Some(|s, _, context, ui| match s.get_value(context) {
            Ok(v) => {
                ui.horizontal(|ui| {
                    v.as_ref()
                        .cstr_cs(ui.visuals().weak_text_color(), CstrStyle::Small)
                        .label(ui);
                    v.show(context, ui);
                });
            }
            Err(e) => e.show(context, ui),
        })
    }
    fn fn_paste_preview(
        &mut self,
        vctx: ViewContextNew,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponseNew {
        self.view_with_children_mut(vctx, context, ui)
    }
}
impl ViewFns for f32 {
    fn title_cstr(&self, _: ViewContextNew, _: &Context) -> Cstr {
        type_name_of_val_short(self).cstr()
    }
    fn fn_view_data() -> Option<fn(&Self, ViewContextNew, &Context, &mut Ui)> {
        Some(|s, _, _, ui| {
            s.cstr().label(ui);
        })
    }
    fn fn_view_data_mut(
    ) -> Option<fn(&mut Self, ViewContextNew, &Context, &mut Ui) -> ViewResponseNew> {
        Some(|s, _, context, ui| {
            let mut vr = ViewResponseNew::default();
            vr.changed = s.show_mut(context, ui);
            vr
        })
    }
}

fn view_children_recursive_mut<T: Inject + Injector<I>, I: ViewChildren>(
    s: &mut T,
    vctx: ViewContextNew,
    context: &Context,
    ui: &mut Ui,
) -> ViewResponseNew {
    let mut view_resp = ViewResponseNew::default();
    for (i, e) in <T as Injector<I>>::get_inner_mut(s).into_iter().enumerate() {
        ui.horizontal(|ui| {
            view_resp.merge(e.view_with_children_mut(vctx.with_id(i), context, ui));
        });
    }
    view_resp
}
fn view_children_recursive<T: Inject + Injector<I>, I: ViewChildren>(
    s: &T,
    vctx: ViewContextNew,
    context: &Context,
    ui: &mut Ui,
) -> ViewResponseNew {
    let mut view_resp = ViewResponseNew::default();
    for (i, e) in <T as Injector<I>>::get_inner(s).into_iter().enumerate() {
        ui.horizontal(|ui| {
            view_resp.merge(e.view_new(vctx.with_id(i), context, ui));
        });
    }
    view_resp
}
fn view_children_mut<T: Inject + Injector<I>, I: ViewFns>(
    s: &mut T,
    vctx: ViewContextNew,
    context: &Context,
    ui: &mut Ui,
) -> ViewResponseNew {
    let mut view_resp = ViewResponseNew::default();
    for (i, e) in <T as Injector<I>>::get_inner_mut(s).into_iter().enumerate() {
        ui.horizontal(|ui| {
            view_resp.merge(e.view_mut_new(vctx.with_id(i), context, ui));
        });
    }
    view_resp
}
fn view_children<T: Inject + Injector<I>, I: ViewFns>(
    s: &T,
    vctx: ViewContextNew,
    context: &Context,
    ui: &mut Ui,
) -> ViewResponseNew {
    let mut view_resp = ViewResponseNew::default();
    for (i, e) in <T as Injector<I>>::get_inner(s).into_iter().enumerate() {
        ui.horizontal(|ui| {
            view_resp.merge(e.view_new(vctx.with_id(i), context, ui));
        });
    }
    view_resp
}

impl ViewChildren for Expression {
    fn view_children(
        &self,
        vctx: ViewContextNew,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponseNew {
        let mut vr = view_children_recursive::<_, Self>(self, vctx, context, ui);
        vr.merge(view_children::<_, f32>(self, vctx, context, ui));
        vr
    }
    fn view_children_mut(
        &mut self,
        vctx: ViewContextNew,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponseNew {
        let mut vr = view_children_recursive_mut::<_, Self>(self, vctx, context, ui);
        vr.merge(view_children_mut::<_, f32>(self, vctx, context, ui));
        vr
    }
}
