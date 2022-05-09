use super::*;

#[derive(Clone)]
pub struct UnitCard {
    pub unit: Unit,
    pub template: UnitTemplate,
    pub state: CardState,
}

#[derive(Debug, Clone)]
pub enum CardState {
    Shop { index: usize },
    Party { index: usize },
    Inventory { index: usize },
    Dragged { old_state: Box<CardState> },
}

impl UnitCard {
    pub fn new(template: UnitTemplate, state: CardState) -> Self {
        Self {
            unit: Unit::new(&template, 0, "Hero".to_owned(), Faction::Player, Vec2::ZERO),
            template,
            state,
        }
    }
}
