use super::*;
use crate::ui::core::enum_colors::EnumColor;

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
    fn display(&self, _: &Context, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FDisplay for f32 {
    fn display(&self, _: &Context, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FDisplay for String {
    fn display(&self, _: &Context, ui: &mut Ui) -> Response {
        self.cstr().label_t(ui)
    }
}

impl FDisplay for bool {
    fn display(&self, _: &Context, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FDisplay for Vec2 {
    fn display(&self, _: &Context, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FDisplay for Color32 {
    fn display(&self, _: &Context, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FDisplay for HexColor {
    fn display(&self, _: &Context, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
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

// UnitActionRange
impl FEdit for UnitActionRange {
    fn edit(&mut self, _: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Trigger:");
            changed |= ui.add(DragValue::new(&mut self.trigger)).changed();
            ui.separator();
            ui.label("Start:");
            changed |= ui.add(DragValue::new(&mut self.start)).changed();
            ui.separator();
            ui.label("Length:");
            changed |= ui.add(DragValue::new(&mut self.length)).changed();
        });
        changed
    }
}

// MagicType
impl FEdit for MagicType {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let (old_value, _response) = Selector::ui_enum(self, ui);
        old_value.is_some()
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
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.render(context).colored_title(ui)
    }
}

impl FEdit for VarName {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let (old_value, _response) = Selector::ui_enum(self, ui);
        old_value.is_some()
    }
}

// VarValue
impl FTitle for VarValue {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FDisplay for VarValue {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| match self {
            VarValue::String(v) => v.display(context, ui),
            VarValue::i32(v) => v.display(context, ui),
            VarValue::f32(v) => v.display(context, ui),
            VarValue::u64(v) => v.cstr().label(ui),
            VarValue::bool(v) => v.display(context, ui),
            VarValue::Vec2(v) => v.display(context, ui),
            VarValue::Color32(v) => v.display(context, ui),
            VarValue::Entity(v) => Entity::from_bits(*v).to_string().label(ui),
            VarValue::list(v) => {
                ui.horizontal(|ui| {
                    let resp = "[tw List: ]".cstr().label(ui);
                    for v in v {
                        v.display(context, ui);
                    }
                    resp
                })
                .inner
            }
        })
        .inner
    }
}

impl FEdit for VarValue {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        let (old_value, _response) = Selector::ui_enum(self, ui);
        let changed = old_value.is_some();
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
            || changed
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
    fn display(&self, _: &Context, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FEdit for Expression {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        let response = self
            .render_mut(context)
            .with_menu()
            .add_copy()
            .add_paste()
            .edit_selector_recursive(ui);

        response.custom_action().is_some() || response.pasted().is_some()
    }
}

impl FTitle for Trigger {
    fn title(&self, _context: &Context) -> Cstr {
        self.as_ref().to_owned()
    }
}

impl FEdit for Trigger {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let (old_value, _response) = Selector::ui_enum(self, ui);
        old_value.is_some()
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
    fn display(&self, _: &Context, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FEdit for Action {
    fn edit(&mut self, _: &Context, ui: &mut Ui) -> bool {
        let (old_value, _response) = Selector::ui_enum(self, ui);
        if let Some(mut old_val) = old_value {
            self.move_inner_fields_from(&mut old_val);
            return true;
        }
        false
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
    fn display(&self, _: &Context, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FEdit for PainterAction {
    fn edit(&mut self, _: &Context, ui: &mut Ui) -> bool {
        let (old_value, _response) = Selector::ui_enum(self, ui);
        if let Some(mut old_val) = old_value {
            self.move_inner_fields_from(&mut old_val);
            return true;
        }
        false
    }
}

// FRecursive is implemented in recursive_impl.rs

// Material
impl FTitle for Material {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FDisplay for Material {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.paint_viewer(context, ui)
    }
}

impl FEdit for Material {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.paint_viewer(context, ui);
        ui.vertical(|ui| self.0.render_mut(context).edit_recursive_list(ui))
            .inner
    }
}

// FRecursive is implemented in recursive_impl.rs

// Reaction
impl FTitle for Reaction {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FDisplay for Trigger {
    fn display(&self, _context: &Context, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FDisplay for Reaction {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.trigger.display(context, ui) | self.actions.render(context).recursive_list(ui)
    }
}

impl FEdit for Reaction {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Trigger:");
                changed |= self.trigger.edit(context, ui);
            });
            ui.label("Actions:");
            changed |= self.actions.render_mut(context).edit_recursive_list(ui);
        });
        changed
    }
}

// FRecursive is implemented in recursive_impl.rs

// ============================================================================
// Node Implementations
// ============================================================================

// NUnit
impl FTitle for NUnit {
    fn title(&self, context: &Context) -> Cstr {
        let color = context
            .with_owner_ref(self.entity(), |context| context.get_color(VarName::color))
            .unwrap_or(MISSING_COLOR);
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

impl FDisplay for NUnit {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.render(context).title_label(ui)
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

impl FCopy for NUnit {}
impl FPaste for NUnit {}

impl FPlaceholder for NUnit {
    fn placeholder(owner: u64) -> Self {
        NUnit::new_full(
            owner,
            "New Unit".to_string(),
            NUnitDescription::placeholder(owner),
            NUnitStats::placeholder(owner),
            NUnitState::placeholder(owner),
        )
    }
}

impl FEdit for NUnit {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            "[tw Unit Name:]".cstr().label(ui);
            ui.text_edit_singleline(&mut self.unit_name).changed()
        })
        .inner
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

impl FDisplay for NHouse {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.render(context).title_label(ui)
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

impl FCopy for NHouse {}
impl FPaste for NHouse {}

impl FPlaceholder for NHouse {
    fn placeholder(owner: u64) -> Self {
        NHouse::new_full(
            owner,
            "New House".to_string(),
            NHouseColor::placeholder(owner),
            NAbilityMagic::placeholder(owner),
            NStatusMagic::placeholder(owner),
            vec![],
        )
    }
}

impl FEdit for NHouse {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            "[tw House Name:]".cstr().label(ui);
            ui.text_edit_singleline(&mut self.house_name).changed()
        })
        .inner
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

impl FDisplay for NAbilityMagic {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.render(context).title_label(ui)
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

impl FCopy for NAbilityMagic {}
impl FPaste for NAbilityMagic {}

impl FPlaceholder for NAbilityMagic {
    fn placeholder(owner: u64) -> Self {
        NAbilityMagic::new_full(
            owner,
            "New Ability".to_string(),
            NAbilityDescription::placeholder(owner),
        )
    }
}

impl FInfo for NAbilityMagic {
    fn info(&self, _context: &Context) -> Cstr {
        format!("Ability: {}", self.ability_name).cstr()
    }
}

impl FEdit for NAbilityMagic {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            "[tw Ability Name:]".cstr().label(ui);
            ui.text_edit_singleline(&mut self.ability_name).changed()
        })
        .inner
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

impl FDisplay for NStatusMagic {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.render(context).title_label(ui)
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

impl FCopy for NStatusMagic {}
impl FPaste for NStatusMagic {}

impl FPlaceholder for NStatusMagic {
    fn placeholder(owner: u64) -> Self {
        NStatusMagic::new_full(
            owner,
            "New Status".to_string(),
            NStatusDescription::placeholder(owner),
            NStatusRepresentation::placeholder(owner),
        )
    }
}

impl FInfo for NStatusMagic {
    fn info(&self, _context: &Context) -> Cstr {
        format!("Status: {}", self.status_name).cstr()
    }
}

impl FEdit for NStatusMagic {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            "[tw Status Name:]".cstr().label(ui);
            ui.text_edit_singleline(&mut self.status_name).changed()
        })
        .inner
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

impl FDisplay for NArena {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.render(context).title_label(ui)
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

impl FDisplay for NFloorPool {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.render(context).title_label(ui)
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

impl FDisplay for NFloorBoss {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.render(context).title_label(ui)
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

impl FCopy for NPlayer {}
impl FPaste for NPlayer {}

impl FPlaceholder for NPlayer {
    fn placeholder(owner: u64) -> Self {
        NPlayer::new_full(
            owner,
            "New Player".to_string(),
            NPlayerData::placeholder(owner),
            NPlayerIdentity::placeholder(owner),
            NMatch::placeholder(owner),
        )
    }
}

impl FDisplay for NPlayer {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            let response = self
                .player_name
                .cstr_c(Color32::from_rgb(0, 0, 255))
                .label(ui);
            if let Ok(data) = self.player_data_load(context) {
                if data.online {
                    "●".cstr_c(Color32::from_rgb(0, 255, 0)).label(ui);
                } else {
                    "○".cstr_c(Color32::from_rgb(128, 128, 128)).label(ui);
                }
            }
            response
        })
        .inner
    }
}

impl FEdit for NPlayer {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            "[tw Player Name:]".cstr().label(ui);
            ui.text_edit_singleline(&mut self.player_name).changed()
        })
        .inner
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

impl FDisplay for NPlayerData {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.render(context).title_label(ui)
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

impl FDisplay for NPlayerIdentity {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.render(context).title_label(ui)
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
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.color.display(context, ui)
    }
}

impl FEdit for NHouseColor {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            "[tw Color:]".cstr().label(ui);
            self.color.edit(context, ui)
        })
        .inner
    }
}

impl FTitle for NHouseColor {
    fn title(&self, _: &Context) -> Cstr {
        self.color.cstr()
    }
}

impl FPlaceholder for NHouseColor {
    fn placeholder(owner: u64) -> Self {
        NHouseColor::new_full(owner, HexColor("#FF0000".to_string()))
    }
}

impl FDisplay for NAbilityDescription {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.render(context).title_label(ui)
    }
}

impl FTitle for NAbilityDescription {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FPlaceholder for NAbilityDescription {
    fn placeholder(owner: u64) -> Self {
        NAbilityDescription::new_full(
            owner,
            "Default description".to_string(),
            NAbilityEffect::placeholder(owner),
        )
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

impl FDisplay for NAbilityEffect {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.render(context).title_label(ui)
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

impl FDisplay for NStatusDescription {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.render(context).title_label(ui)
    }
}

impl FTitle for NStatusDescription {
    fn title(&self, _: &Context) -> Cstr {
        self.cstr()
    }
}

impl FPlaceholder for NStatusDescription {
    fn placeholder(owner: u64) -> Self {
        NStatusDescription::new_full(
            owner,
            "Default status description".to_string(),
            NStatusBehavior::placeholder(owner),
        )
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

impl FDisplay for NStatusBehavior {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.render(context).title_label(ui)
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

impl FDisplay for NStatusRepresentation {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.material.display(context, ui)
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

impl FPlaceholder for NStatusRepresentation {
    fn placeholder(owner: u64) -> Self {
        NStatusRepresentation::new_full(
            owner,
            Material(vec![PainterAction::circle(Box::new(Expression::f32(0.5)))]),
        )
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

impl FCopy for NTeam {}
impl FPaste for NTeam {}

impl FPlaceholder for NTeam {
    fn placeholder(owner: u64) -> Self {
        NTeam::new_full(owner, vec![], vec![])
    }
}

impl FDisplay for NTeam {
    fn display(&self, _context: &Context, ui: &mut Ui) -> Response {
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
        })
        .response
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

impl FDisplay for NBattle {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.render(context).title_label(ui)
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

impl FDisplay for NMatch {
    fn display(&self, _context: &Context, ui: &mut Ui) -> Response {
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
        })
        .response
    }
}

impl FEdit for NMatch {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            "[tw Gold:]".cstr().label(ui);
            changed |= ui.add(egui::DragValue::new(&mut self.g)).changed();
            "[tw Floor:]".cstr().label(ui);
            changed |= ui.add(egui::DragValue::new(&mut self.floor)).changed();
            "[tw Lives:]".cstr().label(ui);
            changed |= ui.add(egui::DragValue::new(&mut self.lives)).changed();
            ui.checkbox(&mut self.active, "Active");
        });
        changed
    }
}

impl FPlaceholder for NMatch {
    fn placeholder(owner: u64) -> Self {
        NMatch::new_full(
            owner,
            0,
            1,
            3,
            false,
            vec![],
            NTeam::placeholder(owner),
            vec![],
        )
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

impl FCopy for NFusion {}
impl FPaste for NFusion {}

impl FPlaceholder for NFusion {
    fn placeholder(owner: u64) -> Self {
        NFusion::new_full(owner, 1, 0, 0, 1, 1, 1, vec![])
    }
}

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

impl FDisplay for NFusion {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            // Display fusion stats
            let mut response = ui
                .horizontal(|ui| {
                    ui.label("Power:");
                    let mut response = self.pwr.cstr_c(Color32::LIGHT_BLUE).label(ui);
                    ui.add_space(8.0);
                    ui.label("HP:");
                    response |= self.hp.cstr_c(Color32::LIGHT_GREEN).label(ui);
                    ui.add_space(8.0);
                    ui.label("DMG:");
                    response |= self.dmg.cstr_c(Color32::LIGHT_RED).label(ui);
                    response
                })
                .inner;

            ui.horizontal(|ui| {
                ui.label("Actions Limit:");
                response |= self
                    .actions_limit
                    .to_string()
                    .cstr_c(Color32::YELLOW)
                    .label(ui);
                ui.add_space(8.0);
                ui.label("Index:");
                response |= self.index.to_string().cstr_c(Color32::GRAY).label(ui);
            });

            // Display slots
            if let Some(slots) = self.slots.get_data() {
                ui.separator();
                ui.label("Slots:");
                for slot in slots {
                    response |= slot.render(context).display(ui);
                }
            }
            response
        })
        .inner
    }
}

impl FDisplay for NFusionSlot {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            format!("Slot #{}", self.index)
                .cstr_c(Color32::from_rgb(128, 0, 128))
                .label(ui);
            ui.label(":");
            if let Ok(unit) = self.unit_load(context) {
                unit.unit_name.cstr().label(ui)
            } else {
                "Empty".cstr_c(Color32::from_rgb(128, 128, 128)).label(ui)
            }
        })
        .inner
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
    fn display(&self, _context: &Context, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let mut response = self.description.cstr().label_w(ui);
            ui.horizontal(|ui| {
                ui.label("Type:");
                response |= self.magic_type.cstr_c(self.magic_type.color()).label(ui);
                ui.separator();
                ui.label("Trigger:");
                response |= self.trigger.cstr().label(ui);
            });
            response
        })
        .inner
    }
}

impl FEdit for NUnitDescription {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.group(|ui| {
            ui.horizontal(|ui| {
                "[tw Description:]".cstr().label(ui);
                changed |= ui.text_edit_multiline(&mut self.description).changed();
            });
            ui.horizontal(|ui| {
                "[tw Magic Type:]".cstr().label(ui);
                changed |= self.magic_type.edit(context, ui);
                ui.separator();
                "[tw Trigger:]".cstr().label(ui);
                changed |= self.trigger.edit(context, ui);
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
    fn display(&self, _context: &Context, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            let mut response = format!("PWR: {}", self.pwr)
                .cstr_c(VarName::pwr.color())
                .label(ui);
            ui.separator();
            response |= format!("HP: {}", self.hp)
                .cstr_c(VarName::hp.color())
                .label(ui);
            response
        })
        .inner
    }
}

impl FEdit for NUnitStats {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            "[tw Power:]".cstr().label(ui);
            changed |= ui.add(egui::DragValue::new(&mut self.pwr)).changed();
            ui.separator();
            "[tw Health:]".cstr().label(ui);
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
    fn display(&self, _context: &Context, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.label("Stacks:");
            format!("{}", self.stacks)
                .cstr_c(Color32::from_rgb(255, 255, 0))
                .label(ui);
        })
        .response
    }
}

impl FEdit for NUnitState {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            "[tw Stacks:]".cstr().label(ui);
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

impl FInfo for NUnitBehavior {
    fn info(&self, _context: &Context) -> Cstr {
        format!("{} {}", self.magic_type.cstr(), self.reaction.cstr())
    }
}

impl FDisplay for NUnitBehavior {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Type:");
                self.magic_type.cstr_c(self.magic_type.color()).label(ui);
            });
            ui.label("Reaction:");
            self.reaction.display(context, ui)
        })
        .inner
    }
}

impl FEdit for NUnitBehavior {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                "[tw Magic Type:]".cstr().label(ui);
                changed |= self.magic_type.edit(context, ui);
            });
            "[tw Reaction:]".cstr().label(ui);
            changed |= self.reaction.edit(context, ui);
        });
        changed
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

