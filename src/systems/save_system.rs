use std::fs::write;

use geng::prelude::file::load_json;

use super::*;

pub struct SaveSystem {}

fn path() -> PathBuf {
    save_path().join("save.json")
}
impl SaveSystem {
    pub fn have_saved_game() -> bool {
        path().exists()
    }

    pub fn save(world: &legion::World, resources: &Resources) {
        debug!("Saving...");
        let team = PackedTeam::pack(Faction::Team, world, resources);
        let save = SaveData {
            team,
            level: Ladder::current_ind(resources),
        };
        let save = serde_json::to_string_pretty(&save).unwrap();
        match write(path(), save) {
            Ok(_) => debug!("Saved to {:?}", path()),
            Err(error) => error!("Can't save: {}", error),
        }
    }

    pub fn load(world: &mut legion::World, resources: &mut Resources) {
        debug!("Loading save from {:?}", path());
        match futures::executor::block_on(load_json::<SaveData>(path())) {
            Ok(save) => {
                Game::restart(world, resources);
                save.team.unpack(Faction::Team, world, resources);
                Ladder::set_level(save.level, resources);
                GameStateSystem::set_transition(GameState::Shop, resources);
                resources.shop_data.loaded = true;
            }
            Err(error) => {
                error!("Can't load save: {}", error)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SaveData {
    pub team: PackedTeam,
    pub level: usize,
}
