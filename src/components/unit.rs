use super::*;

#[derive(Debug, Clone)]
pub struct UnitComponent {
    pub slot: usize,
    pub faction: Faction,
    pub rank: u8,
}

impl UnitComponent {
    pub fn new(slot: usize, faction: Faction, rank: u8) -> Self {
        Self {
            slot,
            faction,
            rank,
        }
    }
}

impl VarsProvider for UnitComponent {
    fn extend_vars(&self, vars: &mut Vars, resources: &Resources) {
        vars.insert(VarName::Faction, Var::Float(self.faction.float_value()));
        vars.insert(VarName::Slot, Var::Int(self.slot as i32));
        vars.insert(VarName::Rank, Var::Int(self.rank as i32));
        vars.insert(
            VarName::FactionColor,
            Var::Color(
                *resources
                    .options
                    .colors
                    .factions
                    .get(&self.faction)
                    .unwrap(),
            ),
        );
    }
}
