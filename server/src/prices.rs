use super::*;

#[spacetimedb(table(public))]
#[derive(Default)]
pub struct TPrices {
    #[primarykey]
    pub owner: u64,
    pub ranked_mode: i64,
    pub const_mode: i64,
}

impl TPrices {
    pub fn new(owner: u64) -> Result<(), String> {
        let d = Self { owner, ..default() };
        Self::insert(d.refresh())?;
        Ok(())
    }
    fn refresh(self) -> Self {
        Self {
            owner: self.owner,
            ..default()
        }
    }
    pub fn refresh_all() {
        for d in Self::iter() {
            d.refresh().save();
        }
    }
    pub fn buy_ranked(mut self) -> i64 {
        let price = self.ranked_mode;
        self.ranked_mode = GlobalSettings::get().arena.ranked_cost;
        self.save();
        price
    }
    pub fn buy_const(mut self) -> i64 {
        let price = self.const_mode;
        self.const_mode = GlobalSettings::get().arena.const_cost;
        self.save();
        price
    }
    fn save(self) {
        Self::update_by_owner(&self.owner.clone(), self);
    }
}
