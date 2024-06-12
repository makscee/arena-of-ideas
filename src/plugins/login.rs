use std::thread::sleep;

use spacetimedb_sdk::identity::{load_credentials, Credentials};

use super::*;

pub const HOME_DIR: &str = ".aoi";
const URI: &str = "http://161.35.88.206:3000";
const MODULE: &str = "aoi_dev";

pub struct LoginPlugin;

impl Plugin for LoginPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Connect), Self::connect);
    }
}

impl LoginPlugin {
    fn load_credentials() -> Option<Credentials> {
        load_credentials(HOME_DIR).expect("Failed to load credentials")
    }

    fn connect(world: &World) {
        info!("Connect start");
        let creds: Option<Credentials> = Self::load_credentials();
        let mut tries = 5;
        while let Err(e) = connect(URI, MODULE, creds.clone()) {
            error!("Connection error: {e}");
            sleep(Duration::from_secs(1));
            tries -= 1;
            if tries <= 0 {
                return;
            }
        }
        info!("Connected");
    }
}
