use super::*;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone, Default)]
pub enum Faction {
    #[default]
    Player,
    Enemy,
}
