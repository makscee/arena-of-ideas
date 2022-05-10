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
    pub fn new(template: UnitTemplate, unit_type: UnitType) -> Self {
        Self {
            unit: Unit::new(&template, 0, unit_type, Faction::Player, Vec2::ZERO),
            template,
        }
    }
}
