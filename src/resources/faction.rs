use super::*;

#[derive(
    Clone, Copy, Deserialize, Serialize, Debug, PartialEq, Eq, Hash, enum_iterator::Sequence,
)]
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
    pub fn from_entity(
        entity: legion::Entity,
        world: &legion::World,
        resources: &Resources,
    ) -> Faction {
        if let Ok(entry) = world.entry_ref(entity) {
            if let Ok(unit) = entry.get_component::<UnitComponent>() {
                return unit.faction;
            } else if let Ok(corpse) = entry.get_component::<CorpseComponent>() {
                return corpse.faction;
            }
        }
        panic!("Entity faction not found {:?}", entity)
    }
    pub fn opposite(&self) -> Faction {
        match self {
            Faction::Light => Faction::Dark,
            Faction::Dark => Faction::Light,
            Faction::Team => Faction::Shop,
            Faction::Shop => Faction::Team,
            Faction::Gallery => Faction::Gallery,
        }
    }
}
