use super::*;
use crate::ui::core::enum_colors::EnumColor;

// ============================================================================
// Basic Types Implementations
// ============================================================================

// FTitle implementations for basic types
impl FTitle for i32 {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FTitle for f32 {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FTitle for String {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FTitle for bool {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FTitle for Vec2 {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FTitle for Color32 {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FTitle for HexColor {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

// FDisplay implementations for basic types
impl FDisplay for i32 {
    fn display(&self, _: &ClientContext, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FDisplay for f32 {
    fn display(&self, _: &ClientContext, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FDisplay for String {
    fn display(&self, _: &ClientContext, ui: &mut Ui) -> Response {
        self.cstr().label_t(ui)
    }
}

impl FDisplay for bool {
    fn display(&self, _: &ClientContext, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FDisplay for Vec2 {
    fn display(&self, _: &ClientContext, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FDisplay for Color32 {
    fn display(&self, _: &ClientContext, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FDisplay for HexColor {
    fn display(&self, _: &ClientContext, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

// FEdit implementations for basic types
impl FEdit for i32 {
    fn edit(&mut self, _: &ClientContext, ui: &mut Ui) -> Response {
        DragValue::new(self).ui(ui)
    }
}

impl FEdit for f32 {
    fn edit(&mut self, _: &ClientContext, ui: &mut Ui) -> Response {
        DragValue::new(self).min_decimals(1).ui(ui)
    }
}

impl FEdit for String {
    fn edit(&mut self, _: &ClientContext, ui: &mut Ui) -> Response {
        Input::new("").ui_string(self, ui)
    }
}

impl FEdit for bool {
    fn edit(&mut self, _: &ClientContext, ui: &mut Ui) -> Response {
        Checkbox::new(self, "").ui(ui)
    }
}

impl FEdit for Vec2 {
    fn edit(&mut self, _: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            let rx = DragValue::new(&mut self.x).prefix("x:").ui(ui);
            let ry = DragValue::new(&mut self.y).prefix("y:").ui(ui);
            rx.union(ry)
        })
        .inner
    }
}

impl FEdit for Color32 {
    fn edit(&mut self, _: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            let mut hsva = (*self).into();
            let response = ui.color_edit_button_hsva(&mut hsva);
            if response.changed() {
                *self = hsva.into();
            }
            response
        })
        .inner
    }
}

impl FEdit for HexColor {
    fn edit(&mut self, _: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            let input_id = ui.next_auto_id().with("input");
            let c = self.try_c32().ok();
            let mut response = ui.label("");
            if let Some(c) = c {
                let mut rgb = [c.r(), c.g(), c.b()];
                let color_response = ui.color_edit_button_srgb(&mut rgb);
                if color_response.changed() {
                    *self = Color32::from_rgb(rgb[0], rgb[1], rgb[2]).into();
                }
                response = response.union(color_response);
            }
            let input_response = Input::new("")
                .char_limit(7)
                .desired_width(60.0)
                .color_opt(c)
                .id(input_id)
                .ui_string(&mut self.0, ui);
            response.union(input_response)
        })
        .inner
    }
}

// UnitActionRange
impl FEdit for UnitActionRange {
    fn edit(&mut self, _: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.label("Trigger:");
            let mut response = ui.add(DragValue::new(&mut self.trigger));
            ui.separator();
            ui.label("Start:");
            response = response.union(ui.add(DragValue::new(&mut self.start)));
            ui.separator();
            ui.label("Length:");
            response.union(ui.add(DragValue::new(&mut self.length)))
        })
        .inner
    }
}

// MagicType
impl FEdit for MagicType {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        let (_old_value, response) = Selector::ui_enum(self, ui);
        response
    }
}

// ============================================================================
// Game Types Implementations
// ============================================================================

// VarName
impl FTitle for VarName {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FColoredTitle for VarName {
    fn title_color(&self, _: &ClientContext) -> Color32 {
        self.color()
    }
}

impl FDisplay for VarName {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.colored_title(ctx).label(ui)
    }
}

impl FEdit for VarName {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        let (_old_value, response) = Selector::ui_enum(self, ui);
        response
    }
}

// VarValue
impl FTitle for VarValue {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FDisplay for VarValue {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| match self {
            VarValue::String(v) => v.display(ctx, ui),
            VarValue::i32(v) => v.display(ctx, ui),
            VarValue::f32(v) => v.display(ctx, ui),
            VarValue::u64(v) => v.cstr().label(ui),
            VarValue::bool(v) => v.display(ctx, ui),
            VarValue::Vec2(v) => v.display(ctx, ui),
            VarValue::Color32(v) => v.display(ctx, ui),
            VarValue::Entity(v) => Entity::from_bits(*v).to_string().label(ui),
            VarValue::list(v) => {
                ui.horizontal(|ui| {
                    let resp = "[tw List: ]".cstr().label(ui);
                    for v in v {
                        v.display(ctx, ui);
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
    fn edit(&mut self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        let (_, selector_response) = Selector::ui_enum(self, ui);
        let edit_response = ui
            .horizontal(|ui| match self {
                VarValue::i32(v) => v.edit(ctx, ui),
                VarValue::f32(v) => v.edit(ctx, ui),
                VarValue::u64(v) => DragValue::new(v).ui(ui),
                VarValue::bool(v) => v.edit(ctx, ui),
                VarValue::String(v) => v.edit(ctx, ui),
                VarValue::Vec2(v) => v.edit(ctx, ui),
                VarValue::Color32(v) => v.edit(ctx, ui),
                VarValue::Entity(_) => ui.label("Entity (read-only)"),
                VarValue::list(v) => {
                    let mut response = ui.label("");
                    for v in v {
                        response = response.union(v.edit(ctx, ui));
                    }
                    response
                }
            })
            .inner;
        selector_response.union(edit_response)
    }
}

// Expression
impl FTitle for Expression {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FColoredTitle for Expression {
    fn title_color(&self, _: &ClientContext) -> Color32 {
        self.color()
    }
}

impl FDisplay for Expression {
    fn display(&self, _: &ClientContext, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FEdit for Expression {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        let (old_value, response) = Selector::ui_enum(self, ui);
        if let Some(mut old_value) = old_value {
            self.move_inner_fields_from(&mut old_value);
        }
        response
    }
}

impl FTitle for Trigger {
    fn title(&self, _ctx: &ClientContext) -> Cstr {
        self.as_ref().to_owned()
    }
}

impl FEdit for Trigger {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        Selector::ui_enum(self, ui).1
    }
}

// Action
impl FTitle for Action {
    fn title(&self, ctx: &ClientContext) -> Cstr {
        match self {
            Action::use_ability => {
                let mut r = self.cstr();
                if let Ok(ability) = ctx.get_string(VarName::ability_name) {
                    if let Ok(color) = ctx.get_color(VarName::color) {
                        r += " ";
                        r += &ability.cstr_cs(color, CstrStyle::Bold);
                        if let Ok(lvl) = ctx.get_i32(VarName::lvl) {
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
                if let Ok(status) = ctx.get_string(VarName::status_name) {
                    if let Ok(color) = ctx.get_color(VarName::color) {
                        r += " ";
                        r += &status.cstr_cs(color, CstrStyle::Bold);
                        if let Ok(lvl) = ctx.get_i32(VarName::lvl) {
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
    fn title_color(&self, _: &ClientContext) -> Color32 {
        self.color()
    }
}

impl FDisplay for Action {
    fn display(&self, _: &ClientContext, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FEdit for Action {
    fn edit(&mut self, _: &ClientContext, ui: &mut Ui) -> Response {
        let (old_value, response) = Selector::ui_enum(self, ui);
        if let Some(mut old_val) = old_value {
            self.move_inner_fields_from(&mut old_val);
        }
        response
    }
}

// PainterAction
impl FTitle for PainterAction {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FColoredTitle for PainterAction {
    fn title_color(&self, ctx: &ClientContext) -> Color32 {
        ctx.get_color(VarName::color)
            .unwrap_or(Color32::from_rgb(0, 255, 255))
    }
}

impl FDisplay for PainterAction {
    fn display(&self, _: &ClientContext, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FEdit for PainterAction {
    fn edit(&mut self, _: &ClientContext, ui: &mut Ui) -> Response {
        let (old_value, response) = Selector::ui_enum(self, ui);
        if let Some(mut old_val) = old_value {
            self.move_inner_fields_from(&mut old_val);
        }
        response
    }
}

// FRecursive is implemented in recursive_impl.rs

// Material
impl FTitle for Material {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FDisplay for Material {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.paint_viewer(ctx, ui)
    }
}

impl FEdit for Material {
    fn edit(&mut self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        let paint_response = self.paint_viewer(ctx, ui);
        let edit_response = ui
            .vertical(|ui| {
                let mut response = ui.label("").union(ui.label(""));
                for action in &mut self.0 {
                    response = response.union(action.edit(ctx, ui));
                }
                response
            })
            .inner;
        paint_response.union(edit_response)
    }
}

// FRecursive is implemented in recursive_impl.rs

// Reaction
impl FTitle for Reaction {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FDisplay for Trigger {
    fn display(&self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FDisplay for Reaction {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        let trigger_response = self.trigger.display(ctx, ui);
        let mut actions_response = ui.label("").union(ui.label(""));
        for action in &self.actions {
            actions_response = actions_response.union(action.display(ctx, ui));
        }
        trigger_response | actions_response
    }
}

impl FEdit for Reaction {
    fn edit(&mut self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let trigger_response = ui
                .horizontal(|ui| {
                    ui.label("Trigger:");
                    self.trigger.edit(ctx, ui)
                })
                .inner;
            ui.label("Actions:");
            let mut actions_response = ui.label("").union(ui.label(""));
            for action in &mut self.actions {
                actions_response = actions_response.union(action.edit(ctx, ui));
            }
            trigger_response.union(actions_response)
        })
        .inner
    }
}

// FRecursive is implemented in recursive_impl.rs

// ============================================================================
// Node Implementations
// ============================================================================

// NUnit
impl FTitle for NUnit {
    fn title(&self, ctx: &ClientContext) -> Cstr {
        let color = ctx
            .with_owner(self.id, |ctx| ctx.get_color(VarName::color))
            .unwrap_or(MISSING_COLOR);
        self.unit_name.cstr_c(color)
    }
}

impl FDescription for NUnit {
    fn description(&self, ctx: &ClientContext) -> Cstr {
        if let Ok(description) = self.description_ref(ctx) {
            description.description.clone()
        } else {
            "[tw -]".cstr()
        }
    }
}

impl FStats for NUnit {
    fn stats(&self, ctx: &ClientContext) -> Vec<(VarName, VarValue)> {
        let mut stats = vec![];

        if let Ok(pwr) = ctx.get_var(VarName::pwr) {
            stats.push((VarName::pwr, pwr));
        }
        if let Ok(hp) = ctx.get_var(VarName::hp) {
            stats.push((VarName::hp, hp));
        }
        let tier = if let Ok(behavior) = self.description_ref(ctx).and_then(|d| d.behavior_ref(ctx))
        {
            behavior.reaction.tier()
        } else {
            0
        };
        stats.push((VarName::tier, (tier as i32).into()));
        stats
    }
}

impl FDisplay for NUnit {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.title(ctx).label(ui)
    }
}

impl FTag for NUnit {
    fn tag_name(&self, ctx: &ClientContext) -> Cstr {
        ctx.get_string(VarName::unit_name).unwrap_or_default()
    }

    fn tag_value(&self, ctx: &ClientContext) -> Option<Cstr> {
        let tier = if let Ok(behavior) = self.description_ref(ctx).and_then(|d| d.behavior_ref(ctx))
        {
            behavior.reaction.tier()
        } else {
            0
        };
        let lvl = ctx.get_i32(VarName::lvl).unwrap_or_default();
        let xp = match ctx.get_i32(VarName::xp) {
            Ok(v) => format!(" [tw {v}]/[{} [b {lvl}]]", VarName::lvl.color().to_hex()),
            Err(_) => default(),
        };

        Some(format!(
            "[b {} {} [tw T]{}]{xp}",
            ctx.get_i32(VarName::pwr)
                .unwrap_or_default()
                .cstr_c(VarName::pwr.color()),
            ctx.get_i32(VarName::hp)
                .unwrap_or_default()
                .cstr_c(VarName::hp.color()),
            (tier as i32).cstr_c(VarName::tier.color())
        ))
    }

    fn tag_color(&self, ctx: &ClientContext) -> Color32 {
        ctx.get_color(VarName::color).unwrap_or(MISSING_COLOR)
    }
}

impl FInfo for NUnit {
    fn info(&self, ctx: &ClientContext) -> Cstr {
        let mut info_parts = Vec::new();
        if let Ok(stats) = self.stats_ref(ctx) {
            info_parts.push(format!(
                "[{} {}]/[{} {}]",
                VarName::pwr.color().to_hex(),
                stats.pwr,
                VarName::hp.color().to_hex(),
                stats.hp
            ));
        }
        if let Ok(house) = ctx.load_first_parent::<NHouse>(self.id()) {
            let color = house.color_for_text(ctx);
            info_parts.push(house.house_name.cstr_c(color));
        }
        if let Ok(desc) = self.description_ref(ctx) {
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
        NUnit::new(owner, "New Unit".to_string()).add_components(
            NUnitDescription::placeholder(owner),
            NUnitStats::placeholder(owner),
            NUnitState::placeholder(owner),
        )
    }
}

impl FEdit for NUnit {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            "[tw Unit Name:]".cstr().label(ui);
            ui.text_edit_singleline(&mut self.unit_name)
        })
        .inner
    }
}

// NHouse
impl FTitle for NHouse {
    fn title(&self, ctx: &ClientContext) -> Cstr {
        let color = self.color_for_text(ctx);
        self.house_name.cstr_c(color)
    }
}

impl FDescription for NHouse {
    fn description(&self, _: &ClientContext) -> Cstr {
        String::new()
    }
}

impl FStats for NHouse {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FDisplay for NHouse {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.title(ctx).label(ui)
    }
}

impl FTag for NHouse {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        self.house_name.clone()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        None
    }

    fn tag_color(&self, ctx: &ClientContext) -> Color32 {
        self.color_for_text(ctx)
    }
}

impl FInfo for NHouse {
    fn info(&self, ctx: &ClientContext) -> Cstr {
        let mut info_parts = vec![self.house_name.clone()];
        if let Ok(ability) = self.ability_ref(ctx) {
            info_parts.push(ability.info(ctx));
        }
        if let Ok(status) = self.status_ref(ctx) {
            info_parts.push(status.info(ctx));
        }
        let color = self.color_for_text(ctx);

        info_parts.into_iter().map(|s| s.cstr_c(color)).join(" | ")
    }
}

impl FCopy for NHouse {}
impl FPaste for NHouse {}

impl FPlaceholder for NHouse {
    fn placeholder(owner: u64) -> Self {
        NHouse::new(owner, "New House".to_string()).add_components(
            NHouseColor::placeholder(owner),
            NAbilityMagic::placeholder(owner),
            NStatusMagic::placeholder(owner),
            default(),
        )
    }
}

impl FEdit for NHouse {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            "[tw House Name:]".cstr().label(ui);
            ui.text_edit_singleline(&mut self.house_name)
        })
        .inner
    }
}

// NAbilityMagic
impl FTitle for NAbilityMagic {
    fn title(&self, ctx: &ClientContext) -> Cstr {
        let color = ctx.get_color(VarName::color).unwrap_or(MISSING_COLOR);
        self.ability_name.cstr_c(color)
    }
}

impl FDescription for NAbilityMagic {
    fn description(&self, ctx: &ClientContext) -> Cstr {
        if let Ok(description) = self.description_ref(ctx) {
            description.description.clone()
        } else {
            String::new()
        }
    }
}

impl FStats for NAbilityMagic {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FDisplay for NAbilityMagic {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.title(ctx).label(ui)
    }
}

impl FTag for NAbilityMagic {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        self.ability_name.clone()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        None
    }

    fn tag_color(&self, ctx: &ClientContext) -> Color32 {
        ctx.get_color(VarName::color).unwrap_or(MISSING_COLOR)
    }
}

impl FCopy for NAbilityMagic {}
impl FPaste for NAbilityMagic {}

impl FPlaceholder for NAbilityMagic {
    fn placeholder(owner: u64) -> Self {
        NAbilityMagic::new(owner, "New Ability".to_string())
            .add_components(NAbilityDescription::placeholder(owner))
    }
}

impl FInfo for NAbilityMagic {
    fn info(&self, _ctx: &ClientContext) -> Cstr {
        format!("Ability: {}", self.ability_name).cstr()
    }
}

impl FEdit for NAbilityMagic {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            "[tw Ability Name:]".cstr().label(ui);
            ui.text_edit_singleline(&mut self.ability_name)
        })
        .inner
    }
}

// NStatusMagic
impl FTitle for NStatusMagic {
    fn title(&self, ctx: &ClientContext) -> Cstr {
        let color = ctx.get_color(VarName::color).unwrap_or(MISSING_COLOR);
        self.status_name.cstr_c(color)
    }
}

impl FDescription for NStatusMagic {
    fn description(&self, ctx: &ClientContext) -> Cstr {
        if let Ok(description) = self.description_ref(ctx) {
            description.description.clone()
        } else {
            String::new()
        }
    }
}

impl FStats for NStatusMagic {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FDisplay for NStatusMagic {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.title(ctx).label(ui)
    }
}

impl FTag for NStatusMagic {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        self.status_name.clone()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        None
    }

    fn tag_color(&self, ctx: &ClientContext) -> Color32 {
        ctx.get_color(VarName::color).unwrap_or(MISSING_COLOR)
    }
}

impl FCopy for NStatusMagic {}
impl FPaste for NStatusMagic {}

impl FPlaceholder for NStatusMagic {
    fn placeholder(owner: u64) -> Self {
        NStatusMagic::new(owner, "New Status".to_string()).add_components(
            NStatusDescription::placeholder(owner),
            NStatusRepresentation::placeholder(owner),
        )
    }
}

impl FInfo for NStatusMagic {
    fn info(&self, _ctx: &ClientContext) -> Cstr {
        format!("Status: {}", self.status_name).cstr()
    }
}

impl FEdit for NStatusMagic {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            "[tw Status Name:]".cstr().label(ui);
            ui.text_edit_singleline(&mut self.status_name)
        })
        .inner
    }
}

// Implement FTitle for other node types
impl FTitle for NArena {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Arena".cstr()
    }
}

impl FDescription for NArena {
    fn description(&self, _ctx: &ClientContext) -> Cstr {
        "Arena Description".into()
    }
}

impl FStats for NArena {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FDisplay for NArena {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.title(ctx).label(ui)
    }
}

impl FTag for NArena {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        "Arena".cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        None
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(128, 0, 128)
    }
}

impl FTitle for NFloorPool {
    fn title(&self, _: &ClientContext) -> Cstr {
        format!("Floor {} Pool", self.floor).cstr()
    }
}

impl FDescription for NFloorPool {
    fn description(&self, _: &ClientContext) -> Cstr {
        "Floor Pool".cstr()
    }
}

impl FStats for NFloorPool {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![(VarName::floor, VarValue::i32(self.floor))]
    }
}

impl FDisplay for NFloorPool {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.title(ctx).label(ui)
    }
}

impl FTag for NFloorPool {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        format!("F{}", self.floor).cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        None
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(0, 128, 128)
    }
}

impl FTitle for NFloorBoss {
    fn title(&self, _: &ClientContext) -> Cstr {
        format!("Floor {} Boss", self.floor).cstr()
    }
}

impl FDescription for NFloorBoss {
    fn description(&self, _: &ClientContext) -> Cstr {
        "Boss team".cstr()
    }
}

impl FStats for NFloorBoss {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![(VarName::floor, VarValue::i32(self.floor))]
    }
}

impl FDisplay for NFloorBoss {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.title(ctx).label(ui)
    }
}

impl FTag for NFloorBoss {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        format!("Boss F{}", self.floor).cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        None
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(255, 0, 0)
    }
}

impl FTitle for NPlayer {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.player_name.cstr()
    }
}

impl FDescription for NPlayer {
    fn description(&self, ctx: &ClientContext) -> Cstr {
        if let Ok(data) = self.player_data_ref(ctx) {
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
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NPlayer {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        self.player_name.cstr()
    }

    fn tag_value(&self, ctx: &ClientContext) -> Option<Cstr> {
        if let Ok(data) = self.player_data_ref(ctx) {
            Some(if data.online {
                "●".cstr_c(Color32::from_rgb(0, 255, 0))
            } else {
                "○".cstr_c(Color32::from_rgb(128, 128, 128))
            })
        } else {
            None
        }
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(0, 0, 255)
    }
}

impl FCopy for NPlayer {}
impl FPaste for NPlayer {}

impl FPlaceholder for NPlayer {
    fn placeholder(owner: u64) -> Self {
        NPlayer::new(owner, "New Player".to_string()).add_components(
            NPlayerData::placeholder(owner),
            NPlayerIdentity::placeholder(owner),
            NMatch::placeholder(owner),
        )
    }
}

impl FDisplay for NPlayer {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            let response = self
                .player_name
                .cstr_c(Color32::from_rgb(0, 0, 255))
                .label(ui);
            if let Ok(data) = self.player_data_ref(ctx) {
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
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            "[tw Player Name:]".cstr().label(ui);
            ui.text_edit_singleline(&mut self.player_name)
        })
        .inner
    }
}

impl FTitle for NPlayerData {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Player Data".cstr()
    }
}

impl FDescription for NPlayerData {
    fn description(&self, _: &ClientContext) -> Cstr {
        if self.online {
            format!("Online, last login: {}", self.last_login).cstr()
        } else {
            format!("Offline, last login: {}", self.last_login).cstr()
        }
    }
}

impl FStats for NPlayerData {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FDisplay for NPlayerData {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.title(ctx).label(ui)
    }
}

impl FTag for NPlayerData {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        "Data".cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        Some(if self.online {
            "Online".cstr()
        } else {
            "Offline".cstr()
        })
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        if self.online {
            Color32::from_rgb(0, 255, 0)
        } else {
            Color32::from_rgb(128, 128, 128)
        }
    }
}

impl FTitle for NPlayerIdentity {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Player Identity".cstr()
    }
}

impl FDescription for NPlayerIdentity {
    fn description(&self, _: &ClientContext) -> Cstr {
        self.data
            .as_ref()
            .map(|d| d.cstr())
            .unwrap_or_else(|| "No identity data".cstr())
    }
}

impl FStats for NPlayerIdentity {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FDisplay for NPlayerIdentity {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.title(ctx).label(ui)
    }
}

impl FTag for NPlayerIdentity {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        "Identity".cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        self.data.as_ref().map(|_| "✓".cstr())
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(0, 255, 255)
    }
}

impl FDisplay for NHouseColor {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.color.display(ctx, ui)
    }
}

impl FEdit for NHouseColor {
    fn edit(&mut self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            "[tw Color:]".cstr().label(ui);
            self.color.edit(ctx, ui)
        })
        .inner
    }
}

