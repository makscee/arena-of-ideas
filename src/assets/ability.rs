use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ability {
    pub name: Name,
    pub description: Description,
}