impl FDisplay for NUnitRepresentation {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        self.material.display(context, ui)
    }
}

impl FEdit for NUnitRepresentation {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        ui.group(|ui| {
            "[tw Material:]".cstr().label(ui);
            self.material.edit(context, ui)
        })
        .inner
    }
}

// ============================================================================
// Additional FEdit implementations for missing node types
// (Ordered according to raw_nodes.rs struct definitions)
// ============================================================================

impl FEdit for NArena {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        ui.label("Arena");
        false
    }
}

impl FPlaceholder for NArena {
    fn placeholder(owner: u64) -> Self {
        NArena::new_full(owner, vec![], vec![])
    }
}

impl FEdit for NFloorPool {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            "[tw Floor:]".cstr().label(ui);
            ui.add(DragValue::new(&mut self.floor)).changed()
        })
        .inner
    }
}

impl FPlaceholder for NFloorPool {
    fn placeholder(owner: u64) -> Self {
        NFloorPool::new_full(owner, 1, vec![])
    }
}

impl FEdit for NFloorBoss {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            "[tw Floor:]".cstr().label(ui);
            ui.add(DragValue::new(&mut self.floor)).changed()
        })
        .inner
    }
}

impl FPlaceholder for NFloorBoss {
    fn placeholder(owner: u64) -> Self {
        NFloorBoss::new_full(owner, 1, NTeam::placeholder(owner))
    }
}

