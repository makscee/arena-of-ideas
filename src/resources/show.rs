use super::*;

pub trait ShowPrefix {
    fn show(&self, ui: &mut Ui);
}
impl ShowPrefix for Option<&str> {
    fn show(&self, ui: &mut Ui) {
        if let Some(s) = self {
            s.cstr_cs(tokens_global().low_contrast_text(), CstrStyle::Small)
                .label(ui);
        }
    }
}
pub trait Show: StringData {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui);
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool;
}

impl Show for VarName {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr_expanded().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        Selector::new(prefix.unwrap_or_default()).ui_enum(self, ui)
    }
}
impl Show for VarValue {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| match self {
            VarValue::String(v) => v.show(prefix, context, ui),
            VarValue::i32(v) => v.show(prefix, context, ui),
            VarValue::f32(v) => v.show(prefix, context, ui),
            VarValue::u64(v) => v.show(prefix, context, ui),
            VarValue::bool(v) => v.show(prefix, context, ui),
            VarValue::Vec2(v) => v.show(prefix, context, ui),
            VarValue::Color32(v) => v.show(prefix, context, ui),
            VarValue::Entity(v) => Entity::from_bits(*v).show(prefix, context, ui),
            VarValue::List(v) => v.show(prefix, context, ui),
        })
        .inner
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| match self {
            VarValue::i32(v) => v.show_mut(prefix, ui),
            VarValue::f32(v) => v.show_mut(prefix, ui),
            VarValue::u64(v) => v.show_mut(prefix, ui),
            VarValue::bool(v) => v.show_mut(prefix, ui),
            VarValue::String(v) => v.show_mut(prefix, ui),
            VarValue::Vec2(v) => v.show_mut(prefix, ui),
            VarValue::Color32(v) => v.show_mut(prefix, ui),
            VarValue::Entity(v) => Entity::from_bits(*v).show_mut(prefix, ui),
            VarValue::List(v) => v.show_mut(prefix, ui),
        })
        .inner
    }
}

impl Show for i32 {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr().label(ui);
            }
            DragValue::new(self).ui(ui)
        })
        .inner
        .changed()
    }
}
impl Show for f32 {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            prefix.show(ui);
            DragValue::new(self).min_decimals(1).ui(ui)
        })
        .inner
        .changed()
    }
}
impl Show for f64 {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            prefix.show(ui);
            DragValue::new(self).min_decimals(1).ui(ui)
        })
        .inner
        .changed()
    }
}
impl Show for u64 {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            prefix.show(ui);
            DragValue::new(self).ui(ui)
        })
        .inner
        .changed()
    }
}
impl Show for u32 {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            prefix.show(ui);
            DragValue::new(self).ui(ui)
        })
        .inner
        .changed()
    }
}
impl Show for bool {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            Checkbox::new(
                self,
                prefix
                    .unwrap_or_default()
                    .to_owned()
                    .widget(1.0, ui.style()),
            )
            .ui(ui)
        })
        .inner
        .changed()
    }
}
impl Show for Vec2 {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr().label(ui);
            }
            let rx = DragValue::new(&mut self.x).prefix("x:").ui(ui);
            let ry = DragValue::new(&mut self.y).prefix("y:").ui(ui);
            rx.union(ry)
        })
        .inner
        .changed()
    }
}
impl Show for String {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label_w(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        Input::new(prefix.unwrap_or_default())
            .ui_string(self, ui)
            .changed()
    }
}
impl Show for Option<String> {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        if let Some(s) = self {
            s.cstr().label_w(ui);
        } else {
            "none"
                .cstr_cs(tokens_global().low_contrast_text(), CstrStyle::Small)
                .label_w(ui);
        }
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        let mut changed = false;
        let mut checked = self.is_some();
        if Checkbox::new(&mut checked, "").ui(ui).changed() {
            changed = true;
            if checked {
                *self = Some(default());
            } else {
                *self = None;
            }
        }
        if let Some(s) = self {
            changed |= Input::new(prefix.unwrap_or_default())
                .ui_string(s, ui)
                .changed();
        }
        changed
    }
}
impl Show for Color {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            let mut c = self.c32();
            let r = c.show_mut(prefix, ui);
            if r {
                *self = c.to_color();
            }
            r
        })
        .inner
    }
}
impl Show for Color32 {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            if let Some(prefix) = prefix {
                prefix.cstr().label(ui);
            }
            let mut hsva = self.clone().into();
            let r = ui.color_edit_button_hsva(&mut hsva).changed();
            if r {
                *self = hsva.into();
            }
            r
        })
        .inner
    }
}
impl Show for HexColor {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        prefix.show(ui);
        let mut changed = false;
        let mut err = None;
        let c = match self.try_c32() {
            Ok(c) => {
                let mut rgb = [c.r(), c.g(), c.b()];
                if ui.color_edit_button_srgb(&mut rgb).changed() {
                    *self = Color32::from_rgb(rgb[0], rgb[1], rgb[2]).into();
                    changed = true;
                }
                Some(c)
            }
            Err(e) => {
                err = Some(format!("[red Hex parse err:] {e:?}"));
                None
            }
        };
        if Input::new("")
            .char_limit(7)
            .desired_width(60.0)
            .color_opt(c)
            .id("hex input")
            .ui_string(&mut self.0, ui)
            .changed()
        {
            changed = true;
        }
        if let Some(err) = err {
            if "reset".cstr().button(ui).clicked() {
                *self = default();
                changed = true;
            }
            err.label(ui);
        }
        changed
    }
}
impl Show for Entity {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.to_string().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        self.show(prefix, &Context::default(), ui);
        false
    }
}

fn material_view(m: &Material, context: &Context, ui: &mut Ui) {
    let size_id = ui.id().with("view size");
    let mut size = ui.ctx().data_mut(|w| *w.get_temp_mut_or(size_id, 150.0));
    if DragValue::new(&mut size).ui(ui).changed() {
        ui.ctx().data_mut(|w| w.insert_temp(size_id, size));
    }
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
    RepresentationPlugin::paint_rect(rect, context, m, ui).log();
    ui.painter().rect_stroke(
        rect,
        0,
        Stroke::new(1.0, tokens_global().subtle_borders_and_separators()),
        egui::StrokeKind::Middle,
    );
}
impl Show for Material {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        prefix.show(ui);
        material_view(self, context, ui);
        for i in &self.0 {
            i.show(None, context, ui);
        }
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        material_view(self, &Context::default(), ui);
        self.0.show_mut(prefix, ui)
    }
}
impl Show for Actions {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        let r = prefix.show(ui);
        for i in &self.0 {
            i.show(None, context, ui);
        }
        r
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        self.0.show_mut(prefix, ui)
    }
}
impl Show for Event {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr_cs(tokens_info().low_contrast_text(), CstrStyle::Bold)
            .label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        prefix.show(ui);
        Selector::new("").ui_enum(self, ui)
    }
}
impl Show for Vec<(UnitTriggerRef, Vec<UnitActionRef>)> {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        todo!()
    }

    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        todo!()
    }
}
impl Show for Reaction {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.trigger.show(None, context, ui);
        self.actions.show(None, context, ui);
    }

    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        prefix.show(ui);
        let changed = self.trigger.show_mut(None, ui);
        self.actions.show_mut(prefix, ui) || changed
    }
}
