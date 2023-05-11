use std::fs::write;

use geng::prelude::file::load_json;

use super::*;

pub struct SaveSystem {}

fn path() -> PathBuf {
    save_path().join("save.json")
}
impl SaveSystem {
    pub fn save(world: &legion::World, resources: &Resources) {
        debug!("Saving...");
        let team = PackedTeam::pack(&Faction::Team, world, resources);
        let save = SaveData {
            team,
            floor: resources.ladder.current_ind(),
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
                Game::reset(world, resources);
                save.team.unpack(&Faction::Team, world, resources);
                resources.ladder.set(save.floor);
                for level in 0..save.floor {
                    ShopData::load_floor(resources, level);
                }
                ShopSystem::enter(world, resources);
                debug!("Loaded {}", save.team);
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
    pub floor: usize,
}
