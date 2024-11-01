use super::*;

#[spacetimedb(table(public))]
pub struct TRepresentation {
    #[unique]
    pub id: String,
    pub data: String,
}
