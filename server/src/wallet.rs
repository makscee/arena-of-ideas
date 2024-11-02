use super::*;

#[spacetimedb(table(public))]
pub struct TWallet {
    #[primarykey]
    owner: u64,
    amount: i64,
}

impl TWallet {
    pub fn new(owner: u64) -> Result<(), String> {
        let d = Self { owner, amount: 0 };
        Self::insert(d)?;
        Ok(())
    }
    pub fn change(owner: u64, delta: i64) -> Result<(), String> {
        let mut w = Self::get(owner)?;
        w.amount += delta;
        if w.amount < 0 {
            return Err("Insufficient funds".into());
        }
        if delta > 0 {
            TUserStats::register_credits_earned(owner, delta as u32);
        }
        w.save();
        Ok(())
    }
    pub fn get(owner: u64) -> Result<Self, String> {
        Self::filter_by_owner(&owner).context_str("Wallet not found")
    }
    pub fn save(self) {
        Self::update_by_owner(&self.owner.clone(), self);
    }
}

#[spacetimedb(reducer)]
fn give_credits(ctx: ReducerContext) -> Result<(), String> {
    ctx.is_admin()?;
    let user = TUser::find_by_identity(&ctx.sender)?;
    TWallet::change(user.id, 100)?;
    Ok(())
}
