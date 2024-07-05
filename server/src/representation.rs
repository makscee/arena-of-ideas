use super::*;

#[spacetimedb(table)]
pub struct TRepresentation {
    #[primarykey]
    id: u64,
    data: String,
}
