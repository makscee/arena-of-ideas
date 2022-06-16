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
    pub fn new(template: UnitTemplate, unit_type: UnitType, statuses: &Statuses) -> Self {
        Self {
            unit: Unit::new(
                &template,
                &mut 0,
                unit_type,
                Faction::Player,
                Vec2::ZERO,
                statuses,
            ),
            template,
        }
    }
}
