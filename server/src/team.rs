use super::*;

#[spacetimedb(table)]
#[derive(Clone)]
pub struct TTeam {
    #[primarykey]
    pub id: GID,
    pub owner: GID,
    pub units: Vec<FusedUnit>,
}

impl TTeam {
    pub fn get(id: GID) -> Result<Self, String> {
        TTeam::filter_by_id(&id).context_str("Team not found")
    }
    pub fn save(self) {
        TTeam::update_by_id(&self.id.clone(), self);
    }
    pub fn new(owner: GID) -> GID {
        let team = TTeam::insert(TTeam {
            id: next_id(),
            owner,
            units: Vec::new(),
        })
        .unwrap();
        team.id
    }
    pub fn save_clone(&self) -> Self {
        let mut c = self.clone();
        c.id = next_id();
        TTeam::insert(c).expect("Failed to clone team")
    }
}
