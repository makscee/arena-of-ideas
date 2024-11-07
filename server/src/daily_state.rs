use spacetimedb::Table;

use super::*;

#[spacetimedb::table(public, name = daily_state)]
#[derive(Default)]
pub struct TDailyState {
    #[primary_key]
    pub owner: u64,
    pub ranked_cost: i64,
    pub const_cost: i64,
    pub quests_taken: Vec<u64>,
    pub meta_shop_discount_spent: bool,
}

impl TDailyState {
    pub fn get(ctx: &ReducerContext, owner: u64) -> Self {
        ctx.db
            .daily_state()
            .owner()
            .find(owner)
            .unwrap_or_else(|| ctx.db.daily_state().insert(Self { owner, ..default() }))
    }
    pub fn daily_refresh(ctx: &ReducerContext) {
        for d in ctx.db.daily_state().iter() {
            ctx.db.daily_state().delete(d);
        }
    }
    pub fn buy_ranked(mut self, ctx: &ReducerContext) -> i64 {
        let price = self.ranked_cost;
        self.ranked_cost = GlobalSettings::get(ctx).arena.ranked_cost;
        self.save(ctx);
        price
    }
    pub fn buy_const(mut self, ctx: &ReducerContext) -> i64 {
        let price = self.const_cost;
        self.const_cost += GlobalSettings::get(ctx).arena.const_cost;
        self.save(ctx);
        price
    }
    pub fn take_quest(mut self, ctx: &ReducerContext, id: u64) -> Result<(), String> {
        if self.quests_taken.len() as u32 >= GlobalSettings::get(ctx).quest.daily_limit {
            return Err("Daily quest limit reached".into());
        }
        if self.quests_taken.contains(&id) {
            return Err(format!("Quest#{id} already accepted"));
        }
        self.quests_taken.push(id);
        self.save(ctx);
        Ok(())
    }
    pub fn meta_shop_discount(mut self, ctx: &ReducerContext) -> bool {
        if !self.meta_shop_discount_spent {
            self.meta_shop_discount_spent = true;
            self.save(ctx);
            true
        } else {
            false
        }
    }
    fn save(self, ctx: &ReducerContext) {
        ctx.db.daily_state().owner().update(self);
    }
}
