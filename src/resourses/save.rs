use super::*;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Save {
    pub team: PackedTeam,
    pub ladder: Ladder,
    pub current_level: usize,
}

impl Save {
    pub fn save(&self, world: &mut World) -> Result<()> {
        debug!("Saving {self:?}");
        world
            .resource_mut::<PkvStore>()
            .set("save", self)
            .map_err(|e| anyhow!("{}", e.to_string()))
    }
    pub fn get(world: &World) -> Save {
        world
            .resource::<PkvStore>()
            .get::<Save>("save")
            .unwrap_or_default()
    }
    pub fn set_team(&mut self, team: PackedTeam) -> &mut Self {
        self.team = team;
        self
    }
    pub fn set_ladder(&mut self, ladder: Ladder) -> &mut Self {
        self.ladder = ladder;
        self
    }
    pub fn add_ladder_levels(&mut self, mut teams: Vec<PackedTeam>) -> &mut Self {
        debug!("New ladder levels: {teams:#?}");
        self.ladder.teams.append(&mut teams);
        self
    }

    pub fn store_current(world: &mut World) -> Result<()> {
        PersistentData::load(world)
            .set_stored_save(Self::get(world))
            .save(world)?;
        Ok(())
    }

    pub fn load_stored(world: &mut World) -> Result<()> {
        let save = PersistentData::load(world).stored_save;
        world
            .resource_mut::<PkvStore>()
            .set("save", &save)
            .map_err(|e| anyhow!("{}", e.to_string()))
    }
}
