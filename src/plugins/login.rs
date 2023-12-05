use spacetimedb_sdk::{
    identity::{credentials, load_credentials, save_credentials},
    subscribe,
};

use super::*;

pub struct LoginPlugin;

const SPACETIMEDB_URI: &str = "http://localhost:3001";
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
            .add_systems(
                Update,
                (Self::ui, Self::update).run_if(in_state(GameState::Login)),
            )
            .init_resource::<CurrentCredentials>();
    }
}

impl LoginPlugin {
    fn setup(world: &mut World) {
        register_callbacks();
        if let Some(creds) = load_credentials(CREDS_DIR).expect("Failed to load credentials") {
            world.insert_resource(CurrentCredentials { creds: Some(creds) });
        }
    }

    fn update(world: &mut World) {
        if let Ok(creds) = credentials() {
            let cur_creds = &mut world.resource_mut::<CurrentCredentials>().creds;
            if cur_creds.is_none() {
                save_credentials(CREDS_DIR, &creds).expect("Failed to save credentials");
                debug!("New identity: {}", identity_leading_hex(&creds.identity));
                *cur_creds = Some(creds);
            }
            GameState::MainMenu.change(world);
        }
    }

    fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        MainMenuPlugin::menu_window("Login", ctx, |ui| {
            if identity().is_err() {
                if ui.add(MainMenuPlugin::menu_button("Connect")).clicked() {
                    let creds = world.resource::<CurrentCredentials>().creds.clone();
                    connect(SPACETIMEDB_URI, DB_NAME, creds).expect("Failed to connect");
                    subscribe_to_tables();
                }
            }
        });
    }
}

fn subscribe_to_tables() {
    subscribe(&["SELECT * FROM User;SELECT * FROM Ladder;"]).unwrap();
}