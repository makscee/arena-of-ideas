use crate::module_bindings::finish_building_tower;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Save {
    pub mode: GameMode,
    pub climb: TowerClimb,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub enum GameMode {
    #[default]
    NewTower,
    RandomTower {
        tower_id: u64,
    },
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
    pub fn get_tower_id(&self) -> Option<u64> {
        match self.mode {
            GameMode::NewTower => None,
            GameMode::RandomTower { tower_id } => Some(tower_id),
        }
    }
    pub fn add_tower_levels(&mut self, levels: Vec<String>) -> &mut Self {
        debug!("New tower levels: {levels:#?}");
        self.climb.levels.extend(levels.clone());
        self
    }
    pub fn finish_building_tower(&mut self) -> &mut Self {
        if matches!(self.mode, GameMode::NewTower)
            && LoginPlugin::is_connected()
            && self.climb.defeated >= 3
        {
            let team = ron::to_string(&self.climb.team).unwrap();
            debug!("Finish building tower {team}");
            finish_building_tower(self.climb.levels[..=self.climb.defeated].to_vec(), team);
        }
        self
    }
    pub fn tower(&self) -> Option<TableTower> {
        let tower_id = self.get_tower_id()?;
        TableTower::filter_by_id(tower_id)
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
