use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Save {
    pub mode: GameMode,
    pub climb: TowerClimb,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub enum GameMode {
    #[default]
    GlobalTower,
}

impl Save {
    pub fn save(&self, world: &mut World) -> Result<()> {
        debug!("Saving {self:?}");
        world
            .resource_mut::<PkvStore>()
            .set("save", self)
            .map_err(|e| anyhow!(e.to_string()))
    }
    pub fn get(world: &World) -> Result<Save> {
        world
            .resource::<PkvStore>()
            .get::<Save>("save")
            .map_err(|e| anyhow!(e.to_string()))
    }
    pub fn clear(world: &mut World) -> Result<()> {
        world
            .resource_mut::<PkvStore>()
            .set_string("save", "")
            .map_err(|e| anyhow!(e.to_string()))
    }
    pub fn set_team(&mut self, team: PackedTeam) -> &mut Self {
        self.climb.team = team;
        self
    }
    pub fn register_victory(&mut self) -> &mut Self {
        self.climb.defeated += 1;
        self
    }
    pub fn store_current(world: &mut World) -> Result<()> {
        PersistentData::load(world)
            .set_stored_save(Self::get(world)?)
            .save(world)?;
        Ok(())
    }
    pub fn load_stored(world: &mut World) -> Result<()> {
        let save = PersistentData::load(world).stored_save;
        world
            .resource_mut::<PkvStore>()
            .set("save", &save)
            .map_err(|e| anyhow!(e.to_string()))
    }
}
