use super::*;
use crate::ui::core::enum_colors::EnumColor;
use crate::ui::see::{
    Cstr, CstrTrait, RecursiveField, RecursiveFieldMut, RecursiveFields, RecursiveFieldsMut,
    ToCstr, ToRecursiveValue, ToRecursiveValueMut,
};

// ============================================================================
// Basic Types Implementations
// ============================================================================

// FTitle implementations for basic types
impl FTitle for i32 {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FTitle for f32 {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FTitle for String {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FTitle for bool {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FTitle for Vec2 {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FTitle for Color32 {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FTitle for HexColor {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

// FDisplay implementations for basic types
impl FDisplay for i32 {
    fn display(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl FDisplay for f32 {
    fn display(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl FDisplay for String {
    fn display(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label_t(ui);
    }
}

impl FDisplay for bool {
    fn display(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl FDisplay for Vec2 {
    fn display(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl FDisplay for Color32 {
    fn display(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl FDisplay for HexColor {
    fn display(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

// FEdit implementations for basic types
impl FEdit for i32 {
    fn edit(&mut self, _: &Context, ui: &mut Ui) -> bool {
        DragValue::new(self).ui(ui).changed()
    }
}

impl FEdit for f32 {
    fn edit(&mut self, _: &Context, ui: &mut Ui) -> bool {
        DragValue::new(self).min_decimals(1).ui(ui).changed()
    }
}

impl FEdit for String {
    fn edit(&mut self, _: &Context, ui: &mut Ui) -> bool {
        Input::new("").ui_string(self, ui).changed()
    }
}

impl FEdit for bool {
    fn edit(&mut self, _: &Context, ui: &mut Ui) -> bool {
        Checkbox::new(self, "").ui(ui).changed()
    }
}

impl FEdit for Vec2 {
    fn edit(&mut self, _: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            let rx = DragValue::new(&mut self.x).prefix("x:").ui(ui);
            let ry = DragValue::new(&mut self.y).prefix("y:").ui(ui);
            rx.union(ry)
        })
        .inner
        .changed()
    }
}

impl FEdit for Color32 {
    fn edit(&mut self, _: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            let mut hsva = (*self).into();
            let r = ui.color_edit_button_hsva(&mut hsva).changed();
            if r {
                *self = hsva.into();
            }
            r
        })
        .inner
    }
}

impl FEdit for HexColor {
    fn edit(&mut self, _: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            let input_id = ui.next_auto_id().with("input");
            let c = self.try_c32().ok();
            if let Some(c) = c {
                let mut rgb = [c.r(), c.g(), c.b()];
                if ui.color_edit_button_srgb(&mut rgb).changed() {
                    *self = Color32::from_rgb(rgb[0], rgb[1], rgb[2]).into();
                    changed = true;
                }
            }
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
        changed
    }
}

// ============================================================================
// Game Types Implementations
// ============================================================================

// VarName
impl FTitle for VarName {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FColoredTitle for VarName {
    fn title_color(&self, _: &Context) -> Color32 {
        self.color()
    }
}

impl FDisplay for VarName {
    fn display(&self, context: &Context, ui: &mut Ui) {
        self.view(ViewContext::new(ui), context, ui);
    }
}

impl FEdit for VarName {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.view_mut(ViewContext::new(ui), context, ui).changed
    }
}

// VarValue
impl FTitle for VarValue {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FDisplay for VarValue {
    fn display(&self, context: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| match self {
            VarValue::String(v) => v.display(context, ui),
            VarValue::i32(v) => v.display(context, ui),
            VarValue::f32(v) => v.display(context, ui),
            VarValue::u64(v) => {
                v.cstr().label(ui);
            }
            VarValue::bool(v) => v.display(context, ui),
            VarValue::Vec2(v) => v.display(context, ui),
            VarValue::Color32(v) => v.display(context, ui),
            VarValue::Entity(v) => {
                Entity::from_bits(*v).to_string().label(ui);
            }
            VarValue::list(v) => {
                ui.horizontal(|ui| {
                    "[tw List: ]".cstr().label(ui);
                    for v in v {
                        v.display(context, ui);
                    }
                });
            }
        });
    }
}

impl FEdit for VarValue {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| match self {
            VarValue::i32(v) => v.edit(context, ui),
            VarValue::f32(v) => v.edit(context, ui),
            VarValue::u64(v) => DragValue::new(v).ui(ui).changed(),
            VarValue::bool(v) => v.edit(context, ui),
            VarValue::String(v) => v.edit(context, ui),
            VarValue::Vec2(v) => v.edit(context, ui),
            VarValue::Color32(v) => v.edit(context, ui),
            VarValue::Entity(_) => false,
            VarValue::list(v) => {
                let mut r = false;
                for v in v {
                    r |= v.edit(context, ui);
                }
                r
            }
        })
        .inner
    }
}

// Expression
impl FTitle for Expression {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FColoredTitle for Expression {
    fn title_color(&self, _: &Context) -> Color32 {
        self.color()
    }
}

impl FDisplay for Expression {
    fn display(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl FEdit for Expression {
    fn edit(&mut self, _: &Context, ui: &mut Ui) -> bool {
        Selector.ui_enum(self, ui)
    }
}

impl FRecursive for Expression {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        RecursiveFields::recursive_fields_old(self)
    }
}

impl FRecursiveMut for Expression {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        RecursiveFieldsMut::recursive_fields_mut_old(self)
    }
}

// Action
impl FTitle for Action {
    fn title(&self, context: &Context) -> Cstr {
        match self {
            Action::use_ability => {
                let mut r = self.cstr();
                if let Ok(ability) = context.get_string(VarName::ability_name) {
                    if let Ok(color) = context.get_color(VarName::color) {
                        r += " ";
                        r += &ability.cstr_cs(color, CstrStyle::Bold);
                        if let Ok(lvl) = context.get_i32(VarName::lvl) {
                            r += &format!(
                                " [tw [s lvl]][{} [b {lvl}]]",
                                VarName::lvl.color().to_hex()
                            );
                        }
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
                        if let Ok(lvl) = context.get_i32(VarName::lvl) {
                            r += &format!(
                                " [tw [s lvl]][{} [b {lvl}]]",
                                VarName::lvl.color().to_hex()
                            );
                        }
                    }
                }
                r
            }
            _ => self.cstr(),
        }
    }
}

impl FColoredTitle for Action {
    fn title_color(&self, _: &Context) -> Color32 {
        self.color()
    }
}

impl FDisplay for Action {
    fn display(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl FEdit for Action {
    fn edit(&mut self, _: &Context, ui: &mut Ui) -> bool {
        Selector.ui_enum(self, ui)
    }
}

impl FRecursive for Action {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        RecursiveFields::recursive_fields_old(self)
    }
}

impl FRecursiveMut for Action {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        RecursiveFieldsMut::recursive_fields_mut_old(self)
    }
}

// PainterAction
impl FTitle for PainterAction {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FColoredTitle for PainterAction {
    fn title_color(&self, context: &Context) -> Color32 {
        context
            .get_color(VarName::color)
            .unwrap_or(Color32::from_rgb(0, 255, 255))
    }
}

impl FDisplay for PainterAction {
    fn display(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
}

impl FEdit for PainterAction {
    fn edit(&mut self, _: &Context, ui: &mut Ui) -> bool {
        Selector.ui_enum(self, ui)
    }
}

impl FRecursive for PainterAction {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        RecursiveFields::recursive_fields_old(self)
    }
}

impl FRecursiveMut for PainterAction {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        RecursiveFieldsMut::recursive_fields_mut_old(self)
    }
}

// Material
impl FTitle for Material {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FDisplay for Material {
    fn display(&self, context: &Context, ui: &mut Ui) {
        self.render(context).recursive_show(ui);
    }
}

impl FEdit for Material {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        let size_id = ui.id().with("view size");
        let mut size = ui.ctx().data_mut(|w| *w.get_temp_mut_or(size_id, 60.0));
        if DragValue::new(&mut size).ui(ui).changed() {
            ui.ctx().data_mut(|w| w.insert_temp(size_id, size));
        }
        let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
        RepresentationPlugin::paint_rect(rect, context, self, ui).ui(ui);
        ui.painter().rect_stroke(
            rect,
            0,
            Stroke::new(1.0, subtle_borders_and_separators()),
            egui::StrokeKind::Middle,
        );
        // Vec<PainterAction> doesn't support recursive_edit directly
        // Show a simple editor for the actions
        let mut changed = false;
        ui.vertical(|ui| {
            ui.label("Painter Actions:");
            for (i, action) in self.0.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("[{}]", i));
                    ui.label(format!("{:?}", action));
                });
            }
        });
        changed
    }
}

impl FRecursive for Material {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        RecursiveFields::recursive_fields_old(self)
    }
}

impl FRecursiveMut for Material {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        RecursiveFieldsMut::recursive_fields_mut_old(self)
    }
}

// Reaction
impl FTitle for Reaction {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FDisplay for Reaction {
    fn display(&self, context: &Context, ui: &mut Ui) {
        self.render(context).recursive_show(ui);
    }
}

impl FRecursive for Reaction {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        RecursiveFields::recursive_fields_old(self)
    }
}

impl FRecursiveMut for Reaction {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        RecursiveFieldsMut::recursive_fields_mut_old(self)
    }
}

// ============================================================================
// Node Implementations
// ============================================================================

// NUnit
impl FTitle for NUnit {
    fn title(&self, context: &Context) -> Cstr {
        let color = context.get_color(VarName::color).unwrap_or(MISSING_COLOR);
        self.unit_name.cstr_c(color)
    }
}

impl FDescription for NUnit {
    fn description(&self, context: &Context) -> Cstr {
        if let Ok(description) = self.description_load(context) {
            description.description.clone()
        } else {
            String::new()
        }
    }
}

impl FStats for NUnit {
    fn stats(&self, context: &Context) -> Vec<(VarName, VarValue)> {
        let mut stats = vec![];

        if let Ok(pwr) = context.get_var(VarName::pwr) {
            stats.push((VarName::pwr, pwr));
        }
        if let Ok(hp) = context.get_var(VarName::hp) {
            stats.push((VarName::hp, hp));
        }

        let tier = if let Ok(behavior) = context.first_parent_recursive::<NUnitBehavior>(self.id) {
            behavior.reaction.tier()
        } else {
            0
        };
        stats.push((VarName::tier, (tier as i32).into()));

        stats
    }
}

impl FTag for NUnit {
    fn tag_name(&self, context: &Context) -> Cstr {
        context.get_string(VarName::unit_name).unwrap_or_default()
    }

