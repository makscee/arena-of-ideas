use crate::assets::Clan;

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
    pub slot: i32,
    pub faction: Faction,
    pub clans: Vec<Clan>,
    pub vars: HashMap<VarName, Expr>,
}

impl Unit {
    pub fn new(
        id: Id,
        name: Name,
        stats: UnitStats,
        slot: i32,
        faction: Faction,
        clans: Vec<Clan>,
        vars: HashMap<VarName, Expr>,
    ) -> Self {
        Self {
            id,
            name,
            stats,
            slot,
            faction,
            clans,
            vars,
        }
    }

    pub fn new_test(id: Id, faction: Faction) -> Self {
        Self {
            id,
            name: format!("Test#{}", id),
            stats: UnitStats {
                health: 1,
                attack: 1,
                stacks: 1,
            },
            slot: 1,
            faction,
            clans: default(),
            vars: default(),
        }
    }
}
