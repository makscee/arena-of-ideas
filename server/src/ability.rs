use crate::user_access::UserRight;

use super::*;

#[spacetimedb(table)]
pub struct Ability {
    #[primarykey]
    pub name: String,
    pub data: String,
}