    fn tag_value(&self, context: &Context) -> Option<Cstr> {
        let tier = if let Ok(behavior) = context.first_parent_recursive::<NUnitBehavior>(self.id) {
            behavior.reaction.tier()
        } else {
            0
        };
        let lvl = context.get_i32(VarName::lvl).unwrap_or_default();
        let xp = match context.get_i32(VarName::xp) {
            Ok(v) => format!(" [tw {v}]/[{} [b {lvl}]]", VarName::lvl.color().to_hex()),
            Err(_) => default(),
        };

        Some(format!(
            "[b {} {} [tw T]{}]{xp}",
            context
                .get_i32(VarName::pwr)
                .unwrap_or_default()
                .cstr_c(VarName::pwr.color()),
            context
                .get_i32(VarName::hp)
                .unwrap_or_default()
                .cstr_c(VarName::hp.color()),
            (tier as i32).cstr_c(VarName::tier.color())
        ))
    }

    fn tag_color(&self, context: &Context) -> Color32 {
        context.get_color(VarName::color).unwrap_or(MISSING_COLOR)
    }
}

impl FInfo for NUnit {
    fn info(&self, context: &Context) -> Cstr {
        let mut info_parts = Vec::new();
        if let Ok(stats) = self.stats_load(context) {
            info_parts.push(format!(
                "[{} {}]/[{} {}]",
                VarName::pwr.color().to_hex(),
                stats.pwr,
                VarName::hp.color().to_hex(),
                stats.hp
            ));
        }
        if let Ok(house) = context.first_parent::<NHouse>(self.id()) {
            let color = house.color_for_text(context);
            info_parts.push(house.house_name.cstr_c(color));
        }
        if let Ok(desc) = self.description_load(context) {
            if !desc.description.is_empty() {
                info_parts.push(desc.description.clone());
            }
        }
        info_parts.join(" | ")
    }
}

impl FContextMenu for NUnit {
    fn context_actions(&self, _: &Context) -> Vec<ContextAction<Self>> {
        vec![]
    }
}

impl FCopy for NUnit {}
impl FPaste for NUnit {}

impl FEdit for NUnit {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Name:");
            changed |= ui.text_edit_singleline(&mut self.unit_name).changed();
        });
        changed
    }
}

// NHouse
impl FTitle for NHouse {
    fn title(&self, context: &Context) -> Cstr {
        let color = self.color_for_text(context);
        self.house_name.cstr_c(color)
    }
}

impl FDescription for NHouse {
    fn description(&self, _: &Context) -> Cstr {
        String::new()
    }
}

impl FStats for NHouse {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NHouse {
    fn tag_name(&self, _: &Context) -> Cstr {
        self.house_name.clone()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        None
    }

