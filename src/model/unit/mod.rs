use crate::assets::Clan;

use super::*;

mod faction;
mod stats;
mod team;
mod position;

pub use faction::*;
pub use stats::*;
pub use team::*;
pub use position::*;

#[derive(Serialize, Deserialize, HasId, Clone)]
pub struct Unit {
    pub id: Id,
    pub name: Name,
    pub stats: UnitStats,
    pub faction: Faction,
    pub position: Position,
    pub clans: Vec<Clan>,
    pub all_statuses: Vec<AttachedStatus>,
    pub vars: HashMap<VarName, Expr>,
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
