use super::*;

#[spacetimedb(table)]
pub struct TBattle {
    #[primarykey]
    pub id: GID,
    pub owner: GID,
    pub team_left: GID,
    pub team_right: GID,
    pub result: TBattleResult,
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
    pub fn new(owner: GID, team_left: GID, team_right: GID) -> GID {
        let id = next_id();
        TBattle::insert(TBattle {
            id,
            owner,
            team_left,
            team_right,
            result: TBattleResult::default(),
        })
        .expect("Failed to insert TBattle");
        id
    }
    pub fn get(id: GID) -> Result<Self, String> {
        Self::filter_by_id(&id).context_str("TBattle not found")
    }
    pub fn save(self) {
        Self::update_by_id(&self.id.clone(), self);
    }
}
