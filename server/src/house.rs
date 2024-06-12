use super::*;

#[spacetimedb(table)]
pub struct THouse {
    #[primarykey]
    pub name: String,
    pub color: String,
    pub abilities: Vec<String>,
    pub statuses: Vec<String>,
    pub summons: Vec<String>,
}
