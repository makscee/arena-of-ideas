use crate::assets::Clan;

use super::*;

mod faction;
mod stats;
mod team;

pub use faction::*;
pub use stats::*;
pub use team::*;

#[derive(Deserialize, HasId, Clone)]
pub struct Unit {
    #[serde(skip)]
    pub id: Id,
    #[serde(skip)]
    pub slot: i32,
    #[serde(skip)]
    pub faction: Faction,
    pub name: Name,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tier: i32,
    pub stats: UnitStats,
    // #[serde(default)]
    // pub statuses: Vec<Status>,
    // #[serde(default)]
    // pub action: Effect,
    #[serde(default)]
    pub clans: Vec<String>,
    #[serde(default = "default_renders")]
    pub layers: Vec<ShaderProgram>,
    #[serde(default)]
    pub vars: HashMap<VarName, Expr>,
}

fn default_renders() -> Vec<ShaderProgram> {
    let result: Vec<ShaderProgram> = vec![];
    result
}

impl Unit {
    pub fn faction(mut self, faction: Faction) -> Self {
        self.faction = faction;
        self
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }
}
