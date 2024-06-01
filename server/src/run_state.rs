use super::*;

#[spacetimedb(table)]
pub struct RunState {
    #[primarykey]
    id: u64,
    #[unique]
    user_id: u64,

    team: Vec<FusedUnit>,
}
