use super::*;

#[spacetimedb(table)]
pub struct TStatus {
    #[primarykey]
    pub name: String,
    pub description: String,
    pub polarity: i8,
    pub trigger: String,
}
