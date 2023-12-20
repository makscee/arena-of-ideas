use crate::module_bindings::finish_building_ladder;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Save {
    pub mode: GameMode,
    pub climb: LadderClimb,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub enum GameMode {
    #[default]
    NewLadder,
    RandomLadder {
        ladder_id: u64,
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
    pub fn get_ladder_id(&self) -> Option<u64> {
        match self.mode {
            GameMode::NewLadder => None,
            GameMode::RandomLadder { ladder_id } => Some(ladder_id),
        }
    }
    pub fn add_ladder_levels(&mut self, levels: Vec<String>) -> &mut Self {
        debug!("New ladder levels: {levels:#?}");
        self.climb.levels.extend(levels.clone());
        self
    }
    pub fn finish_building_ladder(&mut self) -> &mut Self {
        if matches!(self.mode, GameMode::NewLadder) && LoginPlugin::is_connected() {
            let team = ron::to_string(&self.climb.team).unwrap();
            debug!("Finish building ladder {team}");
            finish_building_ladder(self.climb.levels[..self.climb.defeated].to_vec(), team);
        }
        self
    }
    pub fn ladder(&self) -> Option<TableLadder> {
        let ladder_id = self.get_ladder_id()?;
        TableLadder::filter_by_id(ladder_id)
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
