use std::thread::sleep;

use bevy_tasks::{IoTaskPool, Task};
use spacetimedb_sdk::identity::{load_credentials, Credentials};

use super::*;

pub const HOME_DIR: &str = ".aoi";
const URI: &str = "http://161.35.88.206:3000";
const MODULE: &str = "aoi_dev";

pub struct LoginPlugin;

impl Plugin for LoginPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Connect), Self::connect)
            .add_systems(OnEnter(GameState::Login), Self::login)
            .init_resource::<ConnectionData>();
    }
}

#[derive(Resource, Default)]
struct ConnectionData {
    connect_task: Option<Task<()>>,
    state: ConnectionState,
}

#[derive(Default)]
enum ConnectionState {
    #[default]
    Disconnected,
    Connected,
    LoggedIn,
}

impl LoginPlugin {
    fn load_credentials() -> Option<Credentials> {
        load_credentials(HOME_DIR).expect("Failed to load credentials")
    }
    fn connect(world: &mut World) {
        GameState::set_ready(false);
        let mut cd = world.resource_mut::<ConnectionData>();
        if !matches!(cd.state, ConnectionState::Disconnected) {
            info!("Already connected");
            GameStatePlugin::path_proceed(world);
            return;
        }
        let thread_pool = IoTaskPool::get();
        info!("Connect start");
        let task = thread_pool.spawn(async {
            info!("Connect task start");
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
        });
        cd.connect_task = Some(task);
    }
    fn login(world: &mut World) {
        GameState::set_ready(false);
        let mut cd = world.resource_mut::<ConnectionData>();
        match cd.state {
            ConnectionState::Disconnected => panic!("Disconnected"),
            ConnectionState::LoggedIn => {
                info!("Already logged in");
                GameStatePlugin::path_proceed(world);
                return;
            }
            ConnectionState::Connected => {
                debug!("Do login..");
                cd.state = ConnectionState::LoggedIn;
                GameStatePlugin::path_proceed(world);
            }
        }
    }
    pub fn connect_ui(ui: &mut Ui, world: &mut World) {
        Window::new("State")
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .title_bar(false)
            .auto_sized()
            .show(ui.ctx(), |ui| {
                "Connecting...".cstr_cs(WHITE, CstrStyle::Heading).label(ui);
            });
        let mut cd = world.resource_mut::<ConnectionData>();
        if let Some(task) = &cd.connect_task {
            if task.is_finished() {
                cd.state = ConnectionState::Connected;
                cd.connect_task = None;
                GameStatePlugin::path_proceed(world);
            }
        }
    }
}
