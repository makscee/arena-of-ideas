use super::*;

#[spacetimedb(table)]
pub struct BaseUnit {
    #[primarykey]
    pub name: String,
    pub hp: i32,
    pub pwr: i32,
    pub house: String,
    pub rarity: i8,
    pub repr: u64,
    pub triggers: Vec<String>,
    pub targets: Vec<String>,
    pub effects: Vec<String>,
}
