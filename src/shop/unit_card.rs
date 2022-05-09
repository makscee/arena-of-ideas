use super::*;

#[derive(Clone)]
pub struct UnitCard {
    pub unit: Unit,
    pub template: UnitTemplate,
}

#[derive(Debug, Clone)]
pub enum CardState {
    Shop { index: usize },
    Party { index: usize },
    Inventory { index: usize },
}

impl UnitCard {
    pub fn new(template: UnitTemplate) -> Self {
        Self {
            unit: Unit::new(&template, 0, "Hero".to_owned(), Faction::Player, Vec2::ZERO),
            template,
        }
    }
}