impl FTitle for NHouseColor {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.color.cstr()
    }
}

impl FPlaceholder for NHouseColor {
    fn placeholder(owner: u64) -> Self {
        NHouseColor::new(owner, HexColor("#FF0000".to_string()))
    }
}

impl FDisplay for NAbilityDescription {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.title(ctx).label(ui)
    }
}

impl FTitle for NAbilityDescription {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FPlaceholder for NAbilityDescription {
    fn placeholder(owner: u64) -> Self {
        NAbilityDescription::new(owner, "Default description".to_string())
            .add_components(NAbilityEffect::placeholder(owner))
    }
}

impl FTitle for NAbilityEffect {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Ability Effect".cstr()
    }
}

impl FDescription for NAbilityEffect {
    fn description(&self, _: &ClientContext) -> Cstr {
        format!("{} actions", self.actions.len()).cstr()
    }
}

impl FStats for NAbilityEffect {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FDisplay for NAbilityEffect {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.title(ctx).label(ui)
    }
}

impl FTag for NAbilityEffect {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        "Effect".cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        Some(format!("{} actions", self.actions.len()).cstr())
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(255, 165, 0)
    }
}

impl FDisplay for NStatusDescription {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.title(ctx).label(ui)
    }
}

impl FTitle for NStatusDescription {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FPlaceholder for NStatusDescription {
    fn placeholder(owner: u64) -> Self {
        NStatusDescription::new(owner, "Default status description".to_string())
            .add_components(NStatusBehavior::placeholder(owner))
    }
}

impl FTitle for NStatusBehavior {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Status Behavior".cstr()
    }
}

