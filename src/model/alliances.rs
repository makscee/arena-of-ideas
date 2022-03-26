use super::*;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Alliance {
    Spawners,
    Assassins,
    Critters,
    Archers,
    Freezers,
    Warriors,
    Healers,
    Vampires,
    Exploders,
}
