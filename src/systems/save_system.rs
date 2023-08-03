use std::fs::{create_dir_all, remove_dir_all, write, File};

use geng::prelude::file::load_json;

use super::*;

pub struct SaveSystem {}

fn dir_path() -> PathBuf {
    run_dir().join("save")
}
fn file_path() -> PathBuf {
    dir_path().join("save.json")
}
impl SaveSystem {
    pub fn have_saved_game() -> bool {
        file_path().exists()
    }

    pub fn save_game(world: &legion::World, resources: &Resources) {
        debug!("Saving game...");
        let team = PackedTeam::pack(Faction::Team, world, resources);
        let mut data = match Self::load_data() {
            Ok(data) => data,
            Err(_) => default(),
        };
        data.team = Some(team);
        data.level = Ladder::current_ind(resources);
        Self::save_data(&data);
    }

    pub fn load_game(world: &mut legion::World, resources: &mut Resources) {
        debug!("Loading game from {:?}", file_path());
        match Self::load_data() {
            Ok(save) => {
                if save.team.is_none() {
                    panic!("No saved game detected")
                }
                Game::restart(world, resources);
                save.team.unwrap().unpack(Faction::Team, world, resources);
                Ladder::set_level(save.level, resources);
                GameStateSystem::set_transition(GameState::Shop, resources);
                resources.shop_data.loaded = true;
            }
            Err(error) => {
                error!("Can't load game: {}", error)
            }
        }
    }

    pub fn save_ladder(resources: &Resources) {
        debug!("Saving ladder...");
        let teams = Ladder::get_levels(resources);
        let mut data = match Self::load_data() {
            Ok(data) => data,
            Err(_) => default(),
        };
        data.ladder = teams;
        Self::save_data(&data);
    }

    pub fn load_ladder(resources: &mut Resources) {
        debug!("Loading ladder from {:?}", file_path());
        match Self::load_data() {
            Ok(save) => {
                Ladder::set_levels(save.ladder, resources);
            }
            Err(error) => {
                error!("Can't load ladder: {}", error)
            }
        }
    }

    pub fn load_data() -> Result<SaveData> {
        futures::executor::block_on(load_json::<SaveData>(file_path()))
    }

    pub fn save_data(data: &SaveData) {
        let save = serde_json::to_string_pretty(data).unwrap();
        let path = file_path();
        if !path.exists() {
            create_dir_all(dir_path()).expect("Failed to create dir");
            File::create(path.clone()).expect("Failed to create new save file");
        }
        match write(path.clone(), save) {
            Ok(_) => debug!("Saved to {:?}", path.clone()),
            Err(error) => error!("Can't save: {}", error),
        }
    }

    pub fn clear_save() {
        if file_path().exists() {
            remove_dir_all(dir_path()).expect("Failed to clear save dir");
        }
    }
}
