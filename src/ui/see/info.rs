use super::*;

pub trait SFnInfo {
    fn see_info_cstr(&self, ctx: &Context) -> Cstr;
}

pub trait SFnInfoDefault: NodeExt {
    fn see_info_cstr_default(&self, context: &Context) -> Cstr {
        let vars = self.get_vars(context);
        let mut info_parts = Vec::new();

        for (var_name, var_value) in vars {
            info_parts.push(format!(
                "[{} {var_name}] {var_value}",
                var_name.color().to_hex(),
                var_name = var_name,
                var_value = var_value.cstr()
            ));
        }

        let parents_count = context.parents(self.id()).len();
        if parents_count > 0 {
            info_parts.push(format!("[tw parents: {}]", parents_count));
        }

        info_parts.join(" ")
    }
}

impl SFnInfoDefault for NCore {}
impl SFnInfoDefault for NPlayers {}
impl SFnInfoDefault for NPlayer {}
impl SFnInfoDefault for NPlayerData {}
impl SFnInfoDefault for NPlayerIdentity {}
impl SFnInfoDefault for NHouseColor {}
impl SFnInfoDefault for NActionAbility {}
impl SFnInfoDefault for NActionDescription {}
impl SFnInfoDefault for NActionEffect {}
impl SFnInfoDefault for NStatusAbility {}
impl SFnInfoDefault for NStatusDescription {}
impl SFnInfoDefault for NStatusBehavior {}
impl SFnInfoDefault for NStatusRepresentation {}
impl SFnInfoDefault for NTeam {}
impl SFnInfoDefault for NMatch {}
impl SFnInfoDefault for NFusion {}
impl SFnInfoDefault for NUnitDescription {}
impl SFnInfoDefault for NUnitStats {}
impl SFnInfoDefault for NUnitState {}
impl SFnInfoDefault for NUnitBehavior {}
impl SFnInfoDefault for NUnitRepresentation {}
impl SFnInfoDefault for NArena {}
impl SFnInfoDefault for NFloorPool {}
impl SFnInfoDefault for NFloorBoss {}
impl SFnInfoDefault for NBattle {}

impl<T: SFnInfoDefault> SFnInfo for T {
    fn see_info_cstr(&self, ctx: &Context) -> Cstr {
        self.see_info_cstr_default(ctx)
    }
}

impl SFnInfo for NHouse {
    fn see_info_cstr(&self, context: &Context) -> Cstr {
        let mut info_parts = Vec::new();

        let units_count = context
            .collect_children_components::<NUnit>(self.id)
            .map(|u| u.len())
            .unwrap_or_default();
        if units_count > 0 {
            info_parts.push(format!("units: {}", units_count));
        }
        let color = self.color_for_text(context);
        if let Ok(ability) = self.action_load(context) {
            info_parts.push(ability.ability_name.cstr_c(color));
        }
        if let Ok(status) = self.status_load(context) {
            info_parts.push(status.status_name.cstr_c(color));
        }

        info_parts.join(" | ")
    }
}

impl SFnInfo for NUnit {
    fn see_info_cstr(&self, context: &Context) -> Cstr {
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
