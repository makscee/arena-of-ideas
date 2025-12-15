mod admin;
mod audio;
mod battle;
mod battle_editor;
mod connect;
mod game_timer;
mod incubator;
mod login;
mod r#match;
mod node_state;
mod persistent_data;
mod representation;
mod rhai;
mod stdb;
mod stdb_ext;
mod tile;
mod ui;
mod world_migration;

use super::*;
pub use admin::*;
pub use audio::*;
pub use battle::*;
pub use battle_editor::*;
pub use connect::*;
pub use game_timer::*;
pub use incubator::*;
pub use login::*;
pub use r#match::*;
pub use node_state::*;
pub use persistent_data::*;
pub use representation::*;
pub use rhai::*;
pub use stdb::*;
pub use stdb_ext::*;
pub use tile::*;
pub use ui::*;
pub use world_migration::*;

#[derive(Clone, Debug)]
pub struct DraggedUnit {
    pub unit_id: u64,
    pub from_location: UnitLocation,
}

#[derive(Clone, Debug)]
pub enum UnitLocation {
    Bench,
    Slot { index: i32 },
}
