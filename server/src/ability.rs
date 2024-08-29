use super::*;

#[spacetimedb(table(public))]
pub struct TAbility {
    #[primarykey]
    pub name: String,
    pub description: String,
    pub effect: String,
}
