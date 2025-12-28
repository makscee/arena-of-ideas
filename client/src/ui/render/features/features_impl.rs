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
    fn edit(&mut self, ui: &mut Ui, _ctx: &ClientContext) -> Response {
        DragValue::new(self).ui(ui)
    }
}

impl FEdit for f32 {
    fn edit(&mut self, ui: &mut Ui, _ctx: &ClientContext) -> Response {
        DragValue::new(self).min_decimals(1).ui(ui)
    }
}

impl FEdit for String {
    fn edit(&mut self, ui: &mut Ui, _ctx: &ClientContext) -> Response {
        Input::new("").ui_string(self, ui)
    }
}

impl FEdit for u8 {
    fn edit(&mut self, ui: &mut Ui, _ctx: &ClientContext) -> Response {
        DragValue::new(self).ui(ui)
    }
}

impl FEdit for u64 {
    fn edit(&mut self, ui: &mut Ui, _ctx: &ClientContext) -> Response {
        DragValue::new(self).ui(ui)
    }
}

impl FEdit for usize {
    fn edit(&mut self, ui: &mut Ui, _ctx: &ClientContext) -> Response {
        let mut val = *self as i32;
        let resp = DragValue::new(&mut val).ui(ui);
        *self = val.max(0) as usize;
        resp
    }
}

impl FEdit for bool {
    fn edit(&mut self, ui: &mut Ui, _ctx: &ClientContext) -> Response {
        Checkbox::new(self, "").ui(ui)
    }
}

impl FEdit for Vec2 {
    fn edit(&mut self, ui: &mut Ui, _ctx: &ClientContext) -> Response {
        ui.horizontal(|ui| {
            let rx = DragValue::new(&mut self.x).prefix("x:").ui(ui);
            let ry = DragValue::new(&mut self.y).prefix("y:").ui(ui);
            rx.union(ry)
        })
        .inner
    }
}

