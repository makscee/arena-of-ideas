use super::*;
use serde::de::DeserializeOwned;

pub trait SFnShow {
    fn show(&self, context: &Context, ui: &mut Ui);
}

pub trait SFnShowMut {
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool;
}

impl<T> SFnShow for Option<T>
where
    T: SFnShow + Default + Serialize + DeserializeOwned,
{
    fn show(&self, context: &Context, ui: &mut Ui) {
        if let Some(v) = self.as_ref() {
            v.show(context, ui);
        } else {
            "[tw none]".cstr().label(ui);
        }
    }
}

impl<T> SFnShowMut for Option<T>
where
    T: SFnShowMut + Default + Serialize + DeserializeOwned,
{
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

impl SFnShow for VarValue {
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
            VarValue::list(v) => {
                ui.horizontal(|ui| {
                    "[tw List: ]".cstr().label(ui);
                    for v in v {
                        v.show(context, ui);
                    }
                });
            }
        });
    }
}

impl SFnShowMut for VarValue {
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
            VarValue::list(v) => {
                ui.horizontal(|ui| {
                    "[tw List: ]".cstr().label(ui);
                    let mut r = false;
                    for v in v {
                        r |= v.show_mut(context, ui);
                    }
                    r
                })
                .inner
            }
        })
        .inner
    }
}

impl SFnShow for i32 {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl SFnShowMut for i32 {
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        DragValue::new(self).ui(ui).changed()
    }
}

impl SFnShow for f32 {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl SFnShowMut for f32 {
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        DragValue::new(self).min_decimals(1).ui(ui).changed()
    }
}

impl SFnShow for f64 {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl SFnShowMut for f64 {
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        DragValue::new(self).min_decimals(1).ui(ui).changed()
    }
}

impl SFnShow for u64 {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl SFnShowMut for u64 {
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| DragValue::new(self).ui(ui))
            .inner
            .changed()
    }
}

impl SFnShow for u32 {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl SFnShowMut for u32 {
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| DragValue::new(self).ui(ui))
            .inner
            .changed()
    }
}

impl SFnShow for u8 {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl SFnShowMut for u8 {
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| DragValue::new(self).ui(ui))
            .inner
            .changed()
    }
}

impl SFnShow for bool {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl SFnShowMut for bool {
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        Checkbox::new(self, "").ui(ui).changed()
    }
}

impl SFnShow for Vec2 {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl SFnShowMut for Vec2 {
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

impl SFnShow for String {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label_t(ui);
    }
}

impl SFnShowMut for String {
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        Input::new("").ui_string(self, ui).changed()
    }
}

impl SFnShow for Color {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl SFnShowMut for Color {
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

impl SFnShow for Color32 {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl SFnShowMut for Color32 {
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
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

impl SFnShow for HexColor {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl SFnShowMut for HexColor {
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
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

impl SFnShow for Entity {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.to_string().label(ui);
    }
}

impl SFnShowMut for Entity {
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        self.show(&Context::default(), ui);
        false
    }
}

impl SFnShow for Event {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr_cs(low_contrast_text(), CstrStyle::Bold).label(ui);
    }
}

impl SFnShowMut for Event {
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        Selector::new("").ui_enum(self, ui)
    }
}

impl SFnShow for UnitActionRange {
    fn show(&self, _context: &Context, ui: &mut Ui) {
        format!("{}: {}-{}", self.trigger, self.start, self.length).label(ui);
    }
}

impl SFnShowMut for UnitActionRange {
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.show(context, ui);
        false
    }
}

impl SFnShow for Vec<UnitActionRange> {
    fn show(&self, _context: &Context, _ui: &mut Ui) {}
}

impl SFnShowMut for Vec<UnitActionRange> {
    fn show_mut(&mut self, _context: &Context, _ui: &mut Ui) -> bool {
        false
    }
}

impl SFnShow for Vec<(u64, Vec<UnitActionRange>)> {
    fn show(&self, _: &Context, _: &mut Ui) {}
}

impl SFnShowMut for Vec<(u64, Vec<UnitActionRange>)> {
    fn show_mut(&mut self, _: &Context, _: &mut Ui) -> bool {
        false
    }
}

impl SFnShow for Vec<String> {
    fn show(&self, _: &Context, ui: &mut Ui) {
        for s in self {
            s.label(ui);
        }
    }
}

impl SFnShowMut for Vec<String> {
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        for s in self {
            changed |= Input::new("").ui_string(s, ui).changed();
        }
        changed
    }
}

