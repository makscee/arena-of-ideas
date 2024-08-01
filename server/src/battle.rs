use spacetimedb::Timestamp;

use super::*;

#[spacetimedb(table)]
pub struct TBattle {
    #[primarykey]
    pub id: GID,
    pub mode: GameMode,
    pub owner: GID,
    pub team_left: GID,
    pub team_right: GID,
    result: TBattleResult,
    ts: Timestamp,
}

#[derive(SpacetimeType, Default, Copy, Clone)]
pub enum TBattleResult {
    #[default]
    Tbd,
    Left,
    Right,
    Even,
}

impl TBattle {
    pub fn new(mode: GameMode, owner: GID, team_left: GID, team_right: GID) -> GID {
        let id = next_id();
        TBattle::insert(TBattle {
            id,
            mode,
            owner,
            team_left,
            team_right,
            ts: Timestamp::now(),
            result: TBattleResult::default(),
        })
        .expect("Failed to insert TBattle");
        id
    }
    pub fn set_result(mut self, result: TBattleResult) -> Self {
        self.result = result;
        self.ts = Timestamp::now();
        self
    }
    pub fn is_tbd(&self) -> bool {
        matches!(self.result, TBattleResult::Tbd)
    }
    pub fn get(id: GID) -> Result<Self, String> {
        Self::filter_by_id(&id).context_str("TBattle not found")
    }
    pub fn save(self) {
        Self::update_by_id(&self.id.clone(), self);
    }
}
