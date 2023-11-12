use super::*;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Save {
    pub team: PackedTeam,
    pub ladder: Ladder,
    pub current_level: usize,
}

impl Save {
    pub fn save(&self, world: &mut World) -> Result<()> {
        debug!("Saving {self:?}");
        world
            .get_resource_mut::<PkvStore>()
            .unwrap()
            .set("save", self)
            .map_err(|e| anyhow!("{}", e.to_string()))
    }
    pub fn get(world: &World) -> Result<Save> {
        world
            .get_resource::<PkvStore>()
            .unwrap()
            .get::<Save>("save")
            .map_err(|e| anyhow!("{}", e.to_string()))
    }
    pub fn set_team(&mut self, team: PackedTeam) -> &mut Self {
        self.team = team;
        self
    }
    pub fn set_ladder(&mut self, ladder: Ladder) -> &mut Self {
        self.ladder = ladder;
        self
    }
    pub fn add_ladder_level(&mut self, team: PackedTeam) -> &mut Self {
        debug!("New ladder level: {team:?}");
        self.ladder.teams.push(team);
        self
    }
    pub fn set_current_level(&mut self, ind: usize) -> &mut Self {
        self.current_level = ind;
        self
    }
}