use super::*;

#[spacetimedb(table)]
pub struct Vfx {
    #[primarykey]
    pub name: String,
    pub data: String,
}
