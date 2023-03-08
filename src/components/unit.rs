use super::*;

#[derive(Debug, Clone, Copy)]
pub struct UnitComponent {
    pub slot: usize,
    pub faction: Faction,
}

pub const DEFAULT_UNIT_RADIUS: f32 = 1.0;

#[derive(Clone, Copy, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum Faction {
    Light,
    Dark,
    Team,
    Shop,
    Gallery,
}

impl Faction {
    pub fn color(&self, options: &Options) -> Rgba<f32> {
        *options.colors.faction_colors.get(self).unwrap()
    }
}

impl VarsProvider for UnitComponent {
    fn extend_vars(&self, vars: &mut Vars) {
        let faction_val = match self.faction {
            Faction::Dark => 0.0,
            Faction::Light => 1.0,
            Faction::Team => 2.0,
            Faction::Shop => 3.0,
            Faction::Gallery => 4.0,
        };
        vars.insert(VarName::Faction, Var::Float(faction_val));
        vars.insert(VarName::Slot, Var::Int(self.slot as i32));
    }
}
