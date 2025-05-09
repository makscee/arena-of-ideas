use serde::de::DeserializeOwned;

use super::*;

pub trait ShowPrefix {
    fn show(&self, ui: &mut Ui);
}
impl ShowPrefix for Option<&str> {
    fn show(&self, ui: &mut Ui) {
        if let Some(s) = self {
            s.cstr_cs(ui.visuals().weak_text_color(), CstrStyle::Small)
                .label(ui);
        }
    }
}
pub trait Show {
    fn show(&self, context: &Context, ui: &mut Ui);
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool;
}

impl<T> Show for Option<T>
where
    T: Show + Default + Serialize + DeserializeOwned,
{
    fn show(&self, context: &Context, ui: &mut Ui) {
        if let Some(v) = self.as_ref() {
            v.show(context, ui);
        } else {
            "[tw none]".cstr().label(ui);
        }
    }
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        let mut is_some = self.is_some();
        if Checkbox::new(&mut is_some, "").ui(ui).changed() {
            if is_some {
                *self = Some(default());
            } else {
                *self = None;
            }
            return true;
        }
        if let Some(v) = self.as_mut() {
            v.show_mut(context, ui)
        } else {
            "[tw none".cstr().label(ui);
            false
        }
    }
}

impl Show for VarValue {
    fn show(&self, context: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| match self {
            VarValue::String(v) => v.show(context, ui),
            VarValue::i32(v) => v.show(context, ui),
            VarValue::f32(v) => v.show(context, ui),
            VarValue::u64(v) => v.show(context, ui),
            VarValue::bool(v) => v.show(context, ui),
            VarValue::Vec2(v) => v.show(context, ui),
            VarValue::Color32(v) => v.show(context, ui),
            VarValue::Entity(v) => Entity::from_bits(*v).show(context, ui),
            VarValue::list(v) => {}
        })
        .inner
    }
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| match self {
            VarValue::i32(v) => v.show_mut(context, ui),
            VarValue::f32(v) => v.show_mut(context, ui),
            VarValue::u64(v) => v.show_mut(context, ui),
            VarValue::bool(v) => v.show_mut(context, ui),
            VarValue::String(v) => v.show_mut(context, ui),
            VarValue::Vec2(v) => v.show_mut(context, ui),
            VarValue::Color32(v) => v.show_mut(context, ui),
            VarValue::Entity(v) => Entity::from_bits(*v).show_mut(context, ui),
            VarValue::list(v) => false,
        })
        .inner
    }
}

impl Show for i32 {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        DragValue::new(self).ui(ui).changed()
    }
}
impl Show for f32 {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        DragValue::new(self).min_decimals(1).ui(ui).changed()
    }
}
impl Show for f64 {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        DragValue::new(self).min_decimals(1).ui(ui).changed()
    }
}
impl Show for u64 {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| DragValue::new(self).ui(ui))
            .inner
            .changed()
    }
}
impl Show for u32 {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| DragValue::new(self).ui(ui))
            .inner
            .changed()
    }
}
impl Show for u8 {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| DragValue::new(self).ui(ui))
            .inner
            .changed()
    }
}
impl Show for bool {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        Checkbox::new(self, "").ui(ui).changed()
    }
}
impl Show for Vec2 {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            let rx = DragValue::new(&mut self.x).prefix("x:").ui(ui);
            let ry = DragValue::new(&mut self.y).prefix("y:").ui(ui);
            rx.union(ry)
        })
        .inner
        .changed()
    }
}
impl Show for String {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label_t(ui);
    }
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        Input::new("").ui_string(self, ui).changed()
    }
}
impl Show for Color {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            let mut c = self.c32();
            let r = c.show_mut(context, ui);
            if r {
                *self = c.to_color();
            }
            r
        })
        .inner
    }
}
impl Show for Color32 {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
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
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        let mut err = None;
        ui.horizontal(|ui| {
            let input_id = ui.next_auto_id().with("input");
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
                .id(input_id)
                .ui_string(&mut self.0, ui)
                .changed()
            {
                changed = true;
            }
        });
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
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.to_string().label(ui);
    }
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        self.show(&Context::default(), ui);
        false
    }
}

impl Show for Event {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr_cs(tokens_info().low_contrast_text(), CstrStyle::Bold)
            .label(ui);
    }
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        Selector::new("").ui_enum(self, ui)
    }
}
impl Show for UnitTriggerRef {
    fn show(&self, _context: &Context, _ui: &mut Ui) {}
    fn show_mut(&mut self, _context: &Context, _ui: &mut Ui) -> bool {
        false
    }
}
impl Show for Vec<UnitActionRef> {
    fn show(&self, _context: &Context, _ui: &mut Ui) {}
    fn show_mut(&mut self, _context: &Context, _ui: &mut Ui) -> bool {
        false
    }
}
impl Show for Vec<(UnitTriggerRef, Vec<UnitActionRef>)> {
    fn show(&self, _: &Context, ui: &mut Ui) {}
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        false
    }
}
impl Show for Vec<String> {
    fn show(&self, _: &Context, ui: &mut Ui) {
        for s in self {
            s.label(ui);
        }
    }
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        for s in self {
            changed |= Input::new("").ui_string(s, ui).changed();
        }
        changed
    }
}
impl Show for Vec<u64> {
    fn show(&self, context: &Context, ui: &mut Ui) {
        for i in self {
            i.cstr().show(context, ui);
        }
    }
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.show(context, ui);
        false
    }
}
impl Show for Vec<Action> {
    fn show(&self, context: &Context, ui: &mut Ui) {
        self.view_with_children(ViewContext::new(ui), context, ui);
    }
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.view_with_children_mut(ViewContext::new(ui), context, ui)
            .changed
    }
}
impl Show for Reaction {
    fn show(&self, context: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            let vctx = ViewContext::new(ui).non_interactible(true);
            ui.horizontal(|ui| {
                Icon::Lightning.show(ui);
                self.trigger.view_title(vctx, context, ui);
            });
            self.actions.view(vctx, context, ui);
        });
    }
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.view_with_children_mut(ViewContext::new(ui), context, ui)
            .changed
    }
}

impl Show for Vec<Reaction> {
    fn show(&self, context: &Context, ui: &mut Ui) {
        self.view_with_children(ViewContext::new(ui), context, ui);
    }
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.view_with_children_mut(ViewContext::new(ui), context, ui)
            .changed
    }
}

impl Show for Material {
    fn show(&self, context: &Context, ui: &mut Ui) {
        self.view_with_children(ViewContext::new(ui), context, ui);
    }
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.view_with_children_mut(ViewContext::new(ui), context, ui)
            .changed
    }
}

impl Show for VarName {
    fn show(&self, context: &Context, ui: &mut Ui) {
        self.view(ViewContext::new(ui), context, ui);
    }
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.view_mut(ViewContext::new(ui), context, ui).changed
    }
}

impl Show for ExpressionError {
    fn show(&self, context: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.show(context, ui);
        false
    }
}
