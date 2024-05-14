use spacetimedb::Timestamp;

use super::*;

#[spacetimedb(table)]
pub struct ArenaArchive {
    #[primarykey]
    pub id: u64,
    pub user_id: u64,
    pub round: u32,
    pub wins: u32,
    pub loses: u32,
    pub team: Vec<TableUnit>,
    pub season: u32,
    pub timestamp: Timestamp,
}