impl FDescription for NStatusBehavior {
    fn description(&self, _: &ClientContext) -> Cstr {
        format!("{} reactions", self.reactions.len()).cstr()
    }
}

impl FStats for NStatusBehavior {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FDisplay for NStatusBehavior {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.title(ctx).label(ui)
    }
}

impl FTag for NStatusBehavior {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        "Behavior".cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        Some(format!("{} reactions", self.reactions.len()).cstr())
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(255, 255, 0)
    }
}

impl FDisplay for NStatusRepresentation {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.material.display(ctx, ui)
    }
}

impl FTitle for NStatusRepresentation {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Status Representation".cstr()
    }
}

impl FDescription for NStatusRepresentation {
    fn description(&self, _: &ClientContext) -> Cstr {
        self.material.cstr()
    }
}

impl FStats for NStatusRepresentation {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NStatusRepresentation {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        "Representation".cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        Some(self.material.cstr())
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(0, 128, 128)
    }
}

impl FPlaceholder for NStatusRepresentation {
    fn placeholder(owner: u64) -> Self {
        NStatusRepresentation::new(
            owner,
            Material(vec![PainterAction::circle(Box::new(Expression::f32(0.5)))]),
        )
    }
}

impl FTitle for NTeam {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Team".cstr()
    }
}

