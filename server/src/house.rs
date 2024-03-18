use super::*;

#[spacetimedb(table)]
pub struct House {
    #[primarykey]
    pub name: String,
    pub data: String,
}
