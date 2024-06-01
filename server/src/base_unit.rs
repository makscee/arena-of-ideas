use super::*;

#[spacetimedb(table)]
pub struct BaseUnit {
    #[primarykey]
    name: String,
    hp: i32,
    pwr: i32,
    house: String,
    rarity: i8,
    repr: u64,
    triggers: Vec<String>,
    targets: Vec<String>,
    effects: Vec<String>,
}
