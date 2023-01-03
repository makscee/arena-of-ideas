use super::*;

#[derive(Serialize, Deserialize)]
pub struct Ability {
    pub name: Name,
    pub description: Description,
}
