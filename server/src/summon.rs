use crate::user_access::UserRight;

use super::*;

#[spacetimedb(table)]
pub struct Summon {
    #[primarykey]
    pub name: String,
    pub data: String,
}
