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
    fn edit(&mut self, ui: &mut Ui) -> Response {
        DragValue::new(self).ui(ui)
    }
}

impl FEdit for f32 {
    fn edit(&mut self, ui: &mut Ui) -> Response {
        DragValue::new(self).min_decimals(1).ui(ui)
    }
}

impl FEdit for String {
    fn edit(&mut self, ui: &mut Ui) -> Response {
        Input::new("").ui_string(self, ui)
    }
}

impl FEdit for u8 {
    fn edit(&mut self, ui: &mut Ui) -> Response {
        DragValue::new(self).ui(ui)
    }
}

impl FEdit for u64 {
    fn edit(&mut self, ui: &mut Ui) -> Response {
        DragValue::new(self).ui(ui)
    }
}

impl FEdit for bool {
    fn edit(&mut self, ui: &mut Ui) -> Response {
        Checkbox::new(self, "").ui(ui)
    }
}

impl FEdit for Vec2 {
    fn edit(&mut self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            let rx = DragValue::new(&mut self.x).prefix("x:").ui(ui);
            let ry = DragValue::new(&mut self.y).prefix("y:").ui(ui);
            rx.union(ry)
        })
        .inner
    }
}

impl FEdit for Color32 {
    fn edit(&mut self, ui: &mut Ui) -> Response {
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
    fn edit(&mut self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            let input_id = ui.next_auto_id().with("input");
            let c = self.try_c32().ok();
            let color_response = if let Some(c) = c {
                let mut rgb = [c.r(), c.g(), c.b()];
                let color_response = ui.color_edit_button_srgb(&mut rgb);
                if color_response.changed() {
                    *self = Color32::from_rgb(rgb[0], rgb[1], rgb[2]).into();
                }
                Some(color_response)
            } else {
                None
            };
            let input_response = Input::new("")
                .char_limit(7)
                .desired_width(60.0)
                .color_opt(c)
                .id(input_id)
                .ui_string(&mut self.0, ui);
            if let Some(color_response) = color_response {
                input_response | color_response
            } else {
                input_response
            }
        })
        .inner
    }
}

// UnitActionRange
impl FEdit for UnitActionRange {
    fn edit(&mut self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.label("Start:");
            let response = ui.add(DragValue::new(&mut self.start));
            ui.separator();
            ui.label("Length:");
            response.union(ui.add(DragValue::new(&mut self.length)))
        })
        .inner
    }
}

impl FEdit for MatchState {
    fn edit(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui.label("MatchState");
        egui::ComboBox::from_id_salt("match_state")
            .selected_text(format!("{:?}", self))
            .show_ui(ui, |ui| {
                ui.selectable_value(self, MatchState::Shop, "Shop");
                ui.selectable_value(self, MatchState::RegularBattle, "Regular Battle");
                ui.selectable_value(self, MatchState::BossBattle, "Boss Battle");
                ui.selectable_value(self, MatchState::ChampionBattle, "Champion Battle");
            });
        response
    }
}

