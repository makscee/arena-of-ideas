use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Faction {
    Player,
    Enemy,
}
