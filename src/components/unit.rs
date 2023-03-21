use super::*;

#[derive(Debug, Clone)]
pub struct UnitComponent {
    pub slot: usize,
    pub faction: Faction,
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug, PartialEq, Eq, Hash)]
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
    pub fn float_value(&self) -> f32 {
        match self {
            Faction::Dark => 0.0,
            Faction::Light => 1.0,
            Faction::Team => 2.0,
            Faction::Shop => 3.0,
            Faction::Gallery => 4.0,
        }
    }
    pub fn from_entity(entity: legion::Entity, world: &legion::World) -> Faction {
        world
            .entry_ref(entity)
            .unwrap()
            .get_component::<UnitComponent>()
            .unwrap()
            .faction
    }
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
