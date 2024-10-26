use spacetimedb::Timestamp;

use super::*;

#[spacetimedb(table(public))]
#[derive(Clone)]
pub struct TBattle {
    #[primarykey]
    pub id: u64,
    pub mode: GameMode,
    pub owner: u64,
    pub team_left: u64,
    pub team_right: u64,
    pub result: TBattleResult,
    ts: Timestamp,
}

#[derive(SpacetimeType, Default, Copy, Clone, PartialEq, Eq)]
pub enum TBattleResult {
    #[default]
    Tbd,
    Left,
    Right,
    Even,
}

impl TBattleResult {
    pub fn is_win(self) -> bool {
        self == Self::Left
    }
}

impl TBattle {
    pub fn new(mode: GameMode, owner: u64, team_left: u64, team_right: u64) -> u64 {
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
        GlobalEvent::BattleFinish(self.clone()).post(self.owner);
        self
    }
    pub fn is_tbd(&self) -> bool {
        matches!(self.result, TBattleResult::Tbd)
    }
    pub fn get(id: u64) -> Result<Self, String> {
        Self::filter_by_id(&id).context_str("TBattle not found")
    }
    pub fn save(self) {
        Self::update_by_id(&self.id.clone(), self);
    }
}