    fn tag_color(&self, context: &Context) -> Color32 {
        self.color_for_text(context)
    }
}

impl FInfo for NHouse {
    fn info(&self, context: &Context) -> Cstr {
        let mut info_parts = Vec::new();

        let units_count = context
            .collect_children_components::<NUnit>(self.id)
            .map(|u| u.len())
            .unwrap_or_default();
        if units_count > 0 {
            info_parts.push(format!("units: {}", units_count));
        }
        let color = self.color_for_text(context);
        if let Ok(ability) = self.ability_load(context) {
            info_parts.push(ability.ability_name.cstr_c(color));
        }
        if let Ok(status) = self.status_load(context) {
            info_parts.push(status.status_name.cstr_c(color));
        }

        info_parts.join(" | ")
    }
}

impl FContextMenu for NHouse {
    fn context_actions(&self, _: &Context) -> Vec<ContextAction<Self>> {
        vec![]
    }
}

impl FCopy for NHouse {}
impl FPaste for NHouse {}

impl FEdit for NHouse {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Name:");
            changed |= ui.text_edit_singleline(&mut self.house_name).changed();
        });
        changed
    }
}

// NAbilityMagic
impl FTitle for NAbilityMagic {
    fn title(&self, context: &Context) -> Cstr {
        let color = context.get_color(VarName::color).unwrap_or(MISSING_COLOR);
        self.ability_name.cstr_c(color)
    }
}

impl FDescription for NAbilityMagic {
    fn description(&self, context: &Context) -> Cstr {
        if let Ok(description) = self.description_load(context) {
            description.description.clone()
        } else {
            String::new()
        }
    }
}

impl FStats for NAbilityMagic {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NAbilityMagic {
    fn tag_name(&self, _: &Context) -> Cstr {
        self.ability_name.clone()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        None
    }

    fn tag_color(&self, context: &Context) -> Color32 {
        context.get_color(VarName::color).unwrap_or(MISSING_COLOR)
    }
}

impl FContextMenu for NAbilityMagic {
    fn context_actions(&self, _: &Context) -> Vec<ContextAction<Self>> {
        vec![]
    }
}

impl FCopy for NAbilityMagic {}
impl FPaste for NAbilityMagic {}

impl FInfo for NAbilityMagic {
    fn info(&self, _context: &Context) -> Cstr {
        format!("Ability: {}", self.ability_name).cstr()
    }
}

impl FEdit for NAbilityMagic {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Name:");
            changed |= ui.text_edit_singleline(&mut self.ability_name).changed();
        });
        changed
    }
}

// NStatusMagic
impl FTitle for NStatusMagic {
    fn title(&self, context: &Context) -> Cstr {
        let color = context.get_color(VarName::color).unwrap_or(MISSING_COLOR);
        self.status_name.cstr_c(color)
    }
}

impl FDescription for NStatusMagic {
    fn description(&self, context: &Context) -> Cstr {
        if let Ok(description) = self.description_load(context) {
            description.description.clone()
        } else {
            String::new()
        }
    }
}