impl FEdit for NPlayerData {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            "[tw Pass Hash:]".cstr().label(ui);
            if let Some(ref mut hash) = self.pass_hash {
                changed |= ui.text_edit_singleline(hash).changed();
            } else {
                if ui.button("Set Password").clicked() {
                    self.pass_hash = Some("".to_string());
                    changed = true;
                }
            }
        });

        ui.horizontal(|ui| {
            "[tw Online:]".cstr().label(ui);
            changed |= ui.checkbox(&mut self.online, "").changed();
        });

        ui.horizontal(|ui| {
            "[tw Last Login:]".cstr().label(ui);
            let mut last_login = self.last_login as i64;
            if ui.add(DragValue::new(&mut last_login)).changed() {
                self.last_login = last_login as u64;
                changed = true;
            }
        });

        changed
    }
}

impl FPlaceholder for NPlayerData {
    fn placeholder(owner: u64) -> Self {
        NPlayerData::new_full(owner, None, true, 0)
    }
}

impl FPlaceholder for NAbilityEffect {
    fn placeholder(owner: u64) -> Self {
        NAbilityEffect::new_full(owner, vec![Action::noop])
    }
}

impl FEdit for NPlayerIdentity {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            "[tw Identity Data:]".cstr().label(ui);
            if let Some(ref mut data) = self.data {
                changed |= ui.text_edit_multiline(data).changed();
            } else {
                if ui.button("Set Identity").clicked() {
                    self.data = Some("".to_string());
                    changed = true;
                }
            }
        });

        changed
    }
}

