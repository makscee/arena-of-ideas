use super::*;

#[derive(Clone)]
pub struct UnitCard {
    pub unit: Unit,
    pub template: UnitTemplate,
    pub game_time: Time,
}

impl UnitCard {
    pub fn new(template: UnitTemplate) -> Self {
        Self {
            unit: Unit::new(&template, 0, "Hero".to_owned(), Faction::Player, Vec2::ZERO),
            game_time: Time::new(0.0),
            template,
        }
    }
}
