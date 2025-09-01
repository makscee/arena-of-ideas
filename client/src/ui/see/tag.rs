use super::*;

pub trait SFnTag {
    fn see_tag(&self, context: &Context, ui: &mut Ui) -> Response;
}

impl SFnTag for NUnit {
    fn see_tag(&self, context: &Context, ui: &mut Ui) -> Response {
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
        TagWidget::new_name_value(
            context.get_string(VarName::unit_name).unwrap_or_default(),
            context.get_color(VarName::color).unwrap_or(MISSING_COLOR),
            format!(
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
            ),
        )
        .ui(ui)
    }
}

impl SFnTag for NHouse {
    fn see_tag(&self, context: &Context, ui: &mut Ui) -> Response {
        let color = context.color(ui);
        TagWidget::new_name(&self.house_name, color).ui(ui)
    }
}

impl SFnTag for NAbilityMagic {
    fn see_tag(&self, context: &Context, ui: &mut Ui) -> Response {
        let color = context.color(ui);
        TagWidget::new_name(&self.ability_name, color).ui(ui)
    }
}

impl SFnTag for NStatusMagic {
    fn see_tag(&self, context: &Context, ui: &mut Ui) -> Response {
        let color = context.color(ui);
        TagWidget::new_name(&self.status_name, color).ui(ui)
    }
}
