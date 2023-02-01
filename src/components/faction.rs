use super::*;

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub enum Faction {
    Light,
    Dark,
}

impl VarsProvider for Faction {
    fn extend_vars(&self, vars: &mut Vars) {
        vars.insert(
            VarName::Faction,
            Var::Int(match self {
                Faction::Light => -1,
                Faction::Dark => 1,
            }),
        );
    }
}
