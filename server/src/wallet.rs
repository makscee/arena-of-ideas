use super::*;

#[spacetimedb::table(public, name = wallet)]
pub struct TWallet {
    #[primary_key]
    owner: u64,
    amount: i64,
}

impl TWallet {
    pub fn new(ctx: &ReducerContext, owner: u64) -> Result<(), String> {
        let d = Self { owner, amount: 0 };
        ctx.db.wallet().insert(d);
        Ok(())
    }
    pub fn change(ctx: &ReducerContext, owner: u64, delta: i64) -> Result<(), String> {
        let mut w = Self::get(ctx, owner)?;
        w.amount += delta;
        if w.amount < 0 {
            return Err("Insufficient funds".into());
        }
        w.save(ctx);
        Ok(())
    }
    pub fn get(ctx: &ReducerContext, owner: u64) -> Result<Self, String> {
        ctx.db
            .wallet()
            .owner()
            .find(owner)
            .to_e_s("Wallet not found")
    }
    pub fn save(self, ctx: &ReducerContext) {
        ctx.db.wallet().owner().update(self);
    }
}

#[spacetimedb::reducer]
fn give_credits(ctx: &ReducerContext) -> Result<(), String> {
    ctx.is_admin()?;
    let player = TPlayer::find_by_identity(ctx, &ctx.sender)?;
    TWallet::change(ctx, player.id, 100)?;
    Ok(())
}