impl FEdit for Color32 {
    fn edit(&mut self, ui: &mut Ui, _ctx: &ClientContext) -> Response {
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
    fn edit(&mut self, ui: &mut Ui, _ctx: &ClientContext) -> Response {
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

impl FEdit for MatchState {
    fn edit(&mut self, ui: &mut egui::Ui, _ctx: &ClientContext) -> egui::Response {
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
    fn edit(&mut self, ui: &mut Ui, _ctx: &ClientContext) -> Response {
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
    fn edit(&mut self, ui: &mut Ui, ctx: &ClientContext) -> Response {
        let (_, mut response) = Selector::ui_enum(self, ui);
        ui.horizontal(|ui| match self {
            VarValue::i32(v) => response |= v.edit(ui, ctx),
            VarValue::f32(v) => response |= v.edit(ui, ctx),
            VarValue::u64(v) => response |= DragValue::new(v).ui(ui),
            VarValue::bool(v) => response |= v.edit(ui, ctx),
            VarValue::String(v) => response |= v.edit(ui, ctx),
            VarValue::Vec2(v) => response |= v.edit(ui, ctx),
            VarValue::Color32(v) => response |= v.edit(ui, ctx),
            VarValue::list(v) => {
                for v in v {
                    response |= v.edit(ui, ctx);
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
    fn edit(&mut self, ui: &mut Ui, ctx: &ClientContext) -> Response {
        let (_, mut response) = Selector::ui_enum(self, ui);
        const DEBUG_TXT: &str = "🪲 Debug";
        const SCALE_TXT: &str = "⚖️ Scale";
        let menu_response = self
            .as_empty_mut()
            .with_menu()
            .add_copy()
            .add_paste()
            .add_action_empty(DEBUG_TXT)
            .add_action_empty(SCALE_TXT)
            .compose_with_menu(ctx, ui);
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
        response
    }
}

impl FTitle for Trigger {
    fn title(&self, _ctx: &ClientContext) -> Cstr {
        self.as_ref().to_owned()
    }
}

impl FEdit for Trigger {
    fn edit(&mut self, ui: &mut Ui, ctx: &ClientContext) -> Response {
        ui.horizontal(|ui| {
            "[tw Trigger:]".cstr().label(ui);
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
                Trigger::ChangeStat(var) => var.edit(ui, ctx) | resp,
                Trigger::Any(triggers) => triggers.edit(ui, ctx) | resp,
            }
        })
        .inner
    }
}

impl FEdit for Target {
    fn edit(&mut self, ui: &mut Ui, ctx: &ClientContext) -> Response {
        ui.horizontal(|ui| {
            "[tw Target:]".cstr().label(ui);
            let resp = Selector::ui_enum(self, ui).1;
            match self {
                Target::Owner
                | Target::RandomEnemy
                | Target::AllEnemies
                | Target::RandomAlly
                | Target::AllAllies
                | Target::All
                | Target::Caster
                | Target::Attacker
                | Target::Target
                | Target::AdjacentBack
                | Target::AdjacentFront => resp,
                Target::AllyAtSlot(slot) => {
                    ui.horizontal(|ui| {
                        ui.label("Slot:");
                        let mut slot_i32 = *slot as i32;
                        let r = DragValue::new(&mut slot_i32).range(0..=8).ui(ui);
                        *slot = slot_i32 as u8;
                        r
                    })
                    .inner
                        | resp
                }
                Target::EnemyAtSlot(slot) => {
                    ui.horizontal(|ui| {
                        ui.label("Slot:");
                        let mut slot_i32 = *slot as i32;
                        let r = DragValue::new(&mut slot_i32).range(0..=8).ui(ui);
                        *slot = slot_i32 as u8;
                        r
                    })
                    .inner
                        | resp
                }
                Target::List(targets) => targets.edit(ui, ctx) | resp,
            }
        })
        .inner
    }
}

// Action
impl FTitle for Action {
    fn title(&self, ctx: &ClientContext) -> Cstr {
        fn x_text(ctx: &ClientContext, mut x: i32) -> NodeResult<String> {
            let owner = ctx.owner()?;
            let house = ctx.load_first_parent_recursive_ref::<NHouse>(owner)?;
            let house_x = house.state.load_node(ctx)?.stax;
            if house_x > 0 {
                x = x.at_most(house_x);
            }
            Ok(format!(" [{} x{x}]", VarName::stax.color().to_hex()))
        }
        match self {
            Action::use_ability(..) => {
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
            Action::apply_status(..) => {
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
    fn edit(&mut self, ui: &mut Ui, ctx: &ClientContext) -> Response {
        let (_, mut response) = Selector::ui_enum(self, ui);
        let menu_resp = self
            .as_empty_mut()
            .with_menu()
            .add_copy()
            .add_paste()
            .compose_with_menu(ctx, ui);
        if let Some(value) = menu_resp.pasted() {
            *self = value.clone();
        }
        fn house_selector(
            ui: &mut Ui,
            ctx: &ClientContext,
            house_name: &mut String,
            is_ability: bool,
        ) -> Option<NHouse> {
            match ctx.exec_ref(|ctx| {
                let owner = ctx.owner()?;
                let parent = ctx
                    .first_parent(owner, NodeKind::NTeam)
                    .or_else(|_| ctx.first_parent(owner, NodeKind::NMatch))?;
                let houses = ctx.collect_kind_children_recursive(parent, NodeKind::NHouse)?;
                let mut houses: HashMap<String, NHouse> = HashMap::from_iter(
                    ctx.load_many_ref::<NHouse>(&houses)?
                        .into_iter()
                        .cloned()
                        .filter_map(|mut h| {
                            if (is_ability && h.ability.load_mut(ctx).is_ok_and(|a| a.is_loaded())
                                || !is_ability
                                    && h.status.load_mut(ctx).is_ok_and(|a| a.is_loaded()))
                                && h.color.load_mut(ctx).is_ok_and(|a| a.is_loaded())
                            {
                                Some((h.name().to_owned(), h))
                            } else {
                                None
                            }
                        }),
                );
                "[tw House:]".cstr().label(ui);
                if Selector::ui_iter(house_name, houses.keys(), ui).0 {
                    let house = houses.remove(house_name).unwrap();
                    return Ok(Some(house));
                }
                Ok(None)
            }) {
                Ok(h) => h,
                Err(e) => {
                    e.ui(ui);
                    None
                }
            }
        }
        match self {
            Action::use_ability(house_name, ability_name, color) => {
                ui.horizontal(|ui| {
                    if let Some(mut house) = house_selector(ui, ctx, house_name, true) {
                        *ability_name = house.ability.take_loaded().unwrap().ability_name;
                        *color = house.color.take_loaded().unwrap().color;
                        response.mark_changed();
                    }
                    format!("[{} [b {}][tw /]{}]", color, house_name, ability_name).label(ui);
                });
            }
            Action::apply_status(house_name, status_name, color) => {
                ui.horizontal(|ui| {
                    if let Some(mut house) = house_selector(ui, ctx, house_name, false) {
                        *status_name = house.status.take_loaded().unwrap().status_name;
                        *color = house.color.take_loaded().unwrap().color;
                        response.mark_changed();
                    }
                    format!("[{} [b {}][tw /]{}]", color, house_name, status_name).label(ui);
                });
            }
            _ => {}
        }

        response
    }
}

// PainterAction
impl FTitle for RhaiScript<PainterAction> {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FDescription for RhaiScript<PainterAction> {
    fn description_cstr(&self, _ctx: &ClientContext) -> Cstr {
        format!("Script with {} lines", self.code.lines().count())
    }
}

impl FDisplay for RhaiScript<PainterAction> {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.paint_viewer(ctx, ui)
    }
}

impl FTitle for Behavior {
    fn title(&self, _: &ClientContext) -> Cstr {
        self.cstr()
    }
}

impl FDescription for Behavior {
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr {
        format!(
            "{}:\n{}",
            self.trigger.cstr(),
            self.effect.actions.iter().map(|a| a.title(ctx)).join("\n")
        )
    }
}

impl FDisplay for Trigger {
    fn display(&self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.cstr().label(ui)
    }
}

impl FDisplay for Behavior {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.group(|ui| {
            ui.vertical(|ui| {
                let mut response = self.trigger.display(ctx, ui);
                for action in &self.effect.actions {
                    response |= action.display(ctx, ui);
                }
                response
            })
            .inner
        })
        .inner
    }
}

impl FEdit for Effect {
    fn edit(&mut self, ui: &mut Ui, ctx: &ClientContext) -> Response {
        ui.vertical(|ui| {
            let response = ui
                .horizontal(|ui| {
                    ui.label("Description:");
                    self.description.edit(ui, ctx)
                })
                .inner;
            ui.label("Actions:");
            self.actions.edit(ui, ctx).union(response)
        })
        .inner
    }
}

impl FEdit for Behavior {
    fn edit(&mut self, ui: &mut Ui, ctx: &ClientContext) -> Response {
        ui.vertical(|ui| {
            self.trigger.edit(ui, ctx) | self.target.edit(ui, ctx) | self.effect.edit(ui, ctx)
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
        self.behavior
            .load_node(ctx)
            .map(|b| b.description_cstr(ctx))
            .unwrap_or("[tw -]".to_owned())
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
                .and_then(|h| h.state.load_node(ctx))
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
        let tier = if let Ok(behavior) = self.behavior.load_node(ctx) {
            use schema::Tier;
            behavior.tier()
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
        let tier = if let Ok(behavior) = self.behavior.load_node(ctx) {
            use schema::Tier;
            behavior.tier()
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
        if let Ok(behavior) = self.behavior.load_node(ctx) {
            if let Ok(stats) = behavior.stats.load_node(ctx) {
                info_parts.push(format!(
                    "[{} {}]/[{} {}]",
                    VarName::pwr.color().to_hex(),
                    stats.pwr,
                    VarName::hp.color().to_hex(),
                    stats.hp
                ));
            }
        }

        if let Ok(house) = ctx.load_first_parent_ref::<NHouse>(self.id()) {
            let color = house.color_for_text(ctx);
            info_parts.push(house.house_name.cstr_c(color));
        }

        info_parts.join(" | ")
    }
}

impl FCopy for NUnit {}
impl FPaste for NUnit {}

impl FPlaceholder for NUnit {
    fn placeholder() -> Self {
        NUnit::new(next_id(), player_id(), "New Unit".to_string())
            .with_state(NUnitState::placeholder())
            .with_behavior(NUnitBehavior::placeholder())
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
            self.state.load_node(ctx).ok()?.stax
        ))
    }

    fn tag_color(&self, ctx: &ClientContext) -> Color32 {
        self.color_for_text(ctx)
    }
}

impl FInfo for NHouse {
    fn info(&self, ctx: &ClientContext) -> Cstr {
        let mut info_parts = vec![self.house_name.clone()];
        if let Ok(ability) = self.ability.load_node(ctx) {
            info_parts.push(ability.info(ctx));
        }
        if let Ok(status) = self.status.load_node(ctx) {
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
        NHouse::new(next_id(), player_id(), "Placeholder House".to_string())
            .with_color(NHouseColor::placeholder())
    }
}

// NAbilityMagic
impl FTitle for NAbilityMagic {
    fn title(&self, ctx: &ClientContext) -> Cstr {
        let color = ctx
            .get_var(VarName::color)
            .get_color()
            .unwrap_or(colorix().high_contrast_text());
        self.ability_name.cstr_c(color)
    }
}

impl FDescription for NAbilityMagic {
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr {
        let name = self.name().cstr_c(ctx.color());
        name
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
            .with_effect(NAbilityEffect::placeholder())
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
        let name = self.name().cstr_c(ctx.color());
        name
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
            .with_behavior(NStatusBehavior::placeholder())
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
        if let Ok(data) = self.player_data.load_node(ctx) {
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
        if let Ok(data) = self.player_data.load_node(ctx) {
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
            if let Ok(data) = self.player_data.load_node(ctx) {
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

impl FTitle for NAbilityEffect {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Ability Effect".cstr()
    }
}

impl FDescription for NAbilityEffect {
    fn description_cstr(&self, _ctx: &ClientContext) -> Cstr {
        self.effect.description.clone().into()
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
        Some(self.effect.description.clone().cstr())
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(255, 165, 0)
    }
}

impl FTitle for NStatusBehavior {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Status Behavior".cstr()
    }
}

impl FDescription for NStatusBehavior {
    fn description_cstr(&self, _: &ClientContext) -> Cstr {
        format!("Trigger: {}", self.trigger).cstr()
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
        Some(format!("T{}", self.tier()).cstr())
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(255, 255, 0)
    }
}

impl FStats for NRepresentation {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NRepresentation {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        "Representation".cstr()
    }

    fn tag_value(&self, _: &ClientContext) -> Option<Cstr> {
        Some(self.script.cstr())
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(0, 128, 128)
    }
}

impl FTitle for NTeam {
    fn title(&self, ctx: &ClientContext) -> Cstr {
        match self.slots.load_nodes(ctx) {
            Ok(f) => f
                .into_iter()
                .filter_map(|f| Some(f.unit.load_node(ctx).ok()?.title(ctx)))
                .join("[tw +]"),
            Err(_) => "[red error]".into(),
        }
    }
}

impl FDescription for NTeam {
    fn description_cstr(&self, _ctx: &ClientContext) -> Cstr {
        "Team description".to_owned()
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
        let unit = NUnit::placeholder().with_state(NUnitState::new(next_id(), player_id(), 1, 0));
        let house = NHouse::placeholder();

        let slot = NTeamSlot::new(next_id(), player_id(), 0).with_unit(unit);
        NTeam::new(next_id(), player_id())
            .with_houses([house].into())
            .with_slots(
                [
                    slot,
                    NTeamSlot::new(next_id(), player_id(), 1),
                    NTeamSlot::new(next_id(), player_id(), 2),
                    NTeamSlot::new(next_id(), player_id(), 3),
                    NTeamSlot::new(next_id(), player_id(), 4),
                ]
                .into(),
            )
    }
}

impl FDisplay for NTeam {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            if let Ok(houses) = self.houses.load_nodes(ctx) {
                ui.label(format!("Houses ({})", houses.len()));
                for house in houses {
                    ui.horizontal(|ui| {
                        ui.label("  •");
                        house.house_name.cstr().label(ui);
                    });
                }
            }
        })
        .response
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
            None,
        )
    }
}

impl FTitle for NTeamSlot {
    fn title(&self, _: &ClientContext) -> Cstr {
        format!("Slot #{}", self.index).cstr()
    }
}

impl FDescription for NTeamSlot {
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr {
        if let Ok(unit) = self.unit.load_node(ctx) {
            unit.unit_name.cstr()
        } else {
            "Empty slot".cstr()
        }
    }
}

impl FStats for NTeamSlot {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FTag for NTeamSlot {
    fn tag_name(&self, _: &ClientContext) -> Cstr {
        format!("Slot #{}", self.index).cstr()
    }

    fn tag_value(&self, ctx: &ClientContext) -> Option<Cstr> {
        if let Ok(unit) = self.unit.load_node(ctx) {
            Some(unit.unit_name.cstr())
        } else {
            None
        }
    }

    fn tag_color(&self, _: &ClientContext) -> Color32 {
        Color32::from_rgb(128, 0, 128)
    }
}

impl FDisplay for NTeamSlot {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            format!("Slot #{}", self.index)
                .cstr_c(Color32::from_rgb(128, 0, 128))
                .label(ui);
            ui.label(":");
            if let Ok(unit) = self.unit.load_node(ctx) {
                unit.unit_name.cstr().label(ui)
            } else {
                "Empty".cstr_c(Color32::from_rgb(128, 128, 128)).label(ui)
            }
        })
        .inner
    }
}

impl FDisplay for NShopPool {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.title(ctx).label(ui)
    }
}

impl FTitle for NShopPool {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Shop Pool".cstr()
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
        format!("[tw State] {}x", self.stax).cstr()
    }
}

impl FDescription for NState {
    fn description_cstr(&self, _: &ClientContext) -> Cstr {
        format!("{} stax", self.stax).cstr()
    }
}

impl FStats for NState {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![(VarName::stax, self.stax.into())]
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

impl FTitle for NUnitState {
    fn title(&self, _: &ClientContext) -> Cstr {
        format!("[tw State] {}x", self.stax).cstr()
    }
}

impl FDescription for NUnitState {
    fn description_cstr(&self, _: &ClientContext) -> Cstr {
        format!("{} stax", self.stax).cstr()
    }
}

impl FStats for NUnitState {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![
            (VarName::stax, self.stax.into()),
            (VarName::dmg, self.dmg.into()),
        ]
    }
}

impl FDisplay for NUnitState {
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
        format!("[tw {}]|[yellow {}]", self.kind(), self.trigger.to_string())
    }
}

impl FDescription for NUnitBehavior {
    fn description_cstr(&self, _: &ClientContext) -> Cstr {
        format!("Trigger: {}, Target: {}", self.trigger, self.target).cstr()
    }
}

impl FStats for NUnitBehavior {
    fn stats(&self, _: &ClientContext) -> Vec<(VarName, VarValue)> {
        vec![]
    }
}

impl FInfo for NUnitBehavior {
    fn info(&self, _ctx: &ClientContext) -> Cstr {
        format!("Trigger: {}, Target: {}", self.trigger, self.target).cstr()
    }
}

impl FDisplay for NUnitBehavior {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.description_expanded_cstr(ctx).label_w(ui)
    }
}

impl FTitle for NRepresentation {
    fn title(&self, _: &ClientContext) -> Cstr {
        "Representation".cstr()
    }
}

impl FDescription for NRepresentation {
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr {
        self.script.description_cstr(ctx)
    }
}

impl FDisplay for NRepresentation {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.script.display(ctx, ui)
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
        NAbilityEffect::new(next_id(), 0, schema::RhaiScript::empty())
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
            schema::Trigger::BattleStart,
            schema::RhaiScript::empty(),
        )
        .with_representation(NRepresentation::placeholder())
    }
}
impl FPlaceholder for NTeamSlot {
    fn placeholder() -> Self {
        NTeamSlot::new(next_id(), 0, 0)
    }
}
impl FPlaceholder for NShopPool {
    fn placeholder() -> Self {
        Self::new(next_id(), 0)
    }
}

impl FEdit for ShopOffer {
    fn edit(&mut self, ui: &mut Ui, ctx: &ClientContext) -> Response {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Buy Limit:");
                if let Some(ref mut limit) = self.buy_limit {
                    limit.edit(ui, ctx)
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
    fn edit(&mut self, ui: &mut Ui, ctx: &ClientContext) -> Response {
        ui.vertical(|ui| {
            let mut response = ui
                .horizontal(|ui| {
                    ui.label("Card Kind:");
                    self.card_kind.edit(ui, ctx)
                })
                .inner;

            response = response.union(
                ui.horizontal(|ui| {
                    ui.label("Node ID:");
                    self.node_id.edit(ui, ctx)
                })
                .inner,
            );

            response = response.union(
                ui.horizontal(|ui| {
                    ui.label("Price:");
                    self.price.edit(ui, ctx)
                })
                .inner,
            );

            response = response.union(
                ui.horizontal(|ui| {
                    ui.label("Sold:");
                    self.sold.edit(ui, ctx)
                })
                .inner,
            );

            response
        })
        .inner
    }
}

impl FEdit for CardKind {
    fn edit(&mut self, _ui: &mut Ui, _ctx: &ClientContext) -> Response {
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
    fn edit(&mut self, ui: &mut Ui, ctx: &ClientContext) -> Response {
        ui.vertical(|ui| {
            self.as_mutable_list(|a, _, ui| a.edit(ui, ctx))
                .editable(|| T::default())
                .compose(ctx, ui)
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
    fn display(&self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        format!("List ({})", self.len()).label(ui)
    }
}

impl FPlaceholder for NUnitBehavior {
    fn placeholder() -> Self {
        NUnitBehavior::new(
            next_id(),
            0,
            Trigger::BattleStart,
            Target::default(),
            RhaiScript::new(
                "// Unit behavior effect script\nunit_actions.use_ability(\"debug\", target.id);"
                    .to_string(),
            ),
        )
        .with_stats(NUnitStats::placeholder())
        .with_representation(NRepresentation::placeholder())
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

impl FPlaceholder for NUnitState {
    fn placeholder() -> Self {
        let mut state = NUnitState::default();
        state.set_id(next_id());
        state.stax = 0;
        state.dmg = 0;
        state
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
    fn edit(&mut self, ui: &mut Ui, ctx: &ClientContext) -> Response {
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
            v.edit(ui, ctx)
        } else {
            ui.label("(none)")
        };
        checkbox_response.union(edit_response)
    }
}

impl FEdit for Colorix {
    fn edit(&mut self, ui: &mut Ui, _ctx: &ClientContext) -> Response {
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

impl FPlaceholder for NRepresentation {
    fn placeholder() -> Self {
        NRepresentation::new(
            next_id(),
            0,
            default(),
            true,
            0.0,
            0.0,
            RhaiScript::new("painter.circle(0.5);".to_string()),
        )
    }
}

// ============================================================================
// FCompactView Implementations
// ============================================================================

// impl FCompactView for RhaiScript<PainterAction> {
//     fn render_compact(&self, ctx: &ClientContext, ui: &mut Ui) {
//         let (rect, _) = ui.allocate_exact_size((LINE_HEIGHT * 2.0).v2(), Sense::click());
//         let mut p = Painter::new(rect, ui.ctx());
//         if let Ok(color) = ctx.get_var(VarName::color).get_color() {
//             p.color = color;
//         }
//         self.paint(rect, ctx, &mut p, ui);
//     }

//     fn render_hover(&self, _ctx: &ClientContext, ui: &mut Ui) {
//         ui.label(&self.code);
//     }
// }

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
            if let Ok(behavior) = self.behavior.load_node(ctx) {
                if let Ok(stats) = behavior.stats.load_node(ctx) {
                    ui.label(format!("Power: {}, HP: {}", stats.pwr, stats.hp));
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
            if let Ok(ability) = self.ability.load_node(ctx) {
                ui.label(format!("Ability: {}", ability.ability_name));
            }
            if let Ok(status) = self.status.load_node(ctx) {
                ui.label(format!("Status: {}", status.status_name));
            }
        });
    }
}

impl FCompactView for NRepresentation {
    fn render_compact(&self, _ctx: &ClientContext, _ui: &mut Ui) {}

    fn render_hover(&self, _ctx: &ClientContext, _ui: &mut Ui) {}
}

impl FCompactView for NUnitBehavior {
    fn render_compact(&self, _ctx: &ClientContext, ui: &mut Ui) {
        ui.horizontal(|ui| {
            format!("Trigger: {}", self.trigger).cstr().label(ui);
            ui.add_space(4.0);
            format!("Target: {}", self.target).cstr().label(ui);
        });
    }

    fn render_hover(&self, _ctx: &ClientContext, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.strong("Unit Behavior");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Type:");
            });
            ui.horizontal(|ui| {
                ui.label("Trigger:");
                self.trigger.cstr().label(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Tier:");
                use schema::Tier;
                let tier = self.effect.tier();
                format!("{}", tier).cstr_c(VarName::tier.color()).label(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Target:");
                self.target.cstr().label(ui);
            });
            ui.separator();
            ui.label("Effect Script:");
            if !self.effect.code.is_empty() {
                let preview = if self.effect.code.len() > 100 {
                    format!("{}...", &self.effect.code[..100])
                } else {
                    self.effect.code.clone()
                };
                preview.cstr().label(ui);
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
            if let Ok(effect) = self.effect.load_node(ctx) {
                ui.separator();
                effect.effect.description.cstr().label_w(ui);
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
            if let Ok(behavior) = self.behavior.load_node(ctx) {
                ui.separator();
                ui.label("Status with behavior");
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
            let mat = if let Ok(behavior) = self.behavior.load_node(ctx) {
                if let Ok(rep) = behavior.representation.get() {
                    rep.script.clone()
                } else {
                    unit_rep().script.clone()
                }
            } else {
                unit_rep().script.clone()
            };
            MatRect::new(rect.size())
                .add_mat(&mat, self.id)
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
                if let Ok(ability) = self.ability.load_node(ctx) {
                    ability
                        .ability_name
                        .cstr_cs(color, CstrStyle::Bold)
                        .label(ui);
                }
                if let Ok(status) = self.status.load_node(ctx) {
                    status.status_name.cstr_cs(color, CstrStyle::Bold).label(ui);
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

impl FEdit for Option<(u64, u64, Vec<PackedNodes>)> {
    fn edit(&mut self, ui: &mut Ui, _ctx: &ClientContext) -> Response {
        "fusions".cstr().label(ui)
    }
}

impl FEdit for Option<(u64, u64)> {
    fn edit(&mut self, ui: &mut Ui, ctx: &ClientContext) -> Response {
        let resp = ui.checkbox(&mut self.is_none(), "");
        if resp.changed() {
            if self.is_some() {
                *self = None
            } else {
                *self = Some((0, 0));
            }
        }
        if let Some((a, b)) = self {
            resp | DragValue::new(a).ui(ui) | DragValue::new(b).ui(ui)
        } else {
            resp
        }
    }
}
