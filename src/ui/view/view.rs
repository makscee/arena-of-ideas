use super::*;

#[derive(Copy, Clone, Default)]
pub struct ViewContextNew {
    pub non_interactible: bool,
    pub one_line: bool,
    pub separate_contex_menu_btn: bool,
}

pub trait View: Sized + ViewFns {
    fn view_new(&self, view_ctx: ViewContextNew, context: &Context, ui: &mut Ui) -> Response {
        let r = ui
            .horizontal(|ui| {
                let r = self.view_title(view_ctx, context, ui);
                if let Some(f) = Self::fn_view_context_menu() {
                    if view_ctx.separate_contex_menu_btn {
                        let rect = Rect::from_min_max(
                            r.rect.left_bottom(),
                            r.rect.left_bottom() + 10.0.v2(),
                        );
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
                    } else {
                        r.clone()
                    }
                    .bar_menu(|ui| {
                        f(self, view_ctx, context, ui);
                    });
                }
                if let Some(f) = Self::fn_view_type() {
                    f(self, view_ctx, context, ui);
                }
                r
            })
            .inner;
        if let Some(f) = Self::fn_view_data() {
            f(self, view_ctx, context, ui);
        }
        r
    }
}

impl<T> View for T where T: ViewFns {}

pub trait ViewChildren: View {
    fn view_with_children(
        &self,
        view_ctx: ViewContextNew,
        context: &Context,
        ui: &mut Ui,
    ) -> Response {
        ui.horizontal(|ui| {
            let r = self.view_new(view_ctx, context, ui);
            ui.vertical(|ui| {
                self.view_children(view_ctx, context, ui);
            });
            r
        })
        .inner
    }
    fn view_children(&self, view_ctx: ViewContextNew, context: &Context, ui: &mut Ui);
}

pub trait ViewFns: Sized + Clone + StringData {
    fn view_title(&self, view_ctx: ViewContextNew, context: &Context, ui: &mut Ui) -> Response;
    fn fn_view_data() -> Option<fn(&Self, ViewContextNew, &Context, &mut Ui)> {
        None
    }
    fn fn_view_data_mut() -> Option<fn(&mut Self, ViewContext, &Context, &mut Ui) -> bool> {
        None
    }
    fn fn_view_type() -> Option<fn(&Self, ViewContextNew, &Context, &mut Ui)> {
        None
    }
    fn fn_wrap() -> Option<fn(Self) -> Self> {
        None
    }
    fn fn_view_context_menu() -> Option<fn(&Self, ViewContextNew, &Context, &mut Ui)> {
        Some(|s, view_ctx, context, ui| {
            if ui.button("copy").clicked() {
                clipboard_set(s.get_data());
                ui.close_menu();
            }
        })
    }
    fn fn_view_context_menu_mut(
        &mut self,
    ) -> Option<fn(&mut Self, ViewContextNew, &Context, &mut Ui)> {
        if Self::fn_wrap().is_some() {
            Some(|s, view_ctx, context, ui| {
                if let Some(f) = Self::fn_wrap() {
                    if ui.button("wrap").clicked() {
                        *s = f(s.clone());
                    }
                }
            })
        } else {
            None
        }
    }
}

impl ViewFns for Expression {
    fn view_title(&self, view_ctx: ViewContextNew, context: &Context, ui: &mut Ui) -> Response {
        self.cstr().button(ui)
    }
}
impl ViewFns for f32 {
    fn view_title(&self, view_ctx: ViewContextNew, context: &Context, ui: &mut Ui) -> Response {
        type_name_of_val_short(self).cstr().label(ui)
    }
    fn fn_view_data() -> Option<fn(&Self, ViewContextNew, &Context, &mut Ui)> {
        Some(|s, _, context, ui| {
            s.cstr().label(ui);
        })
    }
    fn fn_view_data_mut() -> Option<fn(&mut Self, ViewContext, &Context, &mut Ui) -> bool> {
        Some(|s, _, context, ui| s.show_mut(context, ui))
    }
}

impl ViewChildren for Expression {
    fn view_children(&self, view_ctx: ViewContextNew, context: &Context, ui: &mut Ui) {
        match self {
            Expression::one => todo!(),
            Expression::zero => todo!(),
            Expression::gt => todo!(),
            Expression::owner => todo!(),
            Expression::target => todo!(),
            Expression::unit_size => todo!(),
            Expression::pi => todo!(),
            Expression::pi2 => todo!(),
            Expression::all_units => todo!(),
            Expression::all_enemy_units => todo!(),
            Expression::all_ally_units => todo!(),
            Expression::all_other_ally_units => todo!(),
            Expression::adjacent_ally_units => todo!(),
            Expression::adjacent_back => todo!(),
            Expression::adjacent_front => todo!(),
            Expression::var(var_name) => todo!(),
            Expression::var_sum(var_name) => todo!(),
            Expression::value(var_value) => todo!(),
            Expression::string(_) => todo!(),
            Expression::f32(v) => {
                v.view_new(view_ctx, context, ui);
            }
            Expression::f32_slider(_) => todo!(),
            Expression::i32(_) => todo!(),
            Expression::bool(_) => todo!(),
            Expression::vec2(_, _) => todo!(),
            Expression::color(hex_color) => todo!(),
            Expression::state_var(a, var_name) => todo!(),
            Expression::sin(a)
            | Expression::cos(a)
            | Expression::even(a)
            | Expression::abs(a)
            | Expression::floor(a)
            | Expression::ceil(a)
            | Expression::fract(a)
            | Expression::sqr(a)
            | Expression::unit_vec(a)
            | Expression::rand(a)
            | Expression::random_unit(a)
            | Expression::to_f32(a) => {
                a.view_with_children(view_ctx, context, ui);
            }
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
            | Expression::less_then(a, b)
            | Expression::fallback(a, b) => {
                a.view_with_children(view_ctx, context, ui);
                b.view_with_children(view_ctx, context, ui);
            }
            Expression::r#if(a, b, c) | Expression::oklch(a, b, c) => {
                a.view_with_children(view_ctx, context, ui);
                b.view_with_children(view_ctx, context, ui);
                c.view_with_children(view_ctx, context, ui);
            }
        }
    }
}
