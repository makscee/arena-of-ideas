use super::*;

#[spacetimedb(table)]
pub struct TWallet {
    #[primarykey]
    owner: GID,
    amount: i64,
}

impl TWallet {
    pub fn new(owner: GID) -> Result<(), String> {
        let d = Self { owner, amount: 0 };
        Self::insert(d)?;
        Ok(())
    }
    pub fn change(owner: GID, delta: i64) -> Result<(), String> {
        let mut w = Self::get(owner)?;
        w.amount += delta;
        w.save();
        Ok(())
    }
    pub fn get(owner: GID) -> Result<Self, String> {
        Self::filter_by_owner(&owner).context_str("Wallet not found")
    }
    pub fn save(self) {
        Self::update_by_owner(&self.owner.clone(), self);
    }
}
