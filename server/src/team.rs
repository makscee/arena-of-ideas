use super::*;

#[spacetimedb(table(public))]
#[derive(Clone)]
pub struct TTeam {
    #[primarykey]
    pub id: u64,
    pub name: String,
    pub owner: u64,
    pub units: Vec<FusedUnit>,
    pub pool: TeamPool,
}

#[derive(SpacetimeType, Clone, Copy)]
pub enum TeamPool {
    Owned,
    Arena,
    Enemy,
}

impl TTeam {
    pub fn get(id: u64) -> Result<Self, String> {
        Self::filter_by_id(&id).context_str("Team not found")
    }
    #[must_use]
    pub fn new(owner: u64, pool: TeamPool) -> Self {
        Self {
            id: next_id(),
            name: String::new(),
            owner,
            units: Vec::new(),
            pool,
        }
    }
    #[must_use]
    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }
    #[must_use]
    pub fn units(mut self, units: Vec<FusedUnit>) -> Self {
        self.units = units;
        self
    }
    pub fn save(self) -> u64 {
        Self::delete_by_id(&self.id);
        Self::insert(self).unwrap().id
    }
    pub fn save_clone(&self) -> Self {
        let mut c = self.clone();
        c.id = next_id();
        TTeam::insert(c).expect("Failed to clone team")
    }
    pub fn get_unit(&self, i: u8) -> Result<&FusedUnit, String> {
        self.units
            .get(i as usize)
            .with_context_str(|| format!("Failed to find unit team#{} slot {i}", self.id))
    }
}

#[spacetimedb(reducer)]
fn new_team(ctx: ReducerContext, name: String) -> Result<(), String> {
    let user = ctx.user()?;
    TTeam::new(user.id, TeamPool::Owned).name(name).save();
    Ok(())
}
