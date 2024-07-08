use super::*;

#[spacetimedb(table)]
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
}
