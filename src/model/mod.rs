mod unit;

use super::*;
pub use unit::*;
pub struct Model {
    pub units: Collection<Unit>,
    pub player_team: Team,
    pub enemy_team: Team,
}