impl FStats for NStatusMagic {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NStatusMagic {
    fn tag_name(&self, _: &Context) -> Cstr {
        self.status_name.clone()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        None
    }

    fn tag_color(&self, context: &Context) -> Color32 {
        context.get_color(VarName::color).unwrap_or(MISSING_COLOR)
    }
}

impl FContextMenu for NStatusMagic {
    fn context_actions(&self, _: &Context) -> Vec<ContextAction<Self>> {
        vec![]
    }
}

impl FCopy for NStatusMagic {}
impl FPaste for NStatusMagic {}

impl FInfo for NStatusMagic {
    fn info(&self, _context: &Context) -> Cstr {
        format!("Status: {}", self.status_name).cstr()
    }
}

impl FEdit for NStatusMagic {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Name:");
            changed |= ui.text_edit_singleline(&mut self.status_name).changed();
        });
        changed
    }
}

// Implement FTitle for other node types
impl FTitle for NArena {
    fn title(&self, _: &Context) -> Cstr {
        "Arena".cstr()
    }
}

impl FDescription for NArena {
    fn description(&self, _context: &Context) -> Cstr {
        let pools_count = self.floor_pools.len();
        let bosses_count = self.floor_bosses.len();
        format!("{} floor pools, {} bosses", pools_count, bosses_count).cstr()
    }
}

impl FStats for NArena {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NArena {
    fn tag_name(&self, _: &Context) -> Cstr {
        "Arena".cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        None
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        Color32::from_rgb(128, 0, 128)
    }
}

impl FContextMenu for NArena {
    fn context_actions(&self, _: &Context) -> Vec<ContextAction<Self>> {
        vec![]
    }
}

impl FTitle for NFloorPool {
    fn title(&self, _: &Context) -> Cstr {
        format!("Floor {} Pool", self.floor).cstr()
    }
}

impl FDescription for NFloorPool {
    fn description(&self, _: &Context) -> Cstr {
        format!("{} teams", self.teams.len()).cstr()
    }
}

impl FStats for NFloorPool {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![(VarName::floor, VarValue::i32(self.floor))]
    }
}

impl FTag for NFloorPool {
    fn tag_name(&self, _: &Context) -> Cstr {
        format!("F{}", self.floor).cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        Some(format!("{} teams", self.teams.len()).cstr())
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        Color32::from_rgb(0, 128, 128)
    }
}

impl FContextMenu for NFloorPool {
    fn context_actions(&self, _: &Context) -> Vec<ContextAction<Self>> {
        vec![]
    }
}

impl FTitle for NFloorBoss {
    fn title(&self, _: &Context) -> Cstr {
        format!("Floor {} Boss", self.floor).cstr()
    }
}

impl FDescription for NFloorBoss {
    fn description(&self, _: &Context) -> Cstr {
        "Boss team".cstr()
    }
}

impl FStats for NFloorBoss {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![(VarName::floor, VarValue::i32(self.floor))]
    }
}

impl FTag for NFloorBoss {
    fn tag_name(&self, _: &Context) -> Cstr {
        format!("Boss F{}", self.floor).cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        None
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        Color32::from_rgb(255, 0, 0)
    }
}

impl FContextMenu for NFloorBoss {
    fn context_actions(&self, _: &Context) -> Vec<ContextAction<Self>> {
        vec![]
    }
}

impl FTitle for NPlayer {
    fn title(&self, _: &Context) -> Cstr {
        self.player_name.cstr()
    }
}

impl FDescription for NPlayer {
    fn description(&self, context: &Context) -> Cstr {
        if let Ok(data) = self.player_data_load(context) {
            if data.online {
                "Online".cstr_c(Color32::from_rgb(0, 255, 0))
            } else {
                "Offline".cstr_c(Color32::from_rgb(128, 128, 128))
            }
        } else {
            String::new()
        }
    }
}

impl FStats for NPlayer {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NPlayer {
    fn tag_name(&self, _: &Context) -> Cstr {
        self.player_name.cstr()
    }

    fn tag_value(&self, context: &Context) -> Option<Cstr> {
        if let Ok(data) = self.player_data_load(context) {
            Some(if data.online {
                "●".cstr_c(Color32::from_rgb(0, 255, 0))
            } else {
                "○".cstr_c(Color32::from_rgb(128, 128, 128))
            })
        } else {
            None
        }
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        Color32::from_rgb(0, 0, 255)
    }
}

impl FContextMenu for NPlayer {
    fn context_actions(&self, _: &Context) -> Vec<ContextAction<Self>> {
        vec![]
    }
}

impl FCopy for NPlayer {}
impl FPaste for NPlayer {}

impl FDisplay for NPlayer {
    fn display(&self, context: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.player_name
                .cstr_c(Color32::from_rgb(0, 0, 255))
                .label(ui);
            if let Ok(data) = self.player_data_load(context) {
                if data.online {
                    "●".cstr_c(Color32::from_rgb(0, 255, 0)).label(ui);
                } else {
                    "○".cstr_c(Color32::from_rgb(128, 128, 128)).label(ui);
                }
            }
        });
    }
}

impl FEdit for NPlayer {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        ui.text_edit_singleline(&mut self.player_name).changed()
    }
}

impl FTitle for NPlayerData {
    fn title(&self, _: &Context) -> Cstr {
        "Player Data".cstr()
    }
}

impl FDescription for NPlayerData {
    fn description(&self, _: &Context) -> Cstr {
        if self.online {
            format!("Online, last login: {}", self.last_login).cstr()
        } else {
            format!("Offline, last login: {}", self.last_login).cstr()
        }
    }
}

