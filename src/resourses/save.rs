use crate::module_bindings::add_ladder_levels;

use super::*;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Save {
    pub team: PackedTeam,
    pub mode: GameMode,
    pub current_level: usize,
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
    pub fn get_ladder_id(&self) -> Result<u64> {
        match self.mode {
            GameMode::NewLadder => {
                let identity = identity()?;
                Ok(TableLadder::find(|l| {
                    l.creator == identity && l.status.eq(&module_bindings::LadderStatus::Building)
                })
                .context("Failed to find ladder")?
                .id)
            }
            GameMode::RandomLadder { ladder_id } => Ok(ladder_id),
        }
    }
    pub fn add_ladder_levels(&mut self, levels: &Vec<PackedTeam>) -> &mut Self {
        debug!("New ladder levels: {levels:#?}");
        let levels = levels.iter().map(|l| l.to_ladder_string()).collect_vec();
        add_ladder_levels(self.get_ladder_id().unwrap(), levels);
        self
    }
    pub fn ladder(&self) -> Option<TableLadder> {
        let ladder_id = self.get_ladder_id().ok()?;
        TableLadder::filter_by_id(ladder_id)
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
            .map_err(|e| anyhow!(e.to_string()))
    }
}