impl FDescription for NTeam {
    fn description(&self, ctx: &ClientContext) -> Cstr {
        let houses = self
            .houses
            .iter()
            .map(|h: &NHouse| h.description(ctx))
            .join(", ");
        let fusions = self
            .fusions
            .iter()
            .map(|f: &NFusion| f.description(ctx))
            .join(", ");
        format!("{} houses, {} fusions", houses, fusions).cstr()
    }
}

impl FStats for NTeam {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NTeam {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        "Team".cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        None
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(255, 165, 0)
    }
}

impl FCopy for NTeam {}
impl FPaste for NTeam {}

impl FPlaceholder for NTeam {
    fn placeholder(owner: u64) -> Self {
        NTeam::new(owner)
    }
}

impl FDisplay for NTeam {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            if let Ok(houses) = self.houses_ref(ctx) {
                ui.label(format!("Houses ({})", houses.len()));
                for house in houses {
                    ui.horizontal(|ui| {
                        ui.label("  •");
                        house.house_name.cstr().label(ui);
                    });
                }
            }
            if let Ok(fusions) = self.fusions_ref(ctx) {
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
    fn title(&self, _: &ClientContext) -> Cstr {
        format!("Battle #{}", self.hash).cstr()
    }
}

impl FDescription for NBattle {
    fn description(&self, _: &ClientContext) -> Cstr {
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
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FDisplay for NBattle {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.title(ctx).label(ui)
    }
}

impl FTag for NBattle {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        "Battle".cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        self.result.map(|r| {
            if r {
                "✓".cstr_c(Color32::from_rgb(0, 255, 0))
            } else {
                "✗".cstr_c(Color32::from_rgb(255, 0, 0))
            }
        })
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        match self.result {
            Some(true) => Color32::from_rgb(0, 255, 0),
            Some(false) => Color32::from_rgb(255, 0, 0),
            None => Color32::from_rgb(255, 255, 0),
        }
    }
}

impl FTitle for NMatch {
    fn title(&self, _: &ClientContext) -> Cstr {
        format!("Match F{}", self.floor).cstr()
    }
}

impl FDescription for NMatch {
    fn description(&self, _: &ClientContext) -> Cstr {
        format!(
            "Gold: {}, Floor: {}, Lives: {}",
            self.g, self.floor, self.lives
        )
        .cstr()
    }
}

impl FStats for NMatch {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![
            (VarName::g, VarValue::i32(self.g)),
            (VarName::floor, VarValue::i32(self.floor)),
            (VarName::lives, VarValue::i32(self.lives)),
        ]
    }
}