impl FEdit for MagicType {
    fn edit(&mut self, ui: &mut Ui) -> Response {
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
    fn edit(&mut self, ui: &mut Ui) -> Response {
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
    fn edit(&mut self, ui: &mut Ui) -> Response {
        let (_, mut response) = Selector::ui_enum(self, ui);
        ui.horizontal(|ui| match self {
            VarValue::i32(v) => response |= v.edit(ui),
            VarValue::f32(v) => response |= v.edit(ui),
            VarValue::u64(v) => response |= DragValue::new(v).ui(ui),
            VarValue::bool(v) => response |= v.edit(ui),
            VarValue::String(v) => response |= v.edit(ui),
            VarValue::Vec2(v) => response |= v.edit(ui),
            VarValue::Color32(v) => response |= v.edit(ui),
            VarValue::list(v) => {
                for v in v {
                    response |= v.edit(ui);
                }
            }
        });
        response
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
    fn edit(&mut self, ui: &mut Ui) -> Response {
        let (old_value, mut response) = Selector::ui_enum(self, ui);
        const DEBUG_TXT: &str = "🪲 Debug";
        const SCALE_TXT: &str = "⚖️ Scale";
        let menu_response = self
            .as_empty_mut()
            .with_menu()
            .add_copy()
            .add_paste()
            .add_action_empty(DEBUG_TXT)
            .add_action_empty(SCALE_TXT)
            .compose_with_menu(&EMPTY_CONTEXT, ui);
        if let Some(d) = menu_response.custom_action() {
            if d.eq(DEBUG_TXT) {
                *self = Expression::dbg(self.clone().into());
                response.mark_changed();
            } else if d.eq(SCALE_TXT) {
                *self = Expression::mul(self.clone().into(), Expression::f32(0.5).into());
            }
        } else if let Some(value) = menu_response.pasted() {
            *self = value.clone();
            response.mark_changed();
        }
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
    fn edit(&mut self, ui: &mut Ui) -> Response {
        let resp = Selector::ui_enum(self, ui).1;
        match self {
            Trigger::BattleStart
            | Trigger::TurnEnd
            | Trigger::BeforeDeath
            | Trigger::BeforeStrike
            | Trigger::AfterStrike
            | Trigger::DamageTaken
            | Trigger::DamageDealt
            | Trigger::StatusApplied
            | Trigger::StatusGained
            | Trigger::ChangeOutgoingDamage
            | Trigger::ChangeIncomingDamage
            | Trigger::AllyDeath => resp,
            Trigger::ChangeStat(var) => var.edit(ui) | resp,
        }
    }
}

// Action
impl FTitle for Action {
    fn title(&self, ctx: &ClientContext) -> Cstr {
        fn x_text(ctx: &ClientContext, mut x: i32) -> NodeResult<String> {
            let owner = ctx.owner()?;
            let house = ctx.load_first_parent_recursive_ref::<NHouse>(owner)?;
            let house_x = house.state_ref(ctx)?.stax;
            if house_x > 0 {
                x = x.at_most(house_x);
            }
            Ok(format!(" [{} x{x}]", VarName::stax.color().to_hex()))
        }
        match self {
            Action::use_ability => {
                let mut r = self.cstr();
                if let Ok(ability) = ctx.get_var(VarName::ability_name).get_string() {
                    if let Ok(color) = ctx.get_var(VarName::color).get_color() {
                        if let Ok(x) = ctx.get_var(VarName::stax).get_i32() {
                            r += &x_text(ctx, x).unwrap_or_default();
                        }
                        r += " ";
                        r += &ability.cstr_cs(color, CstrStyle::Bold);
                    }
                }
                r
            }
            Action::apply_status => {
                let mut r = self.cstr();
                if let Ok(status) = ctx.get_var(VarName::status_name).get_string() {
                    if let Ok(color) = ctx.get_var(VarName::color).get_color() {
                        if let Ok(x) = ctx.get_var(VarName::stax).get_i32() {
                            r += &x_text(ctx, x).unwrap_or_default();
                        }
                        r += " ";
                        r += &status.cstr_c(color);
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
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.title(ctx).label(ui)
    }
}

impl FEdit for Action {
    fn edit(&mut self, ui: &mut Ui) -> Response {
        self.as_recursive_mut(|_, ui, v| call_on_recursive_value_mut!(v, edit_self, ui))
            .with_layout(RecursiveLayout::Tree { indent: 0.0 })
            .compose(&EMPTY_CONTEXT, ui)
    }
    fn edit_self(&mut self, ui: &mut Ui) -> Response {
        let (old_value, response) = Selector::ui_enum(self, ui);
        let menu_resp = self
            .as_empty_mut()
            .with_menu()
            .add_copy()
            .add_paste()
            .compose_with_menu(&EMPTY_CONTEXT, ui);
        if let Some(value) = menu_resp.pasted() {
            *self = value.clone();
        }
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
        ctx.get_var(VarName::color)
            .get_color()
            .unwrap_or(Color32::from_rgb(0, 255, 255))
    }
}

impl FDisplay for PainterAction {
    fn display(&self, _: &ClientContext, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FEdit for PainterAction {
    fn edit(&mut self, ui: &mut Ui) -> Response {
        self.as_recursive_mut(|_, ui, v| call_on_recursive_value_mut!(v, edit_self, ui))
            .with_layout(RecursiveLayout::Tree { indent: 0.0 })
            .compose(&EMPTY_CONTEXT, ui)
    }
    fn edit_self(&mut self, ui: &mut Ui) -> Response {
        let (old_value, response) = Selector::ui_enum(self, ui);
        let menu_resp = self
            .as_empty_mut()
            .with_menu()
            .add_copy()
            .add_paste()
            .compose_with_menu(&EMPTY_CONTEXT, ui);
        if let Some(value) = menu_resp.pasted() {
            *self = value.clone();
        }
        if let Some(mut old_val) = old_value {
            self.move_inner_fields_from(&mut old_val);
        }
        response
    }
}

impl FTitle for Material {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FDescription for Material {
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr {
        self.0.iter().map(|p| p.title_recursive(ctx)).join("\n")
    }
}

impl FDisplay for Material {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.paint_viewer(ctx, ui)
    }
}

impl FEdit for Material {
    fn edit(&mut self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let mut response = self.paint_viewer(&EMPTY_CONTEXT, ui);
            response |= self.0.edit(ui);
            response
        })
        .inner
    }
}

impl FTitle for Reaction {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FDescription for Reaction {
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr {
        format!(
            "{}:\n{}",
            self.trigger.cstr(),
            self.actions
                .iter()
                .map(|a| a.title_recursive(ctx))
                .join("\n")
        )
    }
}

impl FDisplay for Trigger {
    fn display(&self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FDisplay for Reaction {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.group(|ui| {
            ui.vertical(|ui| {
                let mut response = self.trigger.display(ctx, ui);
                for action in &self.actions {
                    response |= action.display(ctx, ui);
                }
                response
            })
            .inner
        })
        .inner
    }
}

impl FEdit for Reaction {
    fn edit(&mut self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let response = ui
                .horizontal(|ui| {
                    ui.label("Trigger:");
                    self.trigger.edit(ui)
                })
                .inner;
            ui.label("Actions:");
            self.actions.edit(ui).union(response)
        })
        .inner
    }
}

impl FTitle for NUnit {
    fn title(&self, ctx: &ClientContext) -> Cstr {
        ctx.exec_ref(|ctx| {
            let color = ctx
                .with_owner(self.id, |ctx| Ok(ctx.color()))
                .unwrap_or(MISSING_COLOR);
            Ok(self.unit_name.cstr_c(color))
        })
        .unwrap()
    }
}

impl FDescription for NUnit {
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr {
        if let Ok(description) = self.description_ref(ctx) {
            description.description_expanded_cstr(ctx)
        } else {
            "[tw -]".cstr()
        }
    }
}

impl FStats for NUnit {
    fn stats(&self, ctx: &ClientContext) -> Vec<(VarName, VarValue)> {
        let mut stats = vec![];
        if let Ok(pwr) = ctx.get_var_inherited(self.id, VarName::pwr) {
            stats.push((VarName::pwr, pwr));
        }
        if let Ok(hp) = ctx.get_var_inherited(self.id, VarName::hp) {
            stats.push((VarName::hp, hp));
        }
        if let Ok(stax) = ctx.get_var_inherited(self.id, VarName::stax) {
            if let Ok(house_state) = ctx
                .load_first_parent_ref::<NHouse>(self.id)
                .and_then(|h| h.state_ref(ctx))
            {
                let house_x = house_state.stax;
                let unit_x = stax.get_i32().unwrap();
                let color = if unit_x > house_x {
                    "red"
                } else if unit_x < house_x {
                    "green"
                } else {
                    "yellow"
                };
                stats.push((
                    VarName::stax,
                    format!("{}/[{color} {}]", unit_x, house_x).into(),
                ));
            } else {
                stats.push((VarName::stax, stax));
            }
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
        ctx.get_var(VarName::unit_name)
            .get_string()
            .unwrap_or_default()
    }

    fn tag_value(&self, ctx: &ClientContext) -> Option<Cstr> {
        let tier = if let Ok(behavior) = self.description_ref(ctx).and_then(|d| d.behavior_ref(ctx))
        {
            behavior.reaction.tier()
        } else {
            0
        };
        let x = ctx.get_var(VarName::stax).get_i32().unwrap_or_default();

        Some(format!(
            "[b {}/{} [tw T]{}] [b x{x}]",
            ctx.get_var(VarName::pwr)
                .get_i32()
                .unwrap_or_default()
                .cstr_c(VarName::pwr.color()),
            ctx.get_var(VarName::hp)
                .get_i32()
                .unwrap_or_default()
                .cstr_c(VarName::hp.color()),
            (tier as i32).cstr_c(VarName::tier.color())
        ))
    }

    fn tag_color(&self, ctx: &ClientContext) -> Color32 {
        ctx.get_var(VarName::color)
            .get_color()
            .unwrap_or(MISSING_COLOR)
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
        if let Ok(house) = ctx.load_first_parent_ref::<NHouse>(self.id()) {
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
    fn placeholder() -> Self {
        NUnit::new(next_id(), player_id(), "New Unit".to_string())
            .with_description(NUnitDescription::placeholder())
            .with_stats(NUnitStats::placeholder())
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
    fn description_cstr(&self, _: &ClientContext) -> Cstr {
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

    fn tag_value(&self, ctx: &ClientContext) -> Option<Cstr> {
        Some(format!(
            "[{} [b x{}]]",
            VarName::stax.color().to_hex(),
            self.state_ref(ctx).ok()?.stax
        ))
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
    fn placeholder() -> Self {
        NHouse::new(next_id(), player_id(), "New House".to_string())
            .with_color(NHouseColor::placeholder())
            .with_units([NUnit::placeholder()].into())
    }
}

// NAbilityMagic
impl FTitle for NAbilityMagic {
    fn title(&self, ctx: &ClientContext) -> Cstr {
        let color = ctx
            .get_var(VarName::color)
            .get_color()
            .unwrap_or(MISSING_COLOR);
        self.ability_name.cstr_c(color)
    }
}

impl FDescription for NAbilityMagic {
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr {
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
        ctx.get_var(VarName::color)
            .get_color()
            .unwrap_or(MISSING_COLOR)
    }
}

impl FCopy for NAbilityMagic {}
impl FPaste for NAbilityMagic {}

impl FPlaceholder for NAbilityMagic {
    fn placeholder() -> Self {
        NAbilityMagic::new(next_id(), player_id(), "New Ability".to_string())
            .with_description(NAbilityDescription::placeholder())
    }
}

impl FInfo for NAbilityMagic {
    fn info(&self, _ctx: &ClientContext) -> Cstr {
        format!("Ability: {}", self.ability_name).cstr()
    }
}

// NStatusMagic
impl FTitle for NStatusMagic {
    fn title(&self, ctx: &ClientContext) -> Cstr {
        let color = ctx
            .get_var(VarName::color)
            .get_color()
            .unwrap_or(MISSING_COLOR);
        self.status_name.cstr_c(color)
    }
}

impl FDescription for NStatusMagic {
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr {
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
        ctx.get_var(VarName::color)
            .get_color()
            .unwrap_or(MISSING_COLOR)
    }
}

impl FCopy for NStatusMagic {}
impl FPaste for NStatusMagic {}

impl FPlaceholder for NStatusMagic {
    fn placeholder() -> Self {
        NStatusMagic::new(next_id(), player_id(), "New Status".to_string())
            .with_description(NStatusDescription::placeholder())
            .with_representation(NStatusRepresentation::placeholder())
            .with_state(NState::new(next_id(), player_id(), 1))
    }
}

impl FInfo for NStatusMagic {
    fn info(&self, _ctx: &ClientContext) -> Cstr {
        format!("Status: {}", self.status_name).cstr()
    }
}

// Implement FTitle for other node types
impl FTitle for NArena {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Arena".cstr()
    }
}

impl FDescription for NArena {
    fn description_cstr(&self, _ctx: &ClientContext) -> Cstr {
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
    fn description_cstr(&self, _: &ClientContext) -> Cstr {
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
    fn description_cstr(&self, _: &ClientContext) -> Cstr {
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
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr {
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
    fn placeholder() -> Self {
        NPlayer::new(next_id(), player_id(), "New Player".to_string())
            .with_player_data(NPlayerData::placeholder())
            .with_identity(NPlayerIdentity::placeholder())
            .with_active_match(NMatch::placeholder())
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

impl FTitle for NPlayerData {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Player Data".cstr()
    }
}

impl FDescription for NPlayerData {
    fn description_cstr(&self, _: &ClientContext) -> Cstr {
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
    fn description_cstr(&self, _: &ClientContext) -> Cstr {
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

impl FTitle for NHouseColor {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.color.cstr()
    }
}

impl FPlaceholder for NHouseColor {
    fn placeholder() -> Self {
        NHouseColor::new(next_id(), player_id(), HexColor("#F08050".to_string()))
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
    fn placeholder() -> Self {
        NAbilityDescription::new(next_id(), player_id(), "Default description".to_string())
            .with_effect(NAbilityEffect::placeholder())
    }
}

impl FDescription for NAbilityDescription {
    fn description_cstr(&self, _ctx: &ClientContext) -> Cstr {
        self.description.cstr()
    }
}

impl FTitle for NAbilityEffect {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Ability Effect".cstr()
    }
}

impl FDescription for NAbilityEffect {
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr {
        self.actions
            .iter()
            .map(|a| a.title_recursive(ctx))
            .join("\n")
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
    fn display(&self, _: &ClientContext, ui: &mut Ui) -> Response {
        self.description.cstr().label(ui)
    }
}

impl FTitle for NStatusDescription {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FPlaceholder for NStatusDescription {
    fn placeholder() -> Self {
        NStatusDescription::new(
            next_id(),
            player_id(),
            "Default status description".to_string(),
        )
        .with_behavior(NStatusBehavior::placeholder())
    }
}

impl FDescription for NStatusDescription {
    fn description_cstr(&self, _: &ClientContext) -> Cstr {
        self.description.cstr()
    }
}

impl FTitle for NStatusBehavior {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Status Behavior".cstr()
    }
}

impl FDescription for NStatusBehavior {
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr {
        self.reactions
            .iter()
            .map(|r| r.description_cstr(ctx))
            .join("\n")
    }
}

impl FStats for NStatusBehavior {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FDisplay for NStatusBehavior {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.description_cstr(ctx).label_w(ui)
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
    fn description_cstr(&self, _: &ClientContext) -> Cstr {
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
    fn placeholder() -> Self {
        NStatusRepresentation::new(
            next_id(),
            player_id(),
            Material(vec![PainterAction::circle(Box::new(Expression::f32(0.5)))]),
        )
    }
}

impl FTitle for NTeam {
    fn title(&self, ctx: &ClientContext) -> Cstr {
        match self.fusions_ref(ctx) {
            Ok(f) => f
                .into_iter()
                .filter(|f| f.trigger_unit_ref(ctx).is_ok())
                .map(|f| f.title(ctx))
                .join("[tw +]"),
            Err(_) => "[red error]".into(),
        }
    }
}

impl FDescription for NTeam {
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr {
        let houses = self
            .houses
            .iter()
            .map(|h: &NHouse| h.description_cstr(ctx))
            .join(", ");
        let fusions = self
            .fusions
            .iter()
            .map(|f: &NFusion| f.description_cstr(ctx))
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
    fn placeholder() -> Self {
        let house = NHouse::placeholder();
        let unit_id = house.units().unwrap()[0].id;
        let fusion = NFusion::placeholder().with_slots(
            [NFusionSlot::new(
                next_id(),
                player_id(),
                0,
                UnitActionRange {
                    start: 0,
                    length: 1,
                },
            )
            .with_unit_id(unit_id)]
            .into(),
        );
        NTeam::new(next_id(), player_id())
            .with_houses([house].into())
            .with_fusions(
                [
                    fusion,
                    NFusion::new(next_id(), player_id(), 1, 0, 0, 0, 0),
                    NFusion::new(next_id(), player_id(), 2, 0, 0, 0, 0),
                    NFusion::new(next_id(), player_id(), 3, 0, 0, 0, 0),
                    NFusion::new(next_id(), player_id(), 4, 0, 0, 0, 0),
                ]
                .into(),
            )
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

impl FCompactView for NTeam {
    fn render_compact(&self, ctx: &ClientContext, ui: &mut Ui) {
        self.title(ctx).label(ui);
    }

    fn render_hover(&self, ctx: &ClientContext, ui: &mut Ui) {
        match self.clone().load_all(ctx) {
            Ok(team) => {
                TeamEditor::new().edit(team, ctx, ui);
            }
            Err(e) => e.ui(ui),
        }
    }
}

impl FTitle for NMatch {
    fn title(&self, _: &ClientContext) -> Cstr {
        format!("Match F{}", self.floor).cstr()
    }
}

impl FDescription for NMatch {
    fn description_cstr(&self, _: &ClientContext) -> Cstr {
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

impl FPlaceholder for NMatch {
    fn placeholder() -> Self {
        NMatch::new(
            next_id(),
            0,
            0,
            1,
            3,
            false,
            MatchState::Shop,
            vec![],
            vec![],
            None,
        )
        .with_team(NTeam::placeholder())
    }
}

impl FTitle for NFusion {
    fn title(&self, ctx: &ClientContext) -> Cstr {
        let units = ctx
            .exec_ref(|ctx| {
                let mut units = self
                    .units(ctx)?
                    .into_iter()
                    .map(|u| {
                        (
                            u.name().to_string(),
                            ctx.exec_ref(|ctx| {
                                ctx.set_owner(u.id);
                                Ok(ctx.color())
                            })
                            .unwrap(),
                        )
                    })
                    .collect_vec();
                fn unit_str(unit: (String, Color32), len: usize) -> String {
                    format!("[{} {}]", unit.1.to_hex(), unit.0.cut_start(len))
                }
                match units.len() {
                    0 => Ok("[tw Empty]".into()),
                    1 => Ok(unit_str(units.remove(0), 0)),
                    2..=3 => Ok(units.into_iter().map(|name| unit_str(name, 3)).join("")),
                    _ => Ok(units.into_iter().map(|name| unit_str(name, 2)).join("")),
                }
            })
            .unwrap_or_default();
        units
    }
}

impl FDescription for NFusion {
    fn description_cstr(&self, _ctx: &ClientContext) -> Cstr {
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
    fn placeholder() -> Self {
        NFusion::new(next_id(), 0, 0, 0, 0, 0, 1).with_slots([NFusionSlot::placeholder()].into())
    }
}

impl FTitle for NFusionSlot {
    fn title(&self, _: &ClientContext) -> Cstr {
        format!("Slot #{}", self.index).cstr()
    }
}

impl FDescription for NFusionSlot {
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr {
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
        self.description.cstr()
    }
}

impl FDescription for NUnitDescription {
    fn description_expanded_cstr(&self, ctx: &ClientContext) -> Cstr {
        let mut description = String::new();
        let _ = ctx.exec_ref(|ctx| {
            let house =
                ctx.load_first_parent_recursive_ref::<NHouse>(ctx.owner().unwrap_or(self.id))?;
            match self.magic_type {
                MagicType::Ability => {
                    let ability = house.ability_ref(ctx)?;
                    description = format!(
                        "{}: {}\n\n",
                        ability.title(ctx),
                        ability.description_cstr(ctx)
                    );
                }
                MagicType::Status => {
                    let status = house.status_ref(ctx)?;
                    description = format!(
                        "{}: {}\n\n",
                        status.title(ctx),
                        status.description_cstr(ctx)
                    );
                }
            }
            Ok(())
        });
        description + &self.description_cstr(ctx)
    }
    fn description_cstr(&self, _: &ClientContext) -> Cstr {
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
            ui.horizontal_wrapped(|ui| {
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

impl FTitle for NUnitStats {
    fn title(&self, _: &ClientContext) -> Cstr {
        format!("{}/{}", self.pwr, self.hp).cstr()
    }
}

impl FDescription for NUnitStats {
    fn description_cstr(&self, _: &ClientContext) -> Cstr {
        format!(
            "{}/{}",
            self.pwr.cstr_c(VarName::pwr.color()),
            self.hp.cstr_c(VarName::hp.color())
        )
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

impl FTitle for NState {
    fn title(&self, _: &ClientContext) -> Cstr {
        format!("[tw Unit State] {}x", self.stax).cstr()
    }
}

impl FDescription for NState {
    fn description_cstr(&self, _: &ClientContext) -> Cstr {
        format!("{} stax", self.stax).cstr()
    }
}

impl FStats for NState {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![(VarName::stax, VarValue::i32(self.stax))]
    }
}

impl FTag for NState {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        "State".cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        Some(format!("{}x", self.stax).cstr())
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(255, 255, 0)
    }
}

impl FDisplay for NState {
    fn display(&self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.label("stax:");
            format!("{}", self.stax)
                .cstr_c(Color32::from_rgb(255, 255, 0))
                .label(ui);
        })
        .response
    }
}

impl FTitle for NUnitBehavior {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.magic_type.cstr()
    }
}

impl FDescription for NUnitBehavior {
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr {
        self.reaction.description_cstr(ctx)
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
        self.description_expanded_cstr(ctx).label(ui)
    }
}

impl FTitle for NUnitRepresentation {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Unit Representation".cstr()
    }
}

impl FTitle for NRepresentation {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Representation".cstr()
    }
}

impl FDescription for NUnitRepresentation {
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr {
        self.material.description_cstr(ctx)
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

impl FDisplay for NRepresentation {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.material.display(ctx, ui)
    }
}

// ============================================================================
// Additional FEdit implementations for missing node types
// (Ordered according to raw_nodes.rs struct definitions)
// ============================================================================

impl FPlaceholder for NArena {
    fn placeholder() -> Self {
        NArena::new(next_id(), 0, 0)
    }
}

impl FPlaceholder for NFloorPool {
    fn placeholder() -> Self {
        NFloorPool::new(next_id(), 0, 1)
    }
}

impl FPlaceholder for NFloorBoss {
    fn placeholder() -> Self {
        NFloorBoss::new(next_id(), 0, 1).with_team(NTeam::placeholder())
    }
}

impl FPlaceholder for NPlayerData {
    fn placeholder() -> Self {
        NPlayerData::new(next_id(), 0, None, true, 0)
    }
}

impl FPlaceholder for NAbilityEffect {
    fn placeholder() -> Self {
        NAbilityEffect::new(next_id(), 0, vec![Action::noop])
    }
}

impl FPlaceholder for NPlayerIdentity {
    fn placeholder() -> Self {
        NPlayerIdentity::new(next_id(), 0, None)
    }
}

impl FPlaceholder for NStatusBehavior {
    fn placeholder() -> Self {
        NStatusBehavior::new(
            next_id(),
            0,
            vec![Reaction {
                trigger: Trigger::BattleStart,
                actions: vec![Action::noop],
            }],
        )
    }
}

impl FPlaceholder for NFusionSlot {
    fn placeholder() -> Self {
        NFusionSlot::new(next_id(), 0, 0, default())
    }
}
impl FEdit for ShopOffer {
    fn edit(&mut self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Buy Limit:");
                if let Some(ref mut limit) = self.buy_limit {
                    limit.edit(ui)
                } else {
                    if ui.button("Set Limit").clicked() {
                        self.buy_limit = Some(1);
                    }
                    ui.label("No limit")
                }
            })
            .inner
        })
        .inner
    }
}

impl FEdit for ShopSlot {
    fn edit(&mut self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let mut response = ui
                .horizontal(|ui| {
                    ui.label("Card Kind:");
                    self.card_kind.edit(ui)
                })
                .inner;

            response = response.union(
                ui.horizontal(|ui| {
                    ui.label("Node ID:");
                    self.node_id.edit(ui)
                })
                .inner,
            );

            response = response.union(
                ui.horizontal(|ui| {
                    ui.label("Price:");
                    self.price.edit(ui)
                })
                .inner,
            );

            response = response.union(
                ui.horizontal(|ui| {
                    ui.label("Sold:");
                    self.sold.edit(ui)
                })
                .inner,
            );

            response
        })
        .inner
    }
}

impl FEdit for CardKind {
    fn edit(&mut self, _: &mut Ui) -> Response {
        // let (_, response) = Selector::ui_iter(self, ui);
        // response
        todo!()
    }
}

// ============================================================================

// Implement for Vec<T> where appropriate
impl<T: FTitle> FTitle for Vec<T> {
    fn title(&self, _: &ClientContext) -> Cstr {
        format!("List ({})", self.len()).cstr()
    }
}

impl<T: FDisplay> FDisplay for Vec<T> {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        let mut response = format!("List ({})", self.len()).label(ui);
        for item in self {
            response |= item.display(ctx, ui);
        }
        response
    }
}

impl<T: FEdit + Default> FEdit for Vec<T> {
    fn edit(&mut self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            self.as_mutable_list(|a, _, ui| a.edit(ui))
                .editable(|| T::default())
                .compose(&EMPTY_CONTEXT, ui)
        })
        .inner
    }
}

impl FTitle for Vec<Box<PainterAction>> {
    fn title(&self, _: &ClientContext) -> Cstr {
        format!("List ({})", self.len()).cstr()
    }
}

impl FDisplay for Vec<Box<PainterAction>> {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        let mut response = format!("List ({})", self.len()).label(ui);
        for item in self {
            response |= item.display(ctx, ui);
        }
        response
    }
}

impl FEdit for Vec<Box<PainterAction>> {
    fn edit(&mut self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            self.as_mutable_list(|a, _, ui| a.edit(ui))
                .editable(|| Box::new(PainterAction::default()))
                .compose(&EMPTY_CONTEXT, ui)
        })
        .inner
    }
}

impl FPlaceholder for NUnitBehavior {
    fn placeholder() -> Self {
        NUnitBehavior::new(
            next_id(),
            0,
            Reaction {
                trigger: Trigger::BattleStart,
                actions: vec![Action::debug(
                    Expression::string("debug action".into()).into(),
                )],
            },
            MagicType::Ability,
        )
    }
}

impl FPlaceholder for NState {
    fn placeholder() -> Self {
        NState::new(next_id(), 0, 1)
    }
}

impl FPlaceholder for NUnitStats {
    fn placeholder() -> Self {
        NUnitStats::new(next_id(), 0, 1, 4)
    }
}

impl FPlaceholder for NUnitDescription {
    fn placeholder() -> Self {
        NUnitDescription::new(
            next_id(),
            0,
            "Placeholder Description".to_string(),
            MagicType::Ability,
            Trigger::BattleStart,
        )
        .with_representation(NUnitRepresentation::placeholder())
        .with_behavior(NUnitBehavior::placeholder())
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
    fn edit(&mut self, ui: &mut Ui) -> Response {
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
            v.edit(ui)
        } else {
            ui.label("(none)")
        };
        checkbox_response.union(edit_response)
    }
}

impl FEdit for Colorix {
    fn edit(&mut self, ui: &mut Ui) -> Response {
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
            response |= ui
                .vertical(|ui| {
                    let mut response = ui.label("Semantic Colors");
                    if self.show_semantic_editor(Semantic::Accent, ui) {
                        response.mark_changed();
                    }
                    if self.show_semantic_editor(Semantic::Background, ui) {
                        response.mark_changed();
                    }
                    if self.show_semantic_editor(Semantic::Success, ui) {
                        response.mark_changed();
                    }
                    if self.show_semantic_editor(Semantic::Error, ui) {
                        response.mark_changed();
                    }
                    if self.show_semantic_editor(Semantic::Warning, ui) {
                        response.mark_changed();
                    }
                    response
                })
                .inner;

            if response.changed() {
                self.apply(ui.ctx());
                self.clone().save();
            }

            response
        })
        .inner
    }
}

impl FPlaceholder for NUnitRepresentation {
    fn placeholder() -> Self {
        NUnitRepresentation::new(
            next_id(),
            0,
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
        self.paint(rect, ctx, ui);
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
        let color = ctx
            .get_var(VarName::color)
            .get_color()
            .unwrap_or(MISSING_COLOR);
        self.unit_name.cstr_c(color).label(ui);
    }

    fn render_hover(&self, ctx: &ClientContext, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.strong(format!("Unit: {}", self.unit_name));
            if let Ok(stats) = self.stats_ref(ctx) {
                ui.label(format!("Power: {}, HP: {}", stats.pwr, stats.hp));
            }
            if let Ok(desc) = self.description_ref(ctx) {
                if !desc.description.is_empty() {
                    ui.separator();
                    desc.description.cstr().label_w(ui);
                }
            }

            if let Ok(house) = ctx.load_first_parent_ref::<NHouse>(self.id()) {
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
        let color = ctx
            .get_var(VarName::color)
            .get_color()
            .unwrap_or(MISSING_COLOR);
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
        let color = ctx
            .get_var(VarName::color)
            .get_color()
            .unwrap_or(MISSING_COLOR);
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

impl FDescription for NHouseColor {
    fn description_cstr(&self, _ctx: &ClientContext) -> Cstr {
        self.color.cstr()
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

impl FCard for NUnit {}

impl FCard for NHouse {}

impl FCard for NAbilityMagic {}

impl FCard for NStatusMagic {}

// FPreview implementations
impl FPreview for NUnit {
    fn preview(&self, ctx: &ClientContext, ui: &mut Ui, rect: Rect) {
        ctx.exec_ref(|ctx| {
            MatRect::new(rect.size())
                .add_mat(
                    &self.description_ref(ctx)?.representation_ref(ctx)?.material,
                    self.id,
                )
                .unit_rep_with_default(self.id)
                .corners(false)
                .enabled(false)
                .ui(ui, ctx);
            Ok(())
        })
        .ui(ui);
    }
}

impl FPreview for NHouse {
    fn preview(&self, ctx: &ClientContext, ui: &mut Ui, rect: Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            let color = ctx.color();
            ui.vertical_centered(|ui| {
                if let Ok(ability) = self.ability_ref(ctx) {
                    ability
                        .ability_name
                        .cstr_cs(color, CstrStyle::Bold)
                        .label(ui);
                    if let Ok(dsc) = ability.description_ref(ctx) {
                        dsc.description.cstr_s(CstrStyle::Small).label_w(ui);
                    }
                }
                if let Ok(status) = self.status_ref(ctx) {
                    status.status_name.cstr_cs(color, CstrStyle::Bold).label(ui);
                    if let Ok(dsc) = status.description_ref(ctx) {
                        dsc.description.cstr_s(CstrStyle::Small).label_w(ui);
                    }
                }
            });
        });
    }
}

impl FPreview for NAbilityMagic {
    fn preview(&self, ctx: &ClientContext, ui: &mut Ui, rect: Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("✨").size(32.0).color(ctx.color()));
                ui.label(
                    RichText::new(&self.ability_name)
                        .strong()
                        .color(ctx.color()),
                );
            });
        });
    }
}

impl FPreview for NStatusMagic {
    fn preview(&self, ctx: &ClientContext, ui: &mut Ui, rect: Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("💫").size(32.0).color(ctx.color()));
                ui.label(RichText::new(&self.status_name).strong().color(ctx.color()));
            });
        });
    }
}
