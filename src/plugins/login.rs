use spacetimedb_sdk::identity::{credentials, load_credentials, save_credentials};

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

#[derive(Resource)]
pub struct CurrentIdentity {
    pub creds: Option<Credentials>,
}

impl Plugin for LoginPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::setup).add_systems(
            Update,
            (Self::ui, Self::update).run_if(in_state(GameState::Login)),
        );
    }
}

impl LoginPlugin {
    fn setup(_: &mut World) {
        register_callbacks();
    }

    fn update(world: &mut World) {
        if let Ok(creds) = credentials() {
            if !world
                .get_resource::<CurrentIdentity>()
                .is_some_and(|i| i.creds.is_some())
            {
                save_credentials(CREDS_DIR, &creds).expect("Failed to save credentials");
                debug!(
                    "Connected with identity: {}",
                    identity_leading_hex(&creds.identity)
                );
                world.insert_resource(CurrentIdentity { creds: Some(creds) });
                GameState::MainMenu.change(world);
            }
        }
    }

    fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        MainMenuPlugin::menu_window("Login", ctx, |ui| {
            if identity().is_err() {
                if ui.add(MainMenuPlugin::menu_button("Connect")).clicked() {
                    let creds = load_credentials(CREDS_DIR).expect("Failed to load credentials");
                    connect(SPACETIMEDB_URI, DB_NAME, creds).expect("Failed to connect");
                }
            }
        });
    }
}