impl FTag for NMatch {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        format!("F{}", self.floor).cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        Some(format!("{}g {}❤", self.g, self.lives).cstr())
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        if self.active {
            Color32::from_rgb(0, 255, 0)
        } else {
            Color32::from_rgb(128, 128, 128)
        }
    }
}

impl FDisplay for NMatch {
    fn display(&self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
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
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            "[tw Gold:]".cstr().label(ui);
            let mut response = ui.add(egui::DragValue::new(&mut self.g));
            "[tw Floor:]".cstr().label(ui);
            response = response.union(ui.add(egui::DragValue::new(&mut self.floor)));
            "[tw Lives:]".cstr().label(ui);
            response = response.union(ui.add(egui::DragValue::new(&mut self.lives)));
            response.union(ui.checkbox(&mut self.active, "Active"))
        })
        .inner
    }
}

impl FPlaceholder for NMatch {
    fn placeholder(owner: u64) -> Self {
        NMatch::new(owner, 0, 1, 3, false, vec![])
            .add_components(NTeam::placeholder(owner), default())
    }
}

impl FTitle for NFusion {
    fn title(&self, _: &ClientContext) -> Cstr {
        format!("Fusion #{}", self.index).cstr()
    }
}

impl FDescription for NFusion {
    fn description(&self, ctx: &ClientContext) -> Cstr {
        "Fusion Slots".cstr()
    }
}

impl FStats for NFusion {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![
            (VarName::pwr, VarValue::i32(self.pwr)),
            (VarName::hp, VarValue::i32(self.hp)),
            (VarName::dmg, VarValue::i32(self.dmg)),
        ]
    }
}

impl FTag for NFusion {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        format!("Fusion #{}", self.index).cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        Some(format!("{}/{}/{}", self.pwr, self.hp, self.dmg).cstr())
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(128, 0, 128)
    }
}

impl FCopy for NFusion {}
impl FPaste for NFusion {}

impl FPlaceholder for NFusion {
    fn placeholder(owner: u64) -> Self {
        NFusion::new(owner, 1, 0, 0, 1, 1, 1).add_components(
            (0..5)
                .map(|_| NFusionSlot::placeholder(owner))
                .collect_vec(),
        )
    }
}

impl FTitle for NFusionSlot {
    fn title(&self, _: &ClientContext) -> Cstr {
        format!("Slot #{}", self.index).cstr()
    }
}

impl FDescription for NFusionSlot {
    fn description(&self, ctx: &ClientContext) -> Cstr {
        if let Ok(unit) = self.unit_ref(ctx) {
            unit.unit_name.cstr()
        } else {
            "Empty slot".cstr()
        }
    }
}

impl FStats for NFusionSlot {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NFusionSlot {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        format!("Slot #{}", self.index).cstr()
    }

    fn tag_value(&self, ctx: &ClientContext) -> Option<Cstr> {
        if let Ok(unit) = self.unit_ref(ctx) {
            Some(unit.unit_name.cstr())
        } else {
            None
        }
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(128, 0, 128)
    }
}

impl FDisplay for NFusion {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
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
            if let Ok(slots) = self.slots_ref(ctx) {
                ui.separator();
                ui.label("Slots:");
                for slot in slots {
                    response |= slot.display(ctx, ui);
                }
            }
            response
        })
        .inner
    }
}

impl FDisplay for NFusionSlot {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            format!("Slot #{}", self.index)
                .cstr_c(Color32::from_rgb(128, 0, 128))
                .label(ui);
            ui.label(":");
            if let Ok(unit) = self.unit_ref(ctx) {
                unit.unit_name.cstr().label(ui)
            } else {
                "Empty".cstr_c(Color32::from_rgb(128, 128, 128)).label(ui)
            }
        })
        .inner
    }
}

impl FTitle for NUnitDescription {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Unit Description".cstr()
    }
}

impl FDescription for NUnitDescription {
    fn description(&self, _: &ClientContext) -> Cstr {
        self.description.cstr()
    }
}

impl FStats for NUnitDescription {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NUnitDescription {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        self.magic_type.cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        Some(self.trigger.cstr())
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        self.magic_type.color()
    }
}

impl FDisplay for NUnitDescription {
    fn display(&self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
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
    fn edit(&mut self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.group(|ui| {
            let desc_response = ui
                .horizontal(|ui| {
                    "[tw Description:]".cstr().label(ui);
                    ui.text_edit_multiline(&mut self.description)
                })
                .inner;

            let type_response = ui
                .horizontal(|ui| {
                    "[tw Magic Type:]".cstr().label(ui);
                    let magic_response = self.magic_type.edit(ctx, ui);
                    ui.separator();
                    "[tw Trigger:]".cstr().label(ui);
                    let trigger_response = self.trigger.edit(ctx, ui);
                    magic_response.union(trigger_response)
                })
                .inner;

            desc_response.union(type_response)
        })
        .response
    }
}

impl FTitle for NUnitStats {
    fn title(&self, _: &ClientContext) -> Cstr {
        format!("{}/{}", self.pwr, self.hp).cstr()
    }
}

impl FDescription for NUnitStats {
    fn description(&self, _: &ClientContext) -> Cstr {
        format!("Power: {}, Health: {}", self.pwr, self.hp).cstr()
    }
}

impl FStats for NUnitStats {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![
            (VarName::pwr, VarValue::i32(self.pwr)),
            (VarName::hp, VarValue::i32(self.hp)),
        ]
    }
}

