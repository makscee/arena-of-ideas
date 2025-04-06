use super::*;

#[derive(Clone, Copy)]
pub struct DataViewContext {
    id: Id,
    collapsed: bool,
}

impl DataViewContext {
    pub fn new(ui: &mut Ui) -> Self {
        Self {
            id: ui.id(),
            collapsed: false,
        }
    }
    fn with_id(mut self, h: impl Hash) -> Self {
        self.id = self.id.with(h);
        self
    }
    fn collapsed(mut self, value: bool) -> Self {
        self.collapsed = value;
        self
    }
    fn merge_state(mut self, view: &impl DataView, ui: &mut Ui) -> Self {
        self.id = self.id.with(view);
        if let Some(state) = ui.data(|r| r.get_temp::<DataViewContext>(self.id)) {
            self.collapsed = state.collapsed;
        }
        self
    }
    fn save_state(self, ui: &mut Ui) {
        ui.data_mut(|w| w.insert_temp(self.id, self));
    }
}

pub trait DataView: Sized + Clone + Default + StringData + ToCstr + Hash {
    fn wrap(value: Self) -> Option<Self> {
        None
    }
    fn replace_options() -> Vec<Self> {
        default()
    }
    fn move_inner(&mut self, source: &mut Self) {}
    fn view(&self, view_ctx: DataViewContext, context: &Context, ui: &mut Ui) {}
    fn view_mut(&mut self, view_ctx: DataViewContext, context: &Context, ui: &mut Ui) -> bool {
        let view_ctx = view_ctx.merge_state(self, ui);
        let mut changed = false;
        let mut show = |view_ctx, ui: &mut Ui| {
            ui.horizontal(|ui| {
                Self::show_title(self.cstr().widget(1.0, ui.style()), ui, |ui| {
                    self.show_value(context, ui);
                    changed |= self.context_menu_mut(ui);
                    self.context_menu(view_ctx, ui);
                });
                changed |= self.show_body_mut(view_ctx, context, ui);
            });
        };
        if view_ctx.collapsed {
            let b = "[tw (...)]".cstr().button(ui);
            if b.clicked() {
                view_ctx.collapsed(false).save_state(ui);
            }
            if b.hovered() {
                cursor_window(ui.ctx(), |ui| {
                    Frame::new()
                        .fill(ui.visuals().faint_bg_color)
                        .stroke(ui.visuals().window_stroke)
                        .inner_margin(8)
                        .corner_radius(6)
                        .show(ui, |ui| {
                            show(view_ctx.collapsed(false), ui);
                        });
                });
            }
        } else {
            show(view_ctx, ui);
        }
        changed
    }
    fn show_value(&self, context: &Context, ui: &mut Ui) {}
    fn show_body_mut(&mut self, view_ctx: DataViewContext, context: &Context, ui: &mut Ui) -> bool {
        ui.vertical(|ui| self.view_children_mut(view_ctx, context, ui))
            .inner
    }
    fn view_children_mut(
        &mut self,
        view_ctx: DataViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> bool {
        false
    }
    fn show_title(text: impl Into<WidgetText>, ui: &mut Ui, context_menu: impl FnOnce(&mut Ui)) {
        ui.button(text).bar_menu(context_menu);
    }
    fn context_menu(&self, view_ctx: DataViewContext, ui: &mut Ui) {
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
    }
    fn context_menu_mut(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;
        let options = Self::replace_options();
        let lookup_id = Id::new("lookup text");
        if !options.is_empty() {
            if ui
                .menu_button("replace", |ui| {
                    let lookup =
                        if let Some(mut lookup) = ui.data(|r| r.get_temp::<String>(lookup_id)) {
                            let resp = Input::new("").ui_string(&mut lookup, ui);
                            if resp.changed() {
                                ui.data_mut(|w| w.insert_temp(lookup_id, lookup.clone()));
                            }
                            resp.request_focus();
                            lookup
                        } else {
                            String::new()
                        };
                    ScrollArea::vertical()
                        .min_scrolled_height(500.0)
                        .show(ui, |ui| {
                            for mut opt in options {
                                let text = opt.cstr();
                                if !lookup.is_empty()
                                    && !text.get_text().to_lowercase().starts_with(&lookup)
                                {
                                    continue;
                                }
                                let resp = opt.cstr().button(ui);
                                if resp.clicked() || resp.gained_focus() {
                                    self.move_inner(&mut opt);
                                    mem::swap(self, &mut opt);
                                    changed = true;
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
                changed = true;
                *self = Self::wrap(self.clone()).unwrap();
            }
        }
        if ui.button("paste").clicked() {
            changed = true;
            self.paste();
        }
        if changed {
            ui.close_menu();
            ui.data_mut(|w| w.remove_temp::<String>(lookup_id));
        }
        changed
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

impl DataView for Expression {
    fn replace_options() -> Vec<Self> {
        Self::iter().collect_vec()
    }
    fn wrap(value: Self) -> Option<Self> {
        Some(Self::abs(Box::new(value)))
    }
    fn move_inner(&mut self, source: &mut Self) {
        <Expression as Injector<Expression>>::inject_inner(self, source);
        <Expression as Injector<f32>>::inject_inner(self, source);
    }
    fn view_children_mut(
        &mut self,
        view_ctx: DataViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> bool {
        let mut changed = false;
        for (i, e) in <Self as Injector<Expression>>::get_inner_mut(self)
            .into_iter()
            .enumerate()
        {
            changed |= e.view_mut(view_ctx.with_id(i), context, ui);
        }
        for i in <Self as Injector<f32>>::get_inner_mut(self) {
            changed |= i.show_mut(None, ui);
        }
        changed
    }
    fn show_value(&self, context: &Context, ui: &mut Ui) {
        match self.get_value(context) {
            Ok(v) => v.cstr_expanded(),
            Err(e) => e.cstr(),
        }
        .label(ui);
    }
}

impl DataView for VarName {
    fn replace_options() -> Vec<Self> {
        Self::iter().collect_vec()
    }
}

impl DataView for VarValue {
    fn replace_options() -> Vec<Self> {
        Self::iter().collect_vec()
    }
}
