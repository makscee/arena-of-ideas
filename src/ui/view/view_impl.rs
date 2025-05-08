use super::*;

fn view_children_recursive_mut<T: Inject + Injector<I>, I: ViewChildren>(
    s: &mut T,
    vctx: ViewContext,
    context: &Context,
    ui: &mut Ui,
) -> ViewResponse {
    let mut vr = ViewResponse::default();
    for (i, e) in <T as Injector<I>>::get_inner_mut(s).into_iter().enumerate() {
        ui.horizontal(|ui| {
            vr.merge(e.view_with_children_mut(vctx.with_id(i), context, ui));
        });
    }
    vr
}
fn view_children_recursive<T: Inject + Injector<I>, I: ViewChildren>(
    s: &T,
    vctx: ViewContext,
    context: &Context,
    ui: &mut Ui,
) -> ViewResponse {
    let mut vr = ViewResponse::default();
    for (i, e) in <T as Injector<I>>::get_inner(s).into_iter().enumerate() {
        ui.horizontal(|ui| {
            vr.merge(e.view(vctx.with_id(i), context, ui));
        });
    }
    vr
}
fn view_children_mut<T: Inject + Injector<I>, I: ViewFns>(
    s: &mut T,
    vctx: ViewContext,
    context: &Context,
    ui: &mut Ui,
) -> ViewResponse {
    let mut vr = ViewResponse::default();
    for (i, e) in <T as Injector<I>>::get_inner_mut(s).into_iter().enumerate() {
        ui.horizontal(|ui| {
            vr.merge(e.view_mut(vctx.with_id(i), context, ui));
        });
    }
    vr
}
fn view_children<T: Inject + Injector<I>, I: ViewFns>(
    s: &T,
    vctx: ViewContext,
    context: &Context,
    ui: &mut Ui,
) -> ViewResponse {
    let mut vr = ViewResponse::default();
    for (i, e) in <T as Injector<I>>::get_inner(s).into_iter().enumerate() {
        ui.horizontal(|ui| {
            vr.merge(e.view(vctx.with_id(i), context, ui));
        });
    }
    vr
}

impl ViewFns for Expression {
    fn title_cstr(&self, vctx: ViewContext, context: &Context) -> Cstr {
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
            <Expression as Injector<i32>>::inject_inner(s, source);
            <Expression as Injector<VarName>>::inject_inner(s, source);
            <Expression as Injector<HexColor>>::inject_inner(s, source);
        })
    }
    fn fn_view_value() -> Option<fn(&Self, ViewContext, &Context, &mut Ui)> {
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
        vctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        self.view_with_children_mut(vctx, context, ui)
    }
}
impl ViewChildren for Expression {
    fn view_children(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        let mut vr = view_children_recursive::<_, Self>(self, vctx, context, ui);
        vr.merge(view_children::<_, f32>(self, vctx, context, ui));
        vr.merge(view_children::<_, i32>(self, vctx, context, ui));
        vr.merge(view_children::<_, HexColor>(self, vctx, context, ui));
        vr
    }
    fn view_children_mut(
        &mut self,
        vctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        let mut vr = view_children_recursive_mut::<_, Self>(self, vctx, context, ui);
        vr.merge(view_children_mut::<_, f32>(self, vctx, context, ui));
        vr.merge(view_children_mut::<_, i32>(self, vctx, context, ui));
        vr.merge(view_children_mut::<_, HexColor>(self, vctx, context, ui));
        vr
    }
}

