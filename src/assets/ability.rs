use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Ability {
    pub name: Name,
    pub description: Description,
}
