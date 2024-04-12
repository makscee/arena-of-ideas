use super::*;

#[spacetimedb(table)]
#[derive(Clone)]
pub struct TableUnit {
    #[primarykey]
    pub name: String,
    pub hp: i32,
    pub atk: i32,
    pub houses: String,
    pub stacks: i32,
    pub level: i32,
    pub statuses: Vec<StatusCharges>,
    pub trigger: String,
    pub representation: String,
    pub state: String,
}

#[derive(SpacetimeType, Clone)]
pub struct StatusCharges {
    pub name: String,
    pub charges: i32,
}
