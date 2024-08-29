use super::*;

#[spacetimedb(table(public))]
pub struct TRepresentation {
    #[unique]
    id: String,
    data: String,
}
