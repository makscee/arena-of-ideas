use spacetimedb_sdk::{
    identity::{load_credentials, save_credentials},
    subscribe,
};

use super::*;

pub struct LoginPlugin;

// const SPACETIMEDB_URI: &str = "http://localhost:3001";
const SPACETIMEDB_URI: &str = "http://178.62.220.183:3000";
const DB_NAME: &str = "aoi";
const CREDS_DIR: &str = ".aoi";

fn on_connected(creds: &Credentials, _client_address: Address) {
    println!("{creds:?} {_client_address:?}");
}
fn register_callbacks() {
    once_on_connect(on_connected);
}
fn identity_leading_hex(id: &Identity) -> String {
    hex::encode(&id.bytes()[0..8])
}

#[derive(Resource, Default)]
pub struct CurrentCredentials {
    pub creds: Option<Credentials>,
}

impl Plugin for LoginPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::setup)
            .init_resource::<CurrentCredentials>();
    }
}

impl LoginPlugin {
    pub fn is_connected() -> bool {
        identity().is_ok()
    }

    fn setup(world: &mut World) {
        register_callbacks();
        if let Some(creds) = load_credentials(CREDS_DIR).expect("Failed to load credentials") {
            world.insert_resource(CurrentCredentials { creds: Some(creds) });
        }
    }

    pub fn connect(world: &mut World) {
        let creds = &mut world.resource_mut::<CurrentCredentials>().creds;
        while let Err(e) = connect(SPACETIMEDB_URI, DB_NAME, creds.clone()) {
            error!("Connection error: {e}");
        }
        subscribe_to_tables();
    }

    pub fn save_credentials(creds: Credentials, world: &mut World) {
        save_credentials(CREDS_DIR, &creds).expect("Failed to save credentials");
        debug!(
            "Connected with identity {}",
            identity_leading_hex(&creds.identity)
        );
        world.resource_mut::<CurrentCredentials>().creds = Some(creds);
    }

    pub fn clear_saved_credentials(world: &mut World) {
        let mut path = home::home_dir().expect("Failed to get home dir");
        path.push(CREDS_DIR);
        std::fs::remove_dir_all(path).expect("Failed to clear credentials dir");
        world.resource_mut::<CurrentCredentials>().creds = None;
    }
}

fn subscribe_to_tables() {
    subscribe(&["SELECT * FROM User; SELECT * FROM Tower;"]).unwrap();
}