impl FStats for NPlayerData {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NPlayerData {
    fn tag_name(&self, _: &Context) -> Cstr {
        "Data".cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        Some(if self.online {
            "Online".cstr()
        } else {
            "Offline".cstr()
        })
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        if self.online {
            Color32::from_rgb(0, 255, 0)
        } else {
            Color32::from_rgb(128, 128, 128)
        }
    }
}

impl FTitle for NPlayerIdentity {
    fn title(&self, _: &Context) -> Cstr {
        "Player Identity".cstr()
    }
}

impl FDescription for NPlayerIdentity {
    fn description(&self, _: &Context) -> Cstr {
        self.data
            .as_ref()
            .map(|d| d.cstr())
            .unwrap_or_else(|| "No identity data".cstr())
    }
}

impl FStats for NPlayerIdentity {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NPlayerIdentity {
    fn tag_name(&self, _: &Context) -> Cstr {
        "Identity".cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        self.data.as_ref().map(|_| "✓".cstr())
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        Color32::from_rgb(0, 255, 255)
    }
}

impl FDisplay for NHouseColor {
    fn display(&self, _context: &Context, ui: &mut Ui) {
        Frame::new()
            .fill(self.color.c32())
            .corner_radius(2.0)
            .inner_margin(4.0)
            .show(ui, |ui| {
                ui.label(&self.color.0);
            });
    }
}

impl FEdit for NHouseColor {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.color.edit(context, ui)
    }
}

impl FTitle for NHouseColor {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FTitle for NAbilityDescription {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FTitle for NAbilityEffect {
    fn title(&self, _: &Context) -> Cstr {
        "Ability Effect".cstr()
    }
}

impl FDescription for NAbilityEffect {
    fn description(&self, _: &Context) -> Cstr {
        format!("{} actions", self.actions.len()).cstr()
    }
}

impl FStats for NAbilityEffect {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NAbilityEffect {
    fn tag_name(&self, _: &Context) -> Cstr {
        "Effect".cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        Some(format!("{} actions", self.actions.len()).cstr())
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        Color32::from_rgb(255, 165, 0)
    }
}

impl FRecursive for NAbilityEffect {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        self.actions
            .iter()
            .enumerate()
            .map(|(i, action)| RecursiveField {
                name: format!("Action {}", i),
                value: action.to_recursive_value(),
            })
            .collect()
    }
}

impl FRecursiveMut for NAbilityEffect {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        self.actions
            .iter_mut()
            .enumerate()
            .map(|(i, action)| RecursiveFieldMut {
                name: format!("Action {}", i),
                value: action.to_recursive_value_mut(),
            })
            .collect()
    }
}

impl FTitle for NStatusDescription {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FTitle for NStatusBehavior {
    fn title(&self, _: &Context) -> Cstr {
        "Status Behavior".cstr()
    }
}

impl FDescription for NStatusBehavior {
    fn description(&self, _: &Context) -> Cstr {
        format!("{} reactions", self.reactions.len()).cstr()
    }
}

impl FStats for NStatusBehavior {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NStatusBehavior {
    fn tag_name(&self, _: &Context) -> Cstr {
        "Behavior".cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        Some(format!("{} reactions", self.reactions.len()).cstr())
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        Color32::from_rgb(255, 255, 0)
    }
}

impl FRecursive for NStatusBehavior {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        self.reactions
            .iter()
            .enumerate()
            .map(|(i, reaction)| RecursiveField {
                name: format!("Reaction {}", i),
                value: reaction.to_recursive_value(),
            })
            .collect()
    }
}

impl FRecursiveMut for NStatusBehavior {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        self.reactions
            .iter_mut()
            .enumerate()
            .map(|(i, reaction)| RecursiveFieldMut {
                name: format!("Reaction {}", i),
                value: reaction.to_recursive_value_mut(),
            })
            .collect()
    }
}

impl FTitle for NStatusRepresentation {
    fn title(&self, _: &Context) -> Cstr {
        "Status Representation".cstr()
    }
}

impl FDescription for NStatusRepresentation {
    fn description(&self, _: &Context) -> Cstr {
        self.material.cstr()
    }
}

impl FStats for NStatusRepresentation {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NStatusRepresentation {
    fn tag_name(&self, _: &Context) -> Cstr {
        "Representation".cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        Some(self.material.cstr())
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        Color32::from_rgb(0, 128, 128)
    }
}

impl FRecursive for NStatusRepresentation {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![RecursiveField::named(
            "material",
            self.material.to_recursive_value(),
        )]
    }
}

impl FRecursiveMut for NStatusRepresentation {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        vec![RecursiveFieldMut::named(
            "material",
            self.material.to_recursive_value_mut(),
        )]
    }
}

impl FTitle for NTeam {
    fn title(&self, _: &Context) -> Cstr {
        format!("Team ({}h {}f)", self.houses.len(), self.fusions.len()).cstr()
    }
}

impl FDescription for NTeam {
    fn description(&self, _: &Context) -> Cstr {
        format!(
            "{} houses, {} fusions",
            self.houses.len(),
            self.fusions.len()
        )
        .cstr()
    }
}

impl FStats for NTeam {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NTeam {
    fn tag_name(&self, _: &Context) -> Cstr {
        "Team".cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        Some(format!("{}h {}f", self.houses.len(), self.fusions.len()).cstr())
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        Color32::from_rgb(255, 165, 0)
    }
}

impl FContextMenu for NTeam {
    fn context_actions(&self, _: &Context) -> Vec<ContextAction<Self>> {
        vec![]
    }
}

impl FCopy for NTeam {}
impl FPaste for NTeam {}

impl FDisplay for NTeam {
    fn display(&self, _context: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            if let Some(houses) = self.houses.get_data() {
                ui.label(format!("Houses ({})", houses.len()));
                for house in houses {
                    ui.horizontal(|ui| {
                        ui.label("  •");
                        house.house_name.cstr().label(ui);
                    });
                }
            }
            if let Some(fusions) = self.fusions.get_data() {
                ui.label(format!("Fusions ({})", fusions.len()));
                for fusion in fusions {
                    ui.horizontal(|ui| {
                        ui.label("  •");
                        format!("Fusion #{}", fusion.index).cstr().label(ui);
                    });
                }
            }
        });
    }
}

