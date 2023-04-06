use super::*;

#[derive(Debug, Clone)]
pub struct CorpseComponent {
    pub faction: Faction,
    pub rank: u8,
    pub killer: legion::Entity,
}

impl Into<UnitComponent> for CorpseComponent {
    fn into(self) -> UnitComponent {
        UnitComponent::new(default(), self.faction, self.rank)
    }
}

impl CorpseComponent {
    pub fn from_unit(unit: UnitComponent, killer: legion::Entity) -> Self {
        Self {
            faction: unit.faction,
            rank: unit.rank,
            killer,
        }
    }
}