impl FPlaceholder for NPlayerIdentity {
    fn placeholder(owner: u64) -> Self {
        NPlayerIdentity::new_full(owner, None)
    }
}

impl FPlaceholder for NStatusBehavior {
    fn placeholder(owner: u64) -> Self {
        NStatusBehavior::new_full(
            owner,
            vec![Reaction {
                trigger: Trigger::BattleStart,
                actions: vec![Action::noop],
            }],
        )
    }
}

impl FEdit for NAbilityDescription {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            "[tw Description:]".cstr().label(ui);
            ui.text_edit_multiline(&mut self.description).changed()
        })
        .inner
    }
}

impl FEdit for NAbilityEffect {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            "[tw Actions:]".cstr().label(ui);
            self.actions.edit(context, ui)
        })
        .inner
    }
}

impl FEdit for NStatusDescription {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            "[tw Description:]".cstr().label(ui);
            ui.text_edit_multiline(&mut self.description).changed()
        })
        .inner
    }
}

impl FEdit for NStatusBehavior {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let changed = false;

        ui.group(|ui| {
            ui.label("Reactions:");
            // For now, just show count - could be enhanced with reaction editor
            ui.label(format!("{} reactions configured", self.reactions.len()));
            if ui.button("Edit Reactions").clicked() {
                // Could open a detailed reaction editor
            }
        });

