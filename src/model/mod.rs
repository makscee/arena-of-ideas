mod unit;

use super::*;
pub use unit::*;
pub struct Model {
    pub battle_units: Collection<Unit>,
    pub game_time: Time,
}

impl Model {
    pub fn new() -> Self {
        Self {
            battle_units: default(),
            game_time: 0.0,
        }
    }
}