impl FTitle for NBattle {
    fn title(&self, _: &Context) -> Cstr {
        format!("Battle #{}", self.hash).cstr()
    }
}

impl FDescription for NBattle {
    fn description(&self, _: &Context) -> Cstr {
        if let Some(result) = self.result {
            if result {
                "Victory".cstr_c(Color32::from_rgb(0, 255, 0))
            } else {
                "Defeat".cstr_c(Color32::from_rgb(255, 0, 0))
            }
        } else {
            "In Progress".cstr_c(Color32::from_rgb(255, 255, 0))
        }
    }
}

impl FStats for NBattle {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NBattle {
    fn tag_name(&self, _: &Context) -> Cstr {
        "Battle".cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        self.result.map(|r| {
            if r {
                "✓".cstr_c(Color32::from_rgb(0, 255, 0))
            } else {
                "✗".cstr_c(Color32::from_rgb(255, 0, 0))
            }
        })
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        match self.result {
            Some(true) => Color32::from_rgb(0, 255, 0),
            Some(false) => Color32::from_rgb(255, 0, 0),
            None => Color32::from_rgb(255, 255, 0),
        }
    }
}

impl FContextMenu for NBattle {
    fn context_actions(&self, _: &Context) -> Vec<ContextAction<Self>> {
        vec![]
    }
}

impl FTitle for NMatch {
    fn title(&self, _: &Context) -> Cstr {
        format!("Match F{}", self.floor).cstr()
    }
}

impl FDescription for NMatch {
    fn description(&self, _: &Context) -> Cstr {
        format!(
            "Gold: {}, Floor: {}, Lives: {}",
            self.g, self.floor, self.lives
        )
        .cstr()
    }
}

impl FStats for NMatch {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![
            (VarName::g, VarValue::i32(self.g)),
            (VarName::floor, VarValue::i32(self.floor)),
            (VarName::lives, VarValue::i32(self.lives)),
        ]
    }
}

impl FTag for NMatch {
    fn tag_name(&self, _: &Context) -> Cstr {
        format!("F{}", self.floor).cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        Some(format!("{}g {}❤", self.g, self.lives).cstr())
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        if self.active {
            Color32::from_rgb(0, 255, 0)
        } else {
            Color32::from_rgb(128, 128, 128)
        }
    }
}

impl FContextMenu for NMatch {
    fn context_actions(&self, _: &Context) -> Vec<ContextAction<Self>> {
        vec![]
    }
}

impl FDisplay for NMatch {
    fn display(&self, _context: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            format!("Floor {}", self.floor)
                .cstr_c(Color32::from_rgb(255, 165, 0))
                .label(ui);
            ui.separator();
            format!("Gold: {}", self.g)
                .cstr_c(Color32::from_rgb(255, 255, 0))
                .label(ui);
            ui.separator();
            format!("Lives: {}", self.lives)
                .cstr_c(if self.lives > 0 {
                    Color32::from_rgb(255, 0, 0)
                } else {
                    Color32::from_rgb(128, 128, 128)
                })
                .label(ui);
            ui.separator();
            if self.active {
                "ACTIVE".cstr_c(Color32::from_rgb(0, 255, 0)).label(ui);
            } else {
                "INACTIVE"
                    .cstr_c(Color32::from_rgb(128, 128, 128))
                    .label(ui);
            }
        });
    }
}

impl FEdit for NMatch {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Gold:");
            changed |= ui.add(egui::DragValue::new(&mut self.g)).changed();
            ui.label("Floor:");
            changed |= ui.add(egui::DragValue::new(&mut self.floor)).changed();
            ui.label("Lives:");
            changed |= ui.add(egui::DragValue::new(&mut self.lives)).changed();
            ui.checkbox(&mut self.active, "Active");
        });
        changed
    }
}

impl FTitle for NFusion {
    fn title(&self, _: &Context) -> Cstr {
        format!("Fusion #{}", self.index).cstr()
    }
}

impl FDescription for NFusion {
    fn description(&self, _: &Context) -> Cstr {
        format!("{} slots", self.slots.len()).cstr()
    }
}

impl FStats for NFusion {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![
            (VarName::pwr, VarValue::i32(self.pwr)),
            (VarName::hp, VarValue::i32(self.hp)),
            (VarName::dmg, VarValue::i32(self.dmg)),
        ]
    }
}

impl FTag for NFusion {
    fn tag_name(&self, _: &Context) -> Cstr {
        format!("Fusion #{}", self.index).cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        Some(format!("{}/{}/{}", self.pwr, self.hp, self.dmg).cstr())
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        Color32::from_rgb(128, 0, 128)
    }
}

