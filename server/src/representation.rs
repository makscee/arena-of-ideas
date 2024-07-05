use super::*;

#[spacetimedb(table)]
pub struct TRepresentation {
    #[unique]
    id: String,
    data: String,
}
