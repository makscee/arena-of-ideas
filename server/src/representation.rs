use super::*;

#[spacetimedb(table)]
pub struct Representation {
    id: u64,
    data: String,
}
