use super::*;

#[derive(Debug, Clone)]
pub struct UnitComponent {
    pub slot: usize,
    pub faction: Faction,
}

impl VarsProvider for UnitComponent {
    fn extend_vars(&self, vars: &mut Vars, resources: &Resources) {
        vars.insert(VarName::Faction, Var::Float(self.faction.float_value()));
        vars.insert(VarName::Slot, Var::Int(self.slot as i32));
        vars.insert(
            VarName::FactionColor,
            Var::Color(
                *resources
                    .options
                    .colors
                    .faction_colors
                    .get(&self.faction)
                    .unwrap(),
            ),
        );
    }
}
