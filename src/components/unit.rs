use super::*;

#[derive(Debug, Clone)]
pub struct UnitComponent {
    pub slot: usize,
    pub faction: Faction,
    pub template_path: PathBuf,
}

pub const DEFAULT_UNIT_RADIUS: f32 = 1.0;
const CARD_ANIMATION_TIME: Time = 0.3;

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
    fn extend_vars(&self, vars: &mut Vars, resources: &Resources) {
        let faction_val = match self.faction {
            Faction::Dark => 0.0,
            Faction::Light => 1.0,
            Faction::Team => 2.0,
            Faction::Shop => 3.0,
            Faction::Gallery => 4.0,
        };
        vars.insert(VarName::Faction, Var::Float(faction_val));
        vars.insert(VarName::Slot, Var::Int(self.slot as i32));

        let mut card: f32 = match self.faction {
            Faction::Shop => 1.0,
            _ => 0.0,
        };
        let hover = match vars.try_get_float(&VarName::Hovered) {
            Some(value) => (1.0
                - value
                - ((resources.global_time - vars.get_float(&VarName::HoveredTs))
                    / CARD_ANIMATION_TIME)
                    .min(1.0))
            .abs(),
            None => 0.0,
        };
        card = card.max(hover);
        vars.insert(VarName::Card, Var::Float(card));
        vars.insert(VarName::Zoom, Var::Float(1.0 + hover));
    }
}