impl SFnShow for Vec<u64> {
    fn show(&self, context: &Context, ui: &mut Ui) {
        for i in self {
            i.cstr().show(context, ui);
        }
    }
}

impl SFnShowMut for Vec<u64> {
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.show(context, ui);
        false
    }
}

impl SFnShow for Vec<Action> {
    fn show(&self, context: &Context, ui: &mut Ui) {
        let vctx = ViewContext::new(ui).non_interactible(true);
        for a in self {
            a.view_title(vctx, context, ui);
        }
    }
}

impl SFnShowMut for Vec<Action> {
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.view_with_children_mut(ViewContext::new(ui), context, ui)
            .changed
    }
}

impl SFnShow for Reaction {
    fn show(&self, context: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            let vctx = ViewContext::new(ui).non_interactible(true);
            ui.horizontal(|ui| {
                Icon::Lightning.show(ui);
                self.trigger.view_title(vctx, context, ui);
            });
            self.actions.show(context, ui);
        });
    }
}

impl SFnShowMut for Reaction {
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.view_with_children_mut(ViewContext::new(ui), context, ui)
            .changed
    }
}

impl SFnShow for Vec<Reaction> {
    fn show(&self, context: &Context, ui: &mut Ui) {
        for r in self {
            r.show(context, ui);
        }
    }
}

impl SFnShowMut for Vec<Reaction> {
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.view_with_children_mut(ViewContext::new(ui), context, ui)
            .changed
    }
}

impl SFnShow for Material {
    fn show(&self, context: &Context, ui: &mut Ui) {
        self.view_with_children(ViewContext::new(ui), context, ui);
    }
}

impl SFnShowMut for Material {
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.view_with_children_mut(ViewContext::new(ui), context, ui)
            .changed
    }
}

impl SFnShow for VarName {
    fn show(&self, context: &Context, ui: &mut Ui) {
        self.view(ViewContext::new(ui), context, ui);
    }
}

impl SFnShowMut for VarName {
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.view_mut(ViewContext::new(ui), context, ui).changed
    }
}

impl SFnShow for ExpressionError {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl SFnShowMut for ExpressionError {
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.show(context, ui);
        false
    }
}

impl SFnShow for CardKind {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.as_ref().cstr().label(ui);
    }
}

impl SFnShowMut for CardKind {
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.show(context, ui);
        false
    }
}

impl SFnShow for Vec<(CardKind, u64)> {
    fn show(&self, _: &Context, _: &mut Ui) {
        todo!()
    }
}

impl SFnShowMut for Vec<(CardKind, u64)> {
    fn show_mut(&mut self, _: &Context, _: &mut Ui) -> bool {
        todo!()
    }
}

impl SFnShow for Vec<ShopOffer> {
    fn show(&self, _: &Context, ui: &mut Ui) {
        "shop offers".cstr().label(ui);
    }
}

impl SFnShowMut for Vec<ShopOffer> {
    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.show(context, ui);
        false
    }
}

impl SFnShow for MagicType {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.as_ref().cstr().label(ui);
    }
}

impl SFnShowMut for MagicType {
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        Selector::new("").ui_enum(self, ui)
    }
}

impl SFnShow for Trigger {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.as_ref().cstr().label(ui);
    }
}

impl SFnShowMut for Trigger {
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        Selector::new("").ui_enum(self, ui)
    }
}

// Basic SFnShow implementation for Expression
impl SFnShow for Expression {
    fn show(&self, _context: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl SFnShow for Action {
    fn show(&self, _context: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl SFnShowMut for Action {
    fn show_mut(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        Selector::new("").ui_enum(self, ui)
    }
}

impl SFnShow for PainterAction {
    fn show(&self, _context: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl SFnShowMut for PainterAction {
    fn show_mut(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        Selector::new("").ui_enum(self, ui)
    }
}
