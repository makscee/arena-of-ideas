use arena::{ArenaSettings, RaritySettings};

use super::*;

#[spacetimedb(table)]
pub struct GlobalSettings {
    #[unique]
    always_zero: u32,
    pub arena: ArenaSettings,
    pub rarities: RaritySettings,
}

impl GlobalSettings {
    pub fn get() -> Self {
        GlobalSettings::filter_by_always_zero(&0).unwrap()
    }
    pub fn replace(self) {
        GlobalSettings::delete_by_always_zero(&0);
        let _ = GlobalSettings::insert(self);
    }
}