impl FContextMenu for NFusion {
    fn context_actions(&self, _: &Context) -> Vec<ContextAction<Self>> {
        vec![]
    }
}

impl FCopy for NFusion {}
impl FPaste for NFusion {}

impl FTitle for NFusionSlot {
    fn title(&self, _: &Context) -> Cstr {
        format!("Slot #{}", self.index).cstr()
    }
}

impl FDescription for NFusionSlot {
    fn description(&self, context: &Context) -> Cstr {
        if let Ok(unit) = self.unit_load(context) {
            unit.unit_name.cstr()
        } else {
            "Empty slot".cstr()
        }
    }
}

impl FStats for NFusionSlot {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NFusionSlot {
    fn tag_name(&self, _: &Context) -> Cstr {
        format!("Slot #{}", self.index).cstr()
    }

    fn tag_value(&self, context: &Context) -> Option<Cstr> {
        if let Ok(unit) = self.unit_load(context) {
            Some(unit.unit_name.cstr())
        } else {
            None
        }
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        Color32::from_rgb(128, 0, 128)
    }
}

impl FContextMenu for NFusionSlot {
    fn context_actions(&self, _: &Context) -> Vec<ContextAction<Self>> {
        vec![]
    }
}

impl FDisplay for NFusion {
    fn display(&self, context: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            // Display fusion stats
            ui.horizontal(|ui| {
                ui.label("Power:");
                self.pwr.to_string().cstr_c(Color32::LIGHT_BLUE).label(ui);
                ui.add_space(8.0);
                ui.label("HP:");
                self.hp.to_string().cstr_c(Color32::LIGHT_GREEN).label(ui);
                ui.add_space(8.0);
                ui.label("DMG:");
                self.dmg.to_string().cstr_c(Color32::LIGHT_RED).label(ui);
            });

            ui.horizontal(|ui| {
                ui.label("Actions Limit:");
                self.actions_limit
                    .to_string()
                    .cstr_c(Color32::YELLOW)
                    .label(ui);
                ui.add_space(8.0);
                ui.label("Index:");
                self.index.to_string().cstr_c(Color32::GRAY).label(ui);
            });

            // Display slots
            if let Some(slots) = self.slots.get_data() {
                ui.separator();
                ui.label("Slots:");
                for slot in slots {
                    slot.render(context).display(ui);
                }
            }
        });
    }
}

impl FDisplay for NFusionSlot {
    fn display(&self, context: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            format!("Slot #{}", self.index)
                .cstr_c(Color32::from_rgb(128, 0, 128))
                .label(ui);
            ui.label(":");
            if let Ok(unit) = self.unit_load(context) {
                unit.unit_name.cstr().label(ui);
            } else {
                "Empty".cstr_c(Color32::from_rgb(128, 128, 128)).label(ui);
            }
        });
    }
}

impl FTitle for NUnitDescription {
    fn title(&self, _: &Context) -> Cstr {
        "Unit Description".cstr()
    }
}

impl FDescription for NUnitDescription {
    fn description(&self, _: &Context) -> Cstr {
        self.description.cstr()
    }
}

impl FStats for NUnitDescription {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NUnitDescription {
    fn tag_name(&self, _: &Context) -> Cstr {
        self.magic_type.cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        Some(self.trigger.cstr())
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        self.magic_type.color()
    }
}

impl FDisplay for NUnitDescription {
    fn display(&self, _context: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            self.description.cstr().label_w(ui);
            ui.horizontal(|ui| {
                ui.label("Type:");
                self.magic_type.cstr_c(self.magic_type.color()).label(ui);
                ui.separator();
                ui.label("Trigger:");
                self.trigger.cstr().label(ui);
            });
        });
    }
}

impl FEdit for NUnitDescription {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.vertical(|ui| {
            ui.label("Description:");
            changed |= ui.text_edit_multiline(&mut self.description).changed();
            ui.horizontal(|ui| {
                ui.label("Magic Type:");
                changed |= self.magic_type.show_mut(context, ui);
                ui.separator();
                ui.label("Trigger:");
                changed |= self.trigger.show_mut(context, ui);
            });
        });
        changed
    }
}

impl FTitle for NUnitStats {
    fn title(&self, _: &Context) -> Cstr {
        format!("{}/{}", self.pwr, self.hp).cstr()
    }
}

impl FDescription for NUnitStats {
    fn description(&self, _: &Context) -> Cstr {
        format!("Power: {}, Health: {}", self.pwr, self.hp).cstr()
    }
}

impl FStats for NUnitStats {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![
            (VarName::pwr, VarValue::i32(self.pwr)),
            (VarName::hp, VarValue::i32(self.hp)),
        ]
    }
}

impl FTag for NUnitStats {
    fn tag_name(&self, _: &Context) -> Cstr {
        "Stats".cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        Some(format!("{}/{}", self.pwr, self.hp).cstr())
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        Color32::from_rgb(255, 255, 255)
    }
}

impl FDisplay for NUnitStats {
    fn display(&self, _context: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            format!("PWR: {}", self.pwr)
                .cstr_c(VarName::pwr.color())
                .label(ui);
            ui.separator();
            format!("HP: {}", self.hp)
                .cstr_c(VarName::hp.color())
                .label(ui);
        });
    }
}

impl FEdit for NUnitStats {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Power:");
            changed |= ui.add(egui::DragValue::new(&mut self.pwr)).changed();
            ui.separator();
            ui.label("Health:");
            changed |= ui.add(egui::DragValue::new(&mut self.hp)).changed();
        });
        changed
    }
}

