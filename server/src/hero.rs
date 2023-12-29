use super::*;

#[spacetimedb(table)]
pub struct Hero {
    #[primarykey]
    pub name: String,
    pub data: String,
}

// #[spacetimedb(reducer)]
