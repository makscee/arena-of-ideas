use super::*;

#[spacetimedb(table)]
pub struct BaseUnit {
    #[primarykey]
    pub name: String,
    pub pwr: i32,
    pub hp: i32,
    pub rarity: i8,
    pub house: String,
    pub repr: u64,
    pub triggers: Vec<String>,
    pub targets: Vec<String>,
    pub effects: Vec<String>,
}
