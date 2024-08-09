use super::*;

#[spacetimedb(table)]
#[derive(Clone)]
pub struct TTeam {
    #[primarykey]
    pub id: u64,
    pub owner: u64,
    pub units: Vec<FusedUnit>,
}

impl TTeam {
    pub fn get(id: u64) -> Result<Self, String> {
        Self::filter_by_id(&id).context_str("Team not found")
    }
    pub fn save(self) {
        Self::update_by_id(&self.id.clone(), self);
    }
    pub fn new(owner: u64) -> u64 {
        let team = Self::insert(Self {
            id: next_id(),
            owner,
            units: Vec::new(),
        })
        .unwrap();
        team.id
    }
    pub fn new_with(owner: u64, units: Vec<FusedUnit>) -> u64 {
        let team = Self {
            id: next_id(),
            owner,
            units,
        };
        Self::insert(team).unwrap().id
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
