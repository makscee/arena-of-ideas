use super::*;

pub struct LoginPlugin;

const SPACETIMEDB_URI: &str = "http://localhost:3001";
const DB_NAME: &str = "aoi";

fn on_connected(creds: &Credentials, _client_address: Address) {
    println!("{creds:?} {_client_address:?}");
}
fn register_callbacks() {
    once_on_connect(on_connected);
}

#[derive(Resource)]
pub struct CurrentIdentity {
    pub identity: Option<Identity>,
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
        if let Ok(identity) = identity() {
            if !world
                .get_resource::<CurrentIdentity>()
                .is_some_and(|i| i.identity.is_some())
            {
                world.insert_resource(CurrentIdentity {
                    identity: Some(identity),
                });
                GameState::MainMenu.change(world);
            }
        }
    }

    fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        MainMenuPlugin::menu_window("Login", ctx, |ui| {
            if identity().is_err() {
                if ui.add(MainMenuPlugin::menu_button("Connect")).clicked() {
                    connect(SPACETIMEDB_URI, DB_NAME, None).expect("Failed to connect");
                }
            }
        });
    }
}
