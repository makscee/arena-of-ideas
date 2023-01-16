use super::*;

mod faction;
mod stats;
mod team;

pub use faction::*;
pub use stats::*;
pub use team::*;

#[derive(Serialize, Deserialize, HasId, Clone)]
pub struct Unit {
    pub id: Id,
    pub name: Name,
    pub stats: UnitStats,
    pub faction: Faction,
}

impl Unit {
    pub fn new(id: Id, name: Name, stats: UnitStats, faction: Faction) -> Self {
        Self {
            id,
            name,
            stats,
            faction,
        }
    }
}