impl FTitle for NUnitState {
    fn title(&self, _: &Context) -> Cstr {
        format!("{}x", self.stacks).cstr()
    }
}

impl FDescription for NUnitState {
    fn description(&self, _: &Context) -> Cstr {
        format!("{} stacks", self.stacks).cstr()
    }
}

impl FStats for NUnitState {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![(VarName::stacks, VarValue::i32(self.stacks))]
    }
}

impl FTag for NUnitState {
    fn tag_name(&self, _: &Context) -> Cstr {
        "State".cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        Some(format!("{}x", self.stacks).cstr())
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        Color32::from_rgb(255, 255, 0)
    }
}

impl FDisplay for NUnitState {
    fn display(&self, _context: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Stacks:");
            format!("{}", self.stacks)
                .cstr_c(Color32::from_rgb(255, 255, 0))
                .label(ui);
        });
    }
}

impl FEdit for NUnitState {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            ui.label("Stacks:");
            ui.add(egui::DragValue::new(&mut self.stacks))
        })
        .inner
        .changed()
    }
}

impl FTitle for NUnitBehavior {
    fn title(&self, _: &Context) -> Cstr {
        self.magic_type.cstr()
    }
}

impl FDescription for NUnitBehavior {
    fn description(&self, _: &Context) -> Cstr {
        self.reaction.cstr()
    }
}

impl FStats for NUnitBehavior {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NUnitBehavior {
    fn tag_name(&self, _: &Context) -> Cstr {
        self.magic_type.cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        Some(format!("T{}", self.reaction.tier()).cstr())
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        self.magic_type.color()
    }
}

impl FContextMenu for NUnitBehavior {
    fn context_actions(&self, _: &Context) -> Vec<ContextAction<Self>> {
        vec![]
    }
}

impl FDisplay for NUnitBehavior {
    fn display(&self, context: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Type:");
                self.magic_type.cstr_c(self.magic_type.color()).label(ui);
            });
            ui.label("Reaction:");
            self.reaction.display(context, ui);
        });
    }
}

impl FEdit for NUnitBehavior {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Magic Type:");
                changed |= self.magic_type.show_mut(context, ui);
            });
            ui.label("Reaction:");
            changed |= self.reaction.show_mut(context, ui);
        });
        changed
    }
}

impl FRecursive for NUnitBehavior {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![RecursiveField::named(
            "reaction",
            self.reaction.to_recursive_value(),
        )]
    }
}

impl FRecursiveMut for NUnitBehavior {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        vec![RecursiveFieldMut::named(
            "reaction",
            self.reaction.to_recursive_value_mut(),
        )]
    }
}

impl FTitle for NUnitRepresentation {
    fn title(&self, _: &Context) -> Cstr {
        "Unit Representation".cstr()
    }
}

impl FDescription for NUnitRepresentation {
    fn description(&self, _: &Context) -> Cstr {
        self.material.cstr()
    }
}

impl FStats for NUnitRepresentation {
    fn stats(&self, _: &Context) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NUnitRepresentation {
    fn tag_name(&self, _: &Context) -> Cstr {
        "Material".cstr()
    }

    fn tag_value(&self, _: &Context) -> Option<Cstr> {
        Some(self.material.cstr())
    }

    fn tag_color(&self, _: &Context) -> Color32 {
        Color32::from_rgb(0, 128, 128)
    }
}

impl FRecursive for NUnitRepresentation {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![RecursiveField::named(
            "material",
            self.material.to_recursive_value(),
        )]
    }
}

impl FRecursiveMut for NUnitRepresentation {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        vec![RecursiveFieldMut::named(
            "material",
            self.material.to_recursive_value_mut(),
        )]
    }
}

// Implement for Vec<T> where appropriate
impl<T: FDisplay> FDisplay for Vec<T> {
    fn display(&self, context: &Context, ui: &mut Ui) {
        for item in self {
            item.display(context, ui);
        }
    }
}

impl<T: FEdit> FEdit for Vec<T> {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        for item in self {
            changed |= item.edit(context, ui);
        }
        changed
    }
}

impl<T: FRecursive + ToRecursiveValue> FRecursive for Vec<T> {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        self.iter()
            .enumerate()
            .map(|(i, item)| RecursiveField {
                name: i.to_string(),
                value: item.to_recursive_value(),
            })
            .collect()
    }
}

impl<T: FRecursiveMut + ToRecursiveValueMut> FRecursiveMut for Vec<T> {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        self.iter_mut()
            .enumerate()
            .map(|(i, item)| RecursiveFieldMut {
                name: i.to_string(),
                value: item.to_recursive_value_mut(),
            })
            .collect()
    }
}

// Implement for Option<T> where appropriate
impl<T: FDisplay> FDisplay for Option<T> {
    fn display(&self, context: &Context, ui: &mut Ui) {
        if let Some(v) = self {
            v.display(context, ui);
        } else {
            "[tw none]".cstr().label(ui);
        }
    }
}

impl<T: FEdit + Default> FEdit for Option<T> {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        let mut is_some = self.is_some();
        if Checkbox::new(&mut is_some, "").ui(ui).changed() {
            if is_some {
                *self = Some(T::default());
            } else {
                *self = None;
            }
            return true;
        }
        if let Some(v) = self {
            FEdit::edit(v, context, ui)
        } else {
            false
        }
    }
}