        changed
    }
}

impl FEdit for NStatusRepresentation {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        self.material.edit(context, ui)
    }
}

impl FEdit for NTeam {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let _changed = false;
        ui.label("Team");

        false
    }
}

impl FEdit for NBattle {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Team Left:");
                let mut team_left = self.team_left as i64;
                if ui.add(DragValue::new(&mut team_left)).changed() {
                    self.team_left = team_left as u64;
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Team Right:");
                let mut team_right = self.team_right as i64;
                if ui.add(DragValue::new(&mut team_right)).changed() {
                    self.team_right = team_right as u64;
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Timestamp:");
                let mut ts = self.ts as i64;
                if ui.add(DragValue::new(&mut ts)).changed() {
                    self.ts = ts as u64;
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Result:");
                if let Some(ref mut result) = self.result {
                    changed |= ui.checkbox(result, "Won").changed();
                } else {
                    if ui.button("Set Result").clicked() {
                        self.result = Some(true);
                        changed = true;
                    }
                }
            });
        });

        changed
    }
}

impl FPlaceholder for NBattle {
    fn placeholder(owner: u64) -> Self {
        NBattle::new_full(owner, 0, 0, 0, 0, None)
    }
}

impl FEdit for NFusion {
    fn edit(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            "[tw Trigger Unit:]".cstr().label(ui);
            let mut trigger_unit = self.trigger_unit as i64;
            if ui.add(DragValue::new(&mut trigger_unit)).changed() {
                self.trigger_unit = trigger_unit as u64;
                changed = true;
            }
        });
        ui.horizontal(|ui| {
            "[tw Index:]".cstr().label(ui);
            changed |= ui.add(DragValue::new(&mut self.index)).changed();
        });
        ui.horizontal(|ui| {
            "[tw Power:]".cstr().label(ui);
            changed |= ui.add(DragValue::new(&mut self.pwr)).changed();
        });
        ui.horizontal(|ui| {
            "[tw HP:]".cstr().label(ui);
            changed |= ui.add(DragValue::new(&mut self.hp)).changed();
        });
        ui.horizontal(|ui| {
            "[tw Damage:]".cstr().label(ui);
            changed |= ui.add(DragValue::new(&mut self.dmg)).changed();
        });
        ui.horizontal(|ui| {
            "[tw Actions Limit:]".cstr().label(ui);
            changed |= ui.add(DragValue::new(&mut self.actions_limit)).changed();
        });
        changed
    }
}

