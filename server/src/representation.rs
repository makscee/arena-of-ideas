use super::*;

#[spacetimedb(table)]
pub struct TRepresentation {
    id: u64,
    data: String,
}