impl ViewFns for Action {
    fn fn_wrap() -> Option<fn(Self) -> Self> {
        Some(|s| Self::repeat(Box::new(Expression::i32(1)), [Box::new(s)].to_vec()))
    }
    fn fn_replace_options() -> Option<fn() -> Vec<Self>> {
        Some(|| Self::iter().collect())
    }
    fn fn_move_inner() -> Option<fn(&mut Self, &mut Self)> {
        Some(|s, source| {
            <Self as Injector<Self>>::inject_inner(s, source);
            <Self as Injector<Expression>>::inject_inner(s, source);
        })
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
impl ViewChildren for Action {
    fn view_children(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        let mut vr = view_children_recursive::<_, Self>(self, vctx, context, ui);
        vr.merge(view_children_recursive::<_, Expression>(
            self, vctx, context, ui,
        ));
        vr
    }
    fn view_children_mut(
        &mut self,
        vctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        let mut vr = view_children_recursive_mut::<_, Self>(self, vctx, context, ui);
        vr.merge(view_children_recursive_mut::<_, Expression>(
            self, vctx, context, ui,
        ));
        vr
    }
}

impl ViewFns for Trigger {
    fn fn_replace_options() -> Option<fn() -> Vec<Self>> {
        Some(|| Self::iter().collect())
    }
    fn title_cstr(&self, vctx: ViewContext, context: &Context) -> Cstr {
        self.cstr()
    }
}
impl ViewChildren for Trigger {
    fn view_children(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        match self {
            Trigger::BattleStart | Trigger::TurnEnd | Trigger::BeforeDeath => {
                ViewResponse::default()
            }
            Trigger::ChangeStat(var) => var.view(vctx, context, ui),
        }
    }
    fn view_children_mut(
        &mut self,
        vctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        match self {
            Trigger::BattleStart | Trigger::TurnEnd | Trigger::BeforeDeath => {
                ViewResponse::default()
            }
            Trigger::ChangeStat(var) => var.view_mut(vctx, context, ui),
        }
    }
}

impl ViewFns for Reaction {
    fn title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        self.cstr()
    }
}
impl ViewChildren for Reaction {
    fn view_children(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        let mut vr = ViewResponse::default();
        vr.merge(self.trigger.view_with_children(vctx, context, ui));
        vr.merge(self.actions.view_with_children(vctx, context, ui));
        vr
    }
    fn view_children_mut(
        &mut self,
        vctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        let mut vr = ViewResponse::default();
        vr.merge(self.trigger.view_with_children_mut(vctx, context, ui));
        vr.merge(self.actions.view_with_children_mut(vctx, context, ui));
        vr
    }
}

impl ViewFns for Material {
    fn title_cstr(&self, vctx: ViewContext, context: &Context) -> Cstr {
        self.cstr()
    }
    fn fn_view_data() -> Option<fn(&Self, ViewContext, &Context, &mut Ui)> {
        Some(|s, vctx, context, ui| {
            let size_id = ui.id().with("view size");
            let mut size = ui.ctx().data_mut(|w| *w.get_temp_mut_or(size_id, 60.0));
            if DragValue::new(&mut size).ui(ui).changed() {
                ui.ctx().data_mut(|w| w.insert_temp(size_id, size));
            }
            let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
            RepresentationPlugin::paint_rect(rect, context, s, ui).ui(ui);
            ui.painter().rect_stroke(
                rect,
                0,
                Stroke::new(1.0, tokens_global().subtle_borders_and_separators()),
                egui::StrokeKind::Middle,
            );
        })
    }
}
impl ViewChildren for Material {
    fn view_children(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        self.0.view_with_children(vctx, context, ui)
    }
    fn view_children_mut(
        &mut self,
        vctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        self.0.view_with_children_mut(vctx, context, ui)
    }
}

impl ViewFns for PainterAction {
    fn title_cstr(&self, vctx: ViewContext, context: &Context) -> Cstr {
        self.cstr()
    }
    fn fn_wrap() -> Option<fn(Self) -> Self> {
        Some(|s| Self::list([Box::new(s)].to_vec()))
    }
    fn fn_replace_options() -> Option<fn() -> Vec<Self>> {
        Some(|| Self::iter().collect())
    }
    fn fn_move_inner() -> Option<fn(&mut Self, &mut Self)> {
        Some(|s, source| {
            <Self as Injector<Self>>::inject_inner(s, source);
            <Self as Injector<Expression>>::inject_inner(s, source);
        })
    }
}
impl ViewChildren for PainterAction {
    fn view_children(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        let mut vr = ViewResponse::default();
        match self {
            PainterAction::list(vec) => {
                return vec.view_with_children(vctx, context, ui);
            }
            _ => {}
        }
        vr.merge(view_children_recursive::<_, Self>(self, vctx, context, ui));
        vr.merge(view_children_recursive::<_, Expression>(
            self, vctx, context, ui,
        ));
        vr
    }
    fn view_children_mut(
        &mut self,
        vctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        let mut vr = ViewResponse::default();
        match self {
            PainterAction::list(vec) => {
                return vec.view_with_children_mut(vctx, context, ui);
            }
            _ => {}
        }
        vr.merge(view_children_recursive_mut::<_, Self>(
            self, vctx, context, ui,
        ));
        vr.merge(view_children_recursive_mut::<_, Expression>(
            self, vctx, context, ui,
        ));
        vr
    }
}

impl ViewFns for VarName {
    fn title_cstr(&self, vctx: ViewContext, context: &Context) -> Cstr {
        self.cstr()
    }
    fn fn_replace_options() -> Option<fn() -> Vec<Self>> {
        Some(|| Self::iter().collect())
    }
}

impl ViewFns for f32 {
    fn title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        type_name_of_val_short(self).cstr()
    }
    fn fn_view_data() -> Option<fn(&Self, ViewContext, &Context, &mut Ui)> {
        Some(|s, _, _, ui| {
            s.cstr().label(ui);
        })
    }
    fn fn_view_data_mut() -> Option<fn(&mut Self, ViewContext, &Context, &mut Ui) -> ViewResponse> {
        Some(|s, _, context, ui| {
            let mut vr = ViewResponse::default();
            vr.changed = s.show_mut(context, ui);
            vr
        })
    }
}

impl ViewFns for i32 {
    fn title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        type_name_of_val_short(self).cstr()
    }
    fn fn_view_data() -> Option<fn(&Self, ViewContext, &Context, &mut Ui)> {
        Some(|s, _, _, ui| {
            s.cstr().label(ui);
        })
    }
    fn fn_view_data_mut() -> Option<fn(&mut Self, ViewContext, &Context, &mut Ui) -> ViewResponse> {
        Some(|s, _, context, ui| {
            let mut vr = ViewResponse::default();
            vr.changed = s.show_mut(context, ui);
            vr
        })
    }
}

impl ViewFns for HexColor {
    fn title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        type_name_of_val_short(self).cstr()
    }
    fn fn_view_data() -> Option<fn(&Self, ViewContext, &Context, &mut Ui)> {
        Some(|s, _, _, ui| {
            s.cstr().label(ui);
        })
    }
    fn fn_view_data_mut() -> Option<fn(&mut Self, ViewContext, &Context, &mut Ui) -> ViewResponse> {
        Some(|s, _, context, ui| {
            let mut vr = ViewResponse::default();
            vr.changed = s.show_mut(context, ui);
            vr
        })
    }
}
