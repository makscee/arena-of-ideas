use serde::de::DeserializeOwned;

use super::*;

pub trait ShowPrefix {
    fn show(&self, ui: &mut Ui);
}
impl ShowPrefix for Option<&str> {
    fn show(&self, ui: &mut Ui) {
        if let Some(s) = self {
            s.cstr_cs(VISIBLE_DARK, CstrStyle::Small).label(ui);
        }
    }
}
pub trait Show: StringData {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui);
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool;
}

impl<T> Show for Vec<Box<T>>
where
    T: Show + Default + Serialize + DeserializeOwned,
{
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        prefix.show(ui);
        for (i, v) in self.into_iter().enumerate() {
            v.show(Some(&format!("[vd {i}:]")), context, ui);
        }
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        prefix.show(ui);
        let mut changed = false;
        let mut swap = None;
        let mut delete = None;
        let mut insert = None;
        let len = self.len();
        fn plus_btn(ui: &mut Ui) -> bool {
            "+".cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold)
                .button(ui)
                .clicked()
        }
        for (i, a) in self.iter_mut().enumerate() {
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
        if self.is_empty() && plus_btn(ui) {
            insert = Some(0);
        }
        if let Some(delete) = delete {
            changed = true;
            self.remove(delete);
        }
        if let Some(index) = insert {
            changed = true;
            self.insert(index, default());
        }
        if let Some((a, b)) = swap {
            changed = true;
            self.swap(a, b);
        }
        changed
    }
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
impl Show for u64 {
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
impl Show for bool {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            Checkbox::new(self, prefix.unwrap_or_default().to_owned().widget(1.0, ui)).ui(ui)
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
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        Input::new(prefix.unwrap_or_default())
            .ui_string(self, ui)
            .changed()
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

impl Show for Material {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        let r = prefix.show(ui);
        let rect = ui.available_rect_before_wrap();
        let mut p = Painter::new(rect, ui.ctx());
        for i in &self.0 {
            let _ = i.paint(context, &mut p, ui);
        }
        for i in &self.0 {
            i.show(None, context, ui);
        }
        r
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        self.0.show_mut(prefix, ui)
    }
}
impl Show for Trigger {
    fn show(&self, prefix: Option<&str>, _: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.cstr_cs(CYAN, CstrStyle::Bold).label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        Selector::new(prefix.unwrap_or_default()).ui_enum(self, ui)
    }
}