impl FPlaceholder for NFusionSlot {
    fn placeholder(owner: u64) -> Self {
        NFusionSlot::new_full(owner, 0, default(), NUnit::placeholder(owner))
    }
}

impl FEdit for NFusionSlot {
    fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Index:");
            changed |= ui.add(DragValue::new(&mut self.index)).changed();
        });
        ui.horizontal(|ui| {
            ui.label("Actions:");
            changed |= self.actions.edit(context, ui);
        });
        changed
    }
}

// ============================================================================

// Implement for Vec<T> where appropriate
impl<T: FDisplay> FDisplay for Vec<T> {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        let mut response = format!("List ({})", self.len()).label(ui);
        for item in self {
            response |= item.display(context, ui);
        }
        response
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

impl FPlaceholder for NUnitBehavior {
    fn placeholder(owner: u64) -> Self {
        NUnitBehavior::new_full(
            owner,
            Reaction {
                trigger: Trigger::BattleStart,
                actions: vec![Action::noop],
            },
            MagicType::Ability,
        )
    }
}

impl FPlaceholder for NUnitState {
    fn placeholder(owner: u64) -> Self {
        NUnitState::new_full(owner, 1)
    }
}

impl FPlaceholder for NUnitStats {
    fn placeholder(owner: u64) -> Self {
        NUnitStats::new_full(owner, 1, 1)
    }
}

impl FPlaceholder for NUnitDescription {
    fn placeholder(owner: u64) -> Self {
        NUnitDescription::new_full(
            owner,
            "Default Description".to_string(),
            MagicType::Ability,
            Trigger::BattleStart,
            NUnitRepresentation::placeholder(owner),
            NUnitBehavior::placeholder(owner),
        )
    }
}

// Implement for Option<T> where appropriate
impl<T: FDisplay> FDisplay for Option<T> {
    fn display(&self, context: &Context, ui: &mut Ui) -> Response {
        if let Some(v) = self {
            v.display(context, ui)
        } else {
            "[tw none]".cstr().label(ui)
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

impl FPlaceholder for NUnitRepresentation {
    fn placeholder(owner: u64) -> Self {
        NUnitRepresentation::new_full(
            owner,
            Material(vec![PainterAction::circle(Box::new(Expression::f32(0.5)))]),
        )
    }
}

// ============================================================================
// FCompactView Implementations
// ============================================================================

impl FCompactView for Material {
    fn render_compact(&self, context: &Context, ui: &mut Ui) {
        let (rect, _) = ui.allocate_exact_size((LINE_HEIGHT * 2.0).v2(), Sense::click());
        self.paint(rect, context, ui).ui(ui);
    }

    fn render_hover(&self, context: &Context, ui: &mut Ui) {
        self.display(context, ui);
        self.0.render(context).recursive_list(ui);
    }
}

impl FCompactView for NUnit {
    fn render_compact(&self, context: &Context, ui: &mut Ui) {
        let color = context.get_color(VarName::color).unwrap_or(MISSING_COLOR);
        self.unit_name.cstr_c(color).label(ui);
    }

    fn render_hover(&self, context: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.strong(format!("Unit: {}", self.unit_name));
            if let Ok(stats) = self.stats_load(context) {
                ui.label(format!("Power: {}, HP: {}", stats.pwr, stats.hp));
            }
            if let Ok(desc) = self.description_load(context) {
                if !desc.description.is_empty() {
                    ui.separator();
                    desc.description.cstr().label_w(ui);
                }
            }
            if let Ok(house) = context.first_parent::<NHouse>(self.id()) {
                ui.separator();
                ui.label(format!("House: {}", house.house_name));
            }
        });
    }
}

impl FCompactView for NHouse {
    fn render_compact(&self, context: &Context, ui: &mut Ui) {
        let color = self.color_for_text(context);
        self.house_name.cstr_c(color).label(ui);
    }

