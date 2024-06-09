use super::*;

#[derive(Asset, Serialize, Deserialize, Clone, Debug, TypePath, PartialEq)]
pub struct House {
    pub name: String,
    pub color: HexColor,
    #[serde(default)]
    pub abilities: Vec<Ability>,
    #[serde(default)]
    pub statuses: Vec<PackedStatus>,
    #[serde(default)]
    pub summons: Vec<PackedUnit>,
}
