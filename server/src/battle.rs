use spacetimedb::{Table, Timestamp};

use super::*;

#[spacetimedb::table(public, name = battle)]
#[derive(Clone)]
pub struct TBattle {
    #[primary_key]
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
    pub fn new(
        ctx: &ReducerContext,
        mode: GameMode,
        owner: u64,
        team_left: u64,
        team_right: u64,
    ) -> u64 {
        let id = next_id(ctx);
        ctx.db.battle().insert(Self {
            id,
            mode,
            owner,
            team_left,
            team_right,
            ts: Timestamp::now(),
            result: TBattleResult::default(),
        });
        id
    }
    pub fn set_result(mut self, ctx: &ReducerContext, result: TBattleResult) -> Self {
        self.result = result;
        self.ts = Timestamp::now();
        GlobalEvent::BattleFinish(self.clone()).post(ctx, self.owner);
        self
    }
    pub fn is_tbd(&self) -> bool {
        matches!(self.result, TBattleResult::Tbd)
    }
    pub fn get(ctx: &ReducerContext, id: u64) -> Result<Self, String> {
        ctx.db
            .battle()
            .id()
            .find(id)
            .context_str("TBattle not found")
    }
    pub fn save(self, ctx: &ReducerContext) {
        ctx.db.battle().id().update(self);
    }
}
