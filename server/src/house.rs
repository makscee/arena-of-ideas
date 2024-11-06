use super::*;

#[spacetimedb::table(name = house)]
pub struct THouse {
    #[primary_key]
    pub name: String,
    pub color: String,
    pub abilities: Vec<String>,
    pub statuses: Vec<String>,
    pub summons: Vec<String>,
    pub defaults: String,
}
