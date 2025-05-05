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
            let r = self.view_title(vctx, context, ui);
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
            if r.clicked() {
                vr.title_clicked = true;
            }
        });
        if let Some(f) = Self::fn_view_data_mut() {
            f(self, vctx, context, ui);
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

pub trait ViewFns: Sized + Clone + StringData {
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
    fn fn_view_data_mut() -> Option<fn(&mut Self, ViewContextNew, &Context, &mut Ui) -> bool> {
        None
    }
    fn fn_view_type() -> Option<fn(&Self, ViewContextNew, &Context, &mut Ui)> {
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
    fn fn_view_context_menu_mut(
    ) -> Option<fn(&mut Self, ViewContextNew, &Context, &mut Ui) -> ViewResponseNew> {
        if Self::fn_wrap().is_some() {
            Some(|s, vctx, context, ui| {
                let mut vr = ViewResponseNew::default();
                if let Some(f) = Self::fn_replace_options() {
                    ui.menu_button("replace", |ui| {
                        ScrollArea::vertical()
                            .min_scrolled_height(150.0)
                            .show(ui, |ui| {
                                for mut opt in f() {
                                    if opt.title_cstr(vctx, context).button(ui).clicked() {
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
                    });
                }
                if let Some(f) = Self::fn_wrap() {
                    if ui.button("wrap").clicked() {
                        *s = f(s.clone());
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
}
impl ViewFns for f32 {
    fn title_cstr(&self, vctx: ViewContextNew, context: &Context) -> Cstr {
        type_name_of_val_short(self).cstr()
    }
    fn fn_view_data() -> Option<fn(&Self, ViewContextNew, &Context, &mut Ui)> {
        Some(|s, _, context, ui| {
            s.cstr().label(ui);
        })
    }
    fn fn_view_data_mut() -> Option<fn(&mut Self, ViewContextNew, &Context, &mut Ui) -> bool> {
        Some(|s, _, context, ui| s.show_mut(context, ui))
    }
}

fn view_children_recursive_mut<T: Inject + Injector<I>, I: ViewChildren + Inject>(
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
fn view_children_recursive<T: Inject + Injector<I>, I: ViewChildren + Inject>(
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
        view_children_recursive::<_, Self>(self, vctx, context, ui)
    }
    fn view_children_mut(
        &mut self,
        vctx: ViewContextNew,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponseNew {
        view_children_recursive_mut::<_, Self>(self, vctx, context, ui)
    }
}
