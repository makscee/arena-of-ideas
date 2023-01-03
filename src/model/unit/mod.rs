use super::*;

mod stats;
mod team;

pub use stats::*;
pub use team::*;

#[derive(Serialize, Deserialize, HasId, Clone)]
pub struct Unit {
    pub id: Id,
    pub name: Name,
    pub stats: UnitStats,
}
