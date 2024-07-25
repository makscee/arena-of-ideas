use arena::{ArenaSettings, RaritySettings};

use super::*;

#[spacetimedb(table)]
pub struct GlobalSettings {
    #[unique]
    always_zero: u32,
    pub arena: ArenaSettings,
    pub rarities: RaritySettings,
    pub battle: BattleSettings,
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

#[derive(SpacetimeType)]
pub struct BattleSettings {
    pub fatigue_start: u32,
    pub deafness_start: u32,
    pub deafness_per_turn: f32,
}
