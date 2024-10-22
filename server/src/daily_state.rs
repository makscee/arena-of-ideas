use super::*;

#[spacetimedb(table(public))]
#[derive(Default)]
pub struct TDailyState {
    #[primarykey]
    pub owner: u64,
    pub ranked_cost: i64,
    pub const_cost: i64,
    pub quests_taken: Vec<u64>,
}

impl TDailyState {
    pub fn get(owner: u64) -> Self {
        Self::filter_by_owner(&owner)
            .unwrap_or_else(|| Self::insert(Self { owner, ..default() }).unwrap())
    }
    pub fn daily_refresh() {
        for d in Self::iter() {
            Self::delete_by_owner(&d.owner);
        }
    }
    pub fn buy_ranked(mut self) -> i64 {
        let price = self.ranked_cost;
        self.ranked_cost = GlobalSettings::get().arena.ranked_cost;
        self.save();
        price
    }
    pub fn buy_const(mut self) -> i64 {
        let price = self.const_cost;
        self.const_cost += GlobalSettings::get().arena.const_cost;
        self.save();
        price
    }
    pub fn take_quest(mut self, id: u64) -> Result<(), String> {
        if self.quests_taken.len() as u32 >= GlobalSettings::get().quest.daily_limit {
            return Err("Daily quest limit reached".into());
        }
        if self.quests_taken.contains(&id) {
            return Err(format!("Quest#{id} already accepted"));
        }
        self.quests_taken.push(id);
        self.save();
        Ok(())
    }
    fn save(self) {
        Self::update_by_owner(&self.owner.clone(), self);
    }
}