impl FTag for NUnitStats {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        "Stats".cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        Some(format!("{}/{}", self.pwr, self.hp).cstr())
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(255, 255, 255)
    }
}

impl FDisplay for NUnitStats {
    fn display(&self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
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
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            "[tw Power:]".cstr().label(ui);
            let pwr_response = ui.add(egui::DragValue::new(&mut self.pwr));
            ui.separator();
            "[tw Health:]".cstr().label(ui);
            let hp_response = ui.add(egui::DragValue::new(&mut self.hp));
            pwr_response.union(hp_response)
        })
        .inner
    }
}

impl FTitle for NUnitState {
    fn title(&self, _: &ClientContext) -> Cstr {
        format!("{}x", self.stacks).cstr()
    }
}

impl FDescription for NUnitState {
    fn description(&self, _: &ClientContext) -> Cstr {
        format!("{} stacks", self.stacks).cstr()
    }
}

impl FStats for NUnitState {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![(VarName::stacks, VarValue::i32(self.stacks))]
    }
}

impl FTag for NUnitState {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        "State".cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        Some(format!("{}x", self.stacks).cstr())
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(255, 255, 0)
    }
}

impl FDisplay for NUnitState {
    fn display(&self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
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
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            "[tw Stacks:]".cstr().label(ui);
            ui.add(egui::DragValue::new(&mut self.stacks))
        })
        .inner
    }
}

impl FTitle for NUnitBehavior {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.magic_type.cstr()
    }
}

impl FDescription for NUnitBehavior {
    fn description(&self, _: &ClientContext) -> Cstr {
        self.reaction.cstr()
    }
}

impl FStats for NUnitBehavior {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NUnitBehavior {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        self.magic_type.cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        Some(format!("T{}", self.reaction.tier()).cstr())
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        self.magic_type.color()
    }
}

impl FInfo for NUnitBehavior {
    fn info(&self, _ctx: &ClientContext) -> Cstr {
        format!("{} {}", self.magic_type.cstr(), self.reaction.cstr())
    }
}

impl FDisplay for NUnitBehavior {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Type:");
                self.magic_type.cstr_c(self.magic_type.color()).label(ui);
            });
            ui.label("Reaction:");
            self.reaction.display(ctx, ui)
        })
        .inner
    }
}

impl FEdit for NUnitBehavior {
    fn edit(&mut self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let magic_response = ui
                .horizontal(|ui| {
                    "[tw Magic Type:]".cstr().label(ui);
                    self.magic_type.edit(ctx, ui)
                })
                .inner;
            "[tw Reaction:]".cstr().label(ui);
            let reaction_response = self.reaction.edit(ctx, ui);
            magic_response.union(reaction_response)
        })
        .inner
    }
}

impl FTitle for NUnitRepresentation {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Unit Representation".cstr()
    }
}

impl FDescription for NUnitRepresentation {
    fn description(&self, _: &ClientContext) -> Cstr {
        self.material.cstr()
    }
}

impl FStats for NUnitRepresentation {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NUnitRepresentation {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        "Material".cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        Some(self.material.cstr())
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(0, 128, 128)
    }
}

impl FDisplay for NUnitRepresentation {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.material.display(ctx, ui)
    }
}

impl FEdit for NUnitRepresentation {
    fn edit(&mut self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.group(|ui| {
            "[tw Material:]".cstr().label(ui);
            self.material.edit(ctx, ui)
        })
        .response
    }
}

// ============================================================================
// Additional FEdit implementations for missing node types
// (Ordered according to raw_nodes.rs struct definitions)
// ============================================================================

impl FEdit for NArena {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.label("Arena")
    }
}

impl FPlaceholder for NArena {
    fn placeholder(owner: u64) -> Self {
        NArena::new(owner)
    }
}

impl FEdit for NFloorPool {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            "[tw Floor:]".cstr().label(ui);
            ui.add(DragValue::new(&mut self.floor))
        })
        .inner
    }
}

impl FPlaceholder for NFloorPool {
    fn placeholder(owner: u64) -> Self {
        NFloorPool::new(owner, 1)
    }
}

impl FEdit for NFloorBoss {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            "[tw Floor:]".cstr().label(ui);
            ui.add(DragValue::new(&mut self.floor))
        })
        .inner
    }
}

impl FPlaceholder for NFloorBoss {
    fn placeholder(owner: u64) -> Self {
        NFloorBoss::new(owner, 1).add_components(NTeam::placeholder(owner))
    }
}

impl FEdit for NPlayerData {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        let mut response = ui.label("");

        response = response.union(
            ui.horizontal(|ui| {
                "[tw Pass Hash:]".cstr().label(ui);
                if let Some(ref mut hash) = self.pass_hash {
                    ui.text_edit_singleline(hash)
                } else {
                    if ui.button("Set Password").clicked() {
                        self.pass_hash = Some("".to_string());
                    }
                    ui.label("No password set")
                }
            })
            .inner,
        );

        response = response.union(
            ui.horizontal(|ui| {
                "[tw Online:]".cstr().label(ui);
                ui.checkbox(&mut self.online, "")
            })
            .inner,
        );

        response = response.union(
            ui.horizontal(|ui| {
                "[tw Last Login:]".cstr().label(ui);
                let mut last_login = self.last_login as i64;
                let drag_response = ui.add(DragValue::new(&mut last_login));
                if drag_response.changed() {
                    self.last_login = last_login as u64;
                }
                drag_response
            })
            .inner,
        );

        response
    }
}

impl FPlaceholder for NPlayerData {
    fn placeholder(owner: u64) -> Self {
        NPlayerData::new(owner, None, true, 0)
    }
}

impl FPlaceholder for NAbilityEffect {
    fn placeholder(owner: u64) -> Self {
        NAbilityEffect::new(owner, vec![Action::noop])
    }
}

impl FEdit for NPlayerIdentity {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            "[tw Identity Data:]".cstr().label(ui);
            if let Some(ref mut data) = self.data {
                ui.text_edit_multiline(data)
            } else {
                if ui.button("Set Identity").clicked() {
                    self.data = Some("".to_string());
                }
                ui.label("No identity set")
            }
        })
        .inner
    }
}

impl FPlaceholder for NPlayerIdentity {
    fn placeholder(owner: u64) -> Self {
        NPlayerIdentity::new(owner, None)
    }
}

impl FPlaceholder for NStatusBehavior {
    fn placeholder(owner: u64) -> Self {
        NStatusBehavior::new(
            owner,
            vec![Reaction {
                trigger: Trigger::BattleStart,
                actions: vec![Action::noop],
            }],
        )
    }
}

impl FEdit for NAbilityDescription {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            "[tw Description:]".cstr().label(ui);
            ui.text_edit_multiline(&mut self.description)
        })
        .inner
    }
}