    fn render_hover(&self, context: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.strong(format!("House: {}", self.house_name));
            if let Ok(ability) = self.ability_load(context) {
                ui.label(format!("Ability: {}", ability.ability_name));
            }
            if let Ok(status) = self.status_load(context) {
                ui.label(format!("Status: {}", status.status_name));
            }
            let units_count = context
                .collect_children_components::<NUnit>(self.id())
                .map(|u| u.len())
                .unwrap_or_default();
            if units_count > 0 {
                ui.separator();
                ui.label(format!("Units: {}", units_count));
            }
        });
    }
}

impl FCompactView for NUnitRepresentation {
    fn render_compact(&self, context: &Context, ui: &mut Ui) {
        self.material.render_compact(context, ui);
    }

    fn render_hover(&self, context: &Context, ui: &mut Ui) {
        self.material.render_hover(context, ui);
    }
}

impl FCompactView for NUnitDescription {
    fn render_compact(&self, _context: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.set_max_width(200.0);
            self.description.cstr().label_w(ui);
            self.trigger.cstr().label(ui);
            self.magic_type.cstr().label(ui);
        });
    }

    fn render_hover(&self, _context: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.strong("Unit Description");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Type:");
                self.magic_type.cstr_c(self.magic_type.color()).label(ui);
            });
            ui.separator();
            self.description.cstr().label_w(ui);
        });
    }
}

impl FCompactView for NUnitBehavior {
    fn render_compact(&self, _context: &Context, ui: &mut Ui) {
        let actions_count = self.reaction.actions.len();
        let tier = self.reaction.tier();

        ui.horizontal(|ui| {
            format!("{} actions", actions_count)
                .cstr_c(self.magic_type.color())
                .label(ui);
            ui.add_space(4.0);
            format!("T{} [{}]", tier, self.magic_type.as_ref())
                .cstr()
                .label(ui);
        });
    }

    fn render_hover(&self, _context: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.strong("Unit Behavior");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Type:");
                self.magic_type.cstr_c(self.magic_type.color()).label(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Trigger:");
                self.reaction.trigger.cstr().label(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Tier:");
                format!("{}", self.reaction.tier())
                    .cstr_c(VarName::tier.color())
                    .label(ui);
            });
            ui.separator();
            ui.label(format!("Actions ({})", self.reaction.actions.len()));
            for (i, action) in self.reaction.actions.iter().take(3).enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}.", i + 1));
                    action.cstr().label(ui);
                });
            }
            if self.reaction.actions.len() > 3 {
                ui.label(format!("... and {} more", self.reaction.actions.len() - 3));
            }
        });
    }
}

impl FCompactView for NUnitStats {
    fn render_compact(&self, _context: &Context, ui: &mut Ui) {
        format!(
            "{}[tw /]{}",
            self.pwr.cstr_c(VarName::pwr.color()),
            self.hp.cstr_c(VarName::hp.color())
        )
        .label(ui);
    }

    fn render_hover(&self, _context: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.strong("Unit Stats");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Power:");
                format!("{}", self.pwr)
                    .cstr_c(VarName::pwr.color())
                    .label(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Health:");
                format!("{}", self.hp).cstr_c(VarName::hp.color()).label(ui);
            });
        });
    }
}

impl FCompactView for NAbilityMagic {
    fn render_compact(&self, context: &Context, ui: &mut Ui) {
        let color = context.get_color(VarName::color).unwrap_or(MISSING_COLOR);
        self.ability_name.cstr_c(color).label(ui);
    }

    fn render_hover(&self, context: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.strong(format!("Ability: {}", self.ability_name));
            if let Ok(desc) = self.description_load(context) {
                ui.separator();
                desc.description.cstr().label_w(ui);
            }
        });
    }
}

impl FCompactView for NStatusMagic {
    fn render_compact(&self, context: &Context, ui: &mut Ui) {
        let color = context.get_color(VarName::color).unwrap_or(MISSING_COLOR);
        self.status_name.cstr_c(color).label(ui);
    }

    fn render_hover(&self, context: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.strong(format!("Status: {}", self.status_name));
            if let Ok(desc) = self.description_load(context) {
                ui.separator();
                desc.description.cstr().label_w(ui);
            }
        });
    }
}

impl FCompactView for NHouseColor {
    fn render_compact(&self, context: &Context, ui: &mut Ui) {
        self.render(context).title_label(ui);
    }

    fn render_hover(&self, context: &Context, ui: &mut Ui) {
        self.render(context).title_label(ui);
    }
}

impl<T: FCompactView> FCompactView for &T {
    fn render_compact(&self, context: &Context, ui: &mut Ui) {
        (*self).render_compact(context, ui)
    }

