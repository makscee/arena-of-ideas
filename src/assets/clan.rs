use super::*;

#[derive(Serialize, Deserialize)]
pub struct Clan {
    pub name: Name,
    pub color: Rgba<f32>,
    pub abilities: Vec<Ability>,
}