impl FEdit for NAbilityEffect {
    fn edit(&mut self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            "[tw Actions:]".cstr().label(ui);
            self.actions.edit(ctx, ui)
        })
        .inner
    }
}

impl FEdit for NStatusDescription {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            "[tw Description:]".cstr().label(ui);
            ui.text_edit_multiline(&mut self.description)
        })
        .inner
    }
}

impl FEdit for NStatusBehavior {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.group(|ui| {
            ui.label("Reactions:");
            ui.label(format!("{} reactions configured", self.reactions.len()));
            ui.button("Edit Reactions")
        })
        .response
    }
}

impl FEdit for NStatusRepresentation {
    fn edit(&mut self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.material.edit(ctx, ui)
    }
}

impl FEdit for NTeam {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.label("Team")
    }
}

impl FEdit for NBattle {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.group(|ui| {
            let mut response = ui
                .horizontal(|ui| {
                    ui.label("Team Left:");
                    let mut team_left = self.team_left as i64;
                    let drag_response = ui.add(DragValue::new(&mut team_left));
                    if drag_response.changed() {
                        self.team_left = team_left as u64;
                    }
                    drag_response
                })
                .inner;

            response = response.union(
                ui.horizontal(|ui| {
                    ui.label("Team Right:");
                    let mut team_right = self.team_right as i64;
                    let drag_response = ui.add(DragValue::new(&mut team_right));
                    if drag_response.changed() {
                        self.team_right = team_right as u64;
                    }
                    drag_response
                })
                .inner,
            );

            response = response.union(
                ui.horizontal(|ui| {
                    ui.label("Timestamp:");
                    let mut ts = self.ts as i64;
                    let drag_response = ui.add(DragValue::new(&mut ts));
                    if drag_response.changed() {
                        self.ts = ts as u64;
                    }
                    drag_response
                })
                .inner,
            );

            response = response.union(
                ui.horizontal(|ui| {
                    ui.label("Result:");
                    if let Some(ref mut result) = self.result {
                        ui.checkbox(result, "Won")
                    } else {
                        let button_response = ui.button("Set Result");
                        if button_response.clicked() {
                            self.result = Some(true);
                        }
                        button_response
                    }
                })
                .inner,
            );

            response
        })
        .inner
    }
}

impl FPlaceholder for NBattle {
    fn placeholder(owner: u64) -> Self {
        NBattle::new(owner, 0, 0, 0, 0, None)
    }
}

impl FEdit for NFusion {
    fn edit(&mut self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        let mut response = ui
            .horizontal(|ui| {
                "[tw Trigger Unit:]".cstr().label(ui);
                let mut trigger_unit = self.trigger_unit as i64;
                let drag_response = ui.add(DragValue::new(&mut trigger_unit));
                if drag_response.changed() {
                    self.trigger_unit = trigger_unit as u64;
                }
                drag_response
            })
            .inner;

        response = response.union(
            ui.horizontal(|ui| {
                "[tw Index:]".cstr().label(ui);
                ui.add(DragValue::new(&mut self.index))
            })
            .inner,
        );

        response = response.union(
            ui.horizontal(|ui| {
                "[tw Power:]".cstr().label(ui);
                ui.add(DragValue::new(&mut self.pwr))
            })
            .inner,
        );

        response = response.union(
            ui.horizontal(|ui| {
                "[tw HP:]".cstr().label(ui);
                ui.add(DragValue::new(&mut self.hp))
            })
            .inner,
        );

        response = response.union(
            ui.horizontal(|ui| {
                "[tw Damage:]".cstr().label(ui);
                ui.add(DragValue::new(&mut self.dmg))
            })
            .inner,
        );

        response.union(
            ui.horizontal(|ui| {
                "[tw Actions Limit:]".cstr().label(ui);
                ui.add(DragValue::new(&mut self.actions_limit))
            })
            .inner,
        )
    }
}

impl FPlaceholder for NFusionSlot {
    fn placeholder(owner: u64) -> Self {
        NFusionSlot::new(owner, 0, default()).add_components(NUnit::placeholder(owner))
    }
}

impl FEdit for NFusionSlot {
    fn edit(&mut self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        let response = ui
            .horizontal(|ui| {
                ui.label("Index:");
                ui.add(DragValue::new(&mut self.index))
            })
            .inner;

        response.union(
            ui.horizontal(|ui| {
                ui.label("Actions:");
                self.actions.edit(ctx, ui)
            })
            .inner,
        )
    }
}

// ============================================================================

// Implement for Vec<T> where appropriate
impl<T: FDisplay> FDisplay for Vec<T> {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        let mut response = format!("List ({})", self.len()).label(ui);
        for item in self {
            response |= item.display(ctx, ui);
        }
        response
    }
}

impl<T: FEdit> FEdit for Vec<T> {
    fn edit(&mut self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        let mut response = ui.label("");
        for item in self {
            response = response.union(item.edit(ctx, ui));
        }
        response
    }
}

impl FPlaceholder for NUnitBehavior {
    fn placeholder(owner: u64) -> Self {
        NUnitBehavior::new(
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
        NUnitState::new(owner, 1)
    }
}

impl FPlaceholder for NUnitStats {
    fn placeholder(owner: u64) -> Self {
        NUnitStats::new(owner, 1, 1)
    }
}

impl FPlaceholder for NUnitDescription {
    fn placeholder(owner: u64) -> Self {
        NUnitDescription::new(
            owner,
            "Default Description".to_string(),
            MagicType::Ability,
            Trigger::BattleStart,
        )
        .add_components(
            NUnitRepresentation::placeholder(owner),
            NUnitBehavior::placeholder(owner),
        )
    }
}

// Implement for Option<T> where appropriate
impl<T: FDisplay> FDisplay for Option<T> {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        if let Some(v) = self {
            v.display(ctx, ui)
        } else {
            "[tw none]".cstr().label(ui)
        }
    }
}

impl<T: FEdit + Default> FEdit for Option<T> {
    fn edit(&mut self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        let mut is_some = self.is_some();
        let checkbox_response = Checkbox::new(&mut is_some, "").ui(ui);
        if checkbox_response.changed() {
            if is_some {
                *self = Some(T::default());
            } else {
                *self = None;
            }
        }
        let edit_response = if let Some(v) = self {
            v.edit(ctx, ui)
        } else {
            ui.label("(none)")
        };
        checkbox_response.union(edit_response)
    }
}

