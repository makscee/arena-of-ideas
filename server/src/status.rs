use super::*;

#[spacetimedb(table)]
pub struct Statuses {
    #[primarykey]
    pub name: String,
    pub data: String,
}
