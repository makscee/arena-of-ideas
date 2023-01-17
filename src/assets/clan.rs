use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Clan {
    pub name: Name,
    pub color: Rgba<f32>,
    pub abilities: Vec<Ability>,
}