impl FEdit for Colorix {
    fn edit(&mut self, _: &ClientContext, ui: &mut Ui) -> Response {
        ui.group(|ui| {
            ui.label("Theme Configuration");

            let mut response = ui
                .horizontal(|ui| {
                    ui.label("Dark Mode:");
                    let mut dark_mode = self.dark_mode();
                    let checkbox_response = ui.checkbox(&mut dark_mode, "");
                    if checkbox_response.changed() {
                        self.set_dark_mode(dark_mode);
                    }
                    checkbox_response
                })
                .inner;

            // Semantic color selectors
            response = response.union(
                ui.vertical(|ui| {
                    let mut semantic_response = ui.label("");
                    semantic_response = semantic_response.union(ui.label("Semantic Colors"));
                    if self.show_semantic_editor(Semantic::Accent, ui) {
                        semantic_response = semantic_response.union(ui.label("Accent changed"));
                    }
                    if self.show_semantic_editor(Semantic::Background, ui) {
                        semantic_response = semantic_response.union(ui.label("Background changed"));
                    }
                    if self.show_semantic_editor(Semantic::Success, ui) {
                        semantic_response = semantic_response.union(ui.label("Success changed"));
                    }
                    if self.show_semantic_editor(Semantic::Error, ui) {
                        semantic_response = semantic_response.union(ui.label("Error changed"));
                    }
                    if self.show_semantic_editor(Semantic::Warning, ui) {
                        semantic_response = semantic_response.union(ui.label("Warning changed"));
                    }
                    semantic_response
                })
                .inner,
            );

            if response.changed() {
                self.apply(ui.ctx());
                self.clone().save();
            }

            response
        })
        .response
    }
}

impl FPlaceholder for NUnitRepresentation {
    fn placeholder(owner: u64) -> Self {
        NUnitRepresentation::new(
            owner,
            Material(vec![PainterAction::circle(Box::new(Expression::f32(0.5)))]),
        )
    }
}

// ============================================================================
// FCompactView Implementations
// ============================================================================

impl FCompactView for Material {
    fn render_compact(&self, ctx: &ClientContext, ui: &mut Ui) {
        let (rect, _) = ui.allocate_exact_size((LINE_HEIGHT * 2.0).v2(), Sense::click());
        self.paint(rect, ctx, ui).ui(ui);
    }

    fn render_hover(&self, ctx: &ClientContext, ui: &mut Ui) {
        self.display(ctx, ui);
        for action in &self.0 {
            action.display(ctx, ui);
        }
    }
}

impl FCompactView for NUnit {
    fn render_compact(&self, ctx: &ClientContext, ui: &mut Ui) {
        let color = ctx.get_color(VarName::color).unwrap_or(MISSING_COLOR);
        self.unit_name.cstr_c(color).label(ui);
    }

    fn render_hover(&self, ctx: &ClientContext, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.strong(format!("Unit: {}", self.unit_name));
            if let Ok(stats) = self.stats_load(ctx) {
                ui.label(format!("Power: {}, HP: {}", stats.pwr, stats.hp));
            }
            if let Ok(desc) = self.description_load(ctx) {
                if !desc.description.is_empty() {
                    ui.separator();
                    desc.description.cstr().label_w(ui);
                }
            }
            if let Ok(house) = ctx.load_first_parent::<NHouse>(self.id()) {
                ui.separator();
                ui.label(format!("House: {}", house.house_name));
            }
        });
    }
}

impl FCompactView for NHouse {
    fn render_compact(&self, ctx: &ClientContext, ui: &mut Ui) {
        let color = self.color_for_text(ctx);
        self.house_name.cstr_c(color).label(ui);
    }

    fn render_hover(&self, ctx: &ClientContext, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.strong(format!("House: {}", self.house_name));
            if let Ok(ability) = self.ability_ref(ctx) {
                ui.label(format!("Ability: {}", ability.ability_name));
            }
            if let Ok(status) = self.status_ref(ctx) {
                ui.label(format!("Status: {}", status.status_name));
            }
        });
    }
}

impl FCompactView for NUnitRepresentation {
    fn render_compact(&self, ctx: &ClientContext, ui: &mut Ui) {
        self.material.render_compact(ctx, ui);
    }

    fn render_hover(&self, ctx: &ClientContext, ui: &mut Ui) {
        self.material.render_hover(ctx, ui);
    }
}

impl FCompactView for NUnitDescription {
    fn render_compact(&self, _ctx: &ClientContext, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.set_max_width(200.0);
            self.description.cstr().label_w(ui);
            self.trigger.cstr().label(ui);
            self.magic_type.cstr().label(ui);
        });
    }

    fn render_hover(&self, _ctx: &ClientContext, ui: &mut Ui) {
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
    fn render_compact(&self, _ctx: &ClientContext, ui: &mut Ui) {
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

    fn render_hover(&self, _ctx: &ClientContext, ui: &mut Ui) {
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
    fn render_compact(&self, _ctx: &ClientContext, ui: &mut Ui) {
        format!(
            "{}[tw /]{}",
            self.pwr.cstr_c(VarName::pwr.color()),
            self.hp.cstr_c(VarName::hp.color())
        )
        .label(ui);
    }

    fn render_hover(&self, _ctx: &ClientContext, ui: &mut Ui) {
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
    fn render_compact(&self, ctx: &ClientContext, ui: &mut Ui) {
        let color = ctx.get_color(VarName::color).unwrap_or(MISSING_COLOR);
        self.ability_name.cstr_c(color).label(ui);
    }

    fn render_hover(&self, ctx: &ClientContext, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.strong(format!("Ability: {}", self.ability_name));
            if let Ok(desc) = self.description_ref(ctx) {
                ui.separator();
                desc.description.cstr().label_w(ui);
            }
        });
    }
}

impl FCompactView for NStatusMagic {
    fn render_compact(&self, ctx: &ClientContext, ui: &mut Ui) {
        let color = ctx.get_color(VarName::color).unwrap_or(MISSING_COLOR);
        self.status_name.cstr_c(color).label(ui);
    }

    fn render_hover(&self, ctx: &ClientContext, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.strong(format!("Status: {}", self.status_name));
            if let Ok(desc) = self.description_ref(ctx) {
                ui.separator();
                desc.description.cstr().label_w(ui);
            }
        });
    }
}

impl FCompactView for NHouseColor {
    fn render_compact(&self, ctx: &ClientContext, ui: &mut Ui) {
        self.title(ctx).label(ui);
    }

    fn render_hover(&self, ctx: &ClientContext, ui: &mut Ui) {
        self.title(ctx).label(ui);
    }
}

impl<T: FCompactView> FCompactView for &T {
    fn render_compact(&self, ctx: &ClientContext, ui: &mut Ui) {
        (*self).render_compact(ctx, ui)
    }

    fn render_hover(&self, ctx: &ClientContext, ui: &mut Ui) {
        (*self).render_hover(ctx, ui)
    }
}

// Colorix implementation
impl FDisplay for Colorix {
    fn display(&self, _: &ClientContext, ui: &mut Ui) -> Response {
        ui.menu_button("Theme".cstr_c(self.color(0)), |ui| {
            "Theme".cstr_c(self.color(0)).label(ui)
        })
        .response
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
