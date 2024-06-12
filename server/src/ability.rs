use super::*;

#[spacetimedb(table)]
pub struct TAbility {
    #[primarykey]
    pub name: String,
    pub description: String,
    pub effect: String,
}