    fn render_hover(&self, context: &Context, ui: &mut Ui) {
        (*self).render_hover(context, ui)
    }
}

// Colorix implementation
impl FDisplay for Colorix {
    fn display(&self, _: &Context, ui: &mut Ui) -> Response {
        ui.menu_button("Theme".cstr_c(self.color(0)), |ui| {
            "Theme".cstr_c(self.color(0)).label(ui)
        })
        .response
    }
}

impl FEdit for Colorix {
    fn edit(&mut self, _: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.group(|ui| {
            ui.label("Theme Configuration");

            ui.horizontal(|ui| {
                ui.label("Dark Mode:");
                let mut dark_mode = self.dark_mode();
                if ui.checkbox(&mut dark_mode, "").changed() {
                    self.set_dark_mode(dark_mode);
                    changed = true;
                }
            });

            // Semantic color selectors
            ui.vertical(|ui| {
                changed |= self.show_semantic_editor(Semantic::Accent, ui);
                changed |= self.show_semantic_editor(Semantic::Background, ui);
                changed |= self.show_semantic_editor(Semantic::Success, ui);
                changed |= self.show_semantic_editor(Semantic::Error, ui);
                changed |= self.show_semantic_editor(Semantic::Warning, ui);
            });
        });

        if changed {
            self.apply(ui.ctx());
            self.clone().save();
        }

        changed
    }
}

impl FCard for NUnit {
    fn render_card(&self, ui: &mut Ui, size: egui::Vec2) -> Response {
        let rect = egui::Rect::from_min_size(ui.next_widget_position(), size);
        let response = ui.allocate_rect(rect, egui::Sense::hover());

        ui.painter()
            .rect_filled(rect, 5.0, egui::Color32::from_gray(40));
        ui.painter().rect_stroke(
            rect,
            5.0,
            egui::Stroke::new(1.0, egui::Color32::WHITE),
            egui::StrokeKind::Middle,
        );

        let text_rect = rect.shrink(10.0);
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(text_rect), |ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(&self.unit_name)
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.label(format!("ID: {}", self.id()));
                ui.label(format!("Type: NUnit"));
            });
        });

        response
    }
}

impl FCard for NHouse {
    fn render_card(&self, ui: &mut Ui, size: egui::Vec2) -> Response {
        let rect = egui::Rect::from_min_size(ui.next_widget_position(), size);
        let response = ui.allocate_rect(rect, egui::Sense::hover());

        ui.painter()
            .rect_filled(rect, 5.0, egui::Color32::from_gray(40));
        ui.painter().rect_stroke(
            rect,
            5.0,
            egui::Stroke::new(1.0, egui::Color32::WHITE),
            egui::StrokeKind::Middle,
        );

        let text_rect = rect.shrink(10.0);
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(text_rect), |ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(&self.house_name)
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.label(format!("ID: {}", self.id()));
                ui.label(format!("Type: NHouse"));
            });
        });

        response
    }
}

impl FCard for NAbilityMagic {
    fn render_card(&self, ui: &mut Ui, size: egui::Vec2) -> Response {
        let rect = egui::Rect::from_min_size(ui.next_widget_position(), size);
        let response = ui.allocate_rect(rect, egui::Sense::hover());

        ui.painter()
            .rect_filled(rect, 5.0, egui::Color32::from_gray(40));
        ui.painter().rect_stroke(
            rect,
            5.0,
            egui::Stroke::new(1.0, egui::Color32::WHITE),
            egui::StrokeKind::Middle,
        );

        let text_rect = rect.shrink(10.0);
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(text_rect), |ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(&self.ability_name)
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.label(format!("ID: {}", self.id()));
                ui.label(format!("Type: NAbilityMagic"));
            });
        });

        response
    }
}

impl FCard for NStatusMagic {
    fn render_card(&self, ui: &mut Ui, size: egui::Vec2) -> Response {
        let rect = egui::Rect::from_min_size(ui.next_widget_position(), size);
        let response = ui.allocate_rect(rect, egui::Sense::hover());

        ui.painter()
            .rect_filled(rect, 5.0, egui::Color32::from_gray(40));
        ui.painter().rect_stroke(
            rect,
            5.0,
            egui::Stroke::new(1.0, egui::Color32::WHITE),
            egui::StrokeKind::Middle,
        );

        let text_rect = rect.shrink(10.0);
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(text_rect), |ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(&self.status_name)
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.label(format!("ID: {}", self.id()));
                ui.label(format!("Type: NStatusMagic"));
            });
        });

        response
    }
}
