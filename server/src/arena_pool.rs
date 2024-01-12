use super::*;

#[spacetimedb(table)]
pub struct ArenaPool {
    #[primarykey]
    #[autoinc]
    pub id: u64,
    pub owner: u64,
    pub round: u8,
    pub team: String,
}
