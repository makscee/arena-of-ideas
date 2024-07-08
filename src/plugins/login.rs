use std::thread::sleep;

use bevy_tasks::IoTaskPool;
use spacetimedb_sdk::{
    identity::{load_credentials, once_on_connect, Credentials},
    once_on_subscription_applied,
    table::TableType,
};

use super::*;

pub const HOME_DIR: &str = ".aoi";
const URI: &str = "http://161.35.88.206:3000";
const MODULE: &str = "aoi_dev";

static USER_NAME: Mutex<&'static str> = Mutex::new("");
static USER_ID: Mutex<u64> = Mutex::new(0);

pub fn user_id() -> u64 {
    *USER_ID.lock().unwrap()
}
pub fn user_name() -> &'static str {
    *USER_NAME.lock().unwrap()
}

pub struct LoginPlugin;

impl Plugin for LoginPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Connect), Self::connect)
            .add_systems(OnEnter(GameState::Login), Self::login)
            .add_systems(OnEnter(GameState::ForceLogin), Self::login)
            .init_resource::<ConnectionData>()
            .init_resource::<LoginData>();
    }
}

#[derive(Resource, Default)]
struct ConnectionData {
    state: ConnectionState,
    creds: Option<Credentials>,
    identity_user: Option<User>,
    user: Option<User>,
}

#[derive(Resource, Default)]
struct LoginData {
    name: String,
    pass: String,
}

#[derive(Default)]
enum ConnectionState {
    #[default]
    Disconnected,
    Connected,
    LoggedIn,
}

impl LoginPlugin {
    pub fn user(world: &World) -> &User {
        world.resource::<ConnectionData>().user.as_ref().unwrap()
    }
    pub fn save_user(user: User, world: &mut World) {
        let mut cd = world.resource_mut::<ConnectionData>();
        cd.identity_user = Some(user.clone());
        cd.user = Some(user.clone());
        *USER_ID.lock().unwrap() = user.id;
        *USER_NAME.lock().unwrap() = user.name.leak();
    }
    fn load_credentials() -> Option<Credentials> {
        load_credentials(HOME_DIR).expect("Failed to load credentials")
    }
    fn connect(world: &mut World) {
        GameState::set_ready(false);
        let cd = world.resource_mut::<ConnectionData>();
        if !matches!(cd.state, ConnectionState::Disconnected) {
            info!("Already connected");
            GameStatePlugin::path_proceed(world);
            return;
        }
        let thread_pool = IoTaskPool::get();
        info!("Connect start");
        once_on_connect(|creds, _| {
            let creds = creds.clone();
            ServerPlugin::subscribe_connect();
            once_on_subscription_applied(|| {
                OperationsPlugin::add(|world| {
                    let mut cd = world.resource_mut::<ConnectionData>();
                    cd.creds = Some(creds);
                    cd.state = ConnectionState::Connected;
                    GameStatePlugin::path_proceed(world);
                });
            });
        });
        thread_pool
            .spawn(async {
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
            })
            .detach();
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
                let identity = cd.creds.clone().unwrap().identity;
                if let Some(user) = User::find(|u| u.identities.contains(&identity)) {
                    cd.identity_user = Some(user);
                }
            }
        }
    }
    pub fn connect_ui(ui: &mut Ui, _: &mut World) {
        center_window("status", ui, |ui| {
            "Connecting...".cstr_cs(WHITE, CstrStyle::Heading).label(ui);
        });
    }
    pub fn login_ui(ui: &mut Ui, world: &mut World) {
        let state = cur_state(world);
        center_window("login", ui, |ui| {
            ui.vertical_centered_justified(|ui| {
                let mut cd = world.resource_mut::<ConnectionData>();
                if let Some(user) = cd.identity_user.clone() {
                    format!("Login as {}", user.name)
                        .cstr_cs(LIGHT_GRAY, CstrStyle::Heading2)
                        .label(ui);
                    if Button::click("Login".into()).ui(ui).clicked()
                        || matches!(state, GameState::ForceLogin)
                    {
                        login_by_identity();
                        once_on_login_by_identity(|_, _, status| {
                            debug!("login reducer {status:?}");
                            match status {
                                spacetimedb_sdk::reducer::Status::Committed => {
                                    OperationsPlugin::add(|world| {
                                        let mut cd = world.resource_mut::<ConnectionData>();
                                        cd.state = ConnectionState::LoggedIn;
                                        Self::save_user(cd.identity_user.clone().unwrap(), world);
                                        ServerPlugin::subscribe_game();
                                        once_on_subscription_applied(|| {
                                            OperationsPlugin::add(|world| {
                                                GameAssets::cache_tables(world);
                                                GameStatePlugin::path_proceed(world);
                                            });
                                        });
                                    })
                                }
                                spacetimedb_sdk::reducer::Status::Failed(e) => {
                                    Notification::new(format!("Login failed: {e}")).push_op()
                                }
                                _ => panic!(),
                            };
                        });
                    }
                    br(ui);
                    if Button::gray("Logout".into()).ui(ui).clicked() {
                        cd.identity_user = None;
                    }
                } else {
                    let mut ld = world.resource_mut::<LoginData>();
                    "Register".cstr_cs(LIGHT_GRAY, CstrStyle::Heading).label(ui);
                    if Button::click("New Player".into()).ui(ui).clicked() {
                        register_empty();
                        once_on_register_empty(|_, _, status| match status {
                            spacetimedb_sdk::reducer::Status::Committed => {
                                OperationsPlugin::add(|world| {
                                    Notification::new("New player created".to_owned()).push(world);
                                    let mut cd = world.resource_mut::<ConnectionData>();
                                    let user = User::find(|u| {
                                        u.identities.contains(&cd.creds.clone().unwrap().identity)
                                    })
                                    .expect("Failed to find user after registration");
                                    cd.identity_user = Some(user);
                                })
                            }
                            spacetimedb_sdk::reducer::Status::Failed(e) => {
                                Notification::new(format!("Registration failed: {e}"))
                                    .error()
                                    .push_op()
                            }
                            _ => panic!(),
                        });
                    }
                    br(ui);
                    "Login".cstr_cs(LIGHT_GRAY, CstrStyle::Heading).label(ui);
                    Input::new("name").ui(&mut ld.name, ui);
                    Input::new("password").password().ui(&mut ld.pass, ui);
                    if Button::click("Submit".into()).ui(ui).clicked() {
                        login(ld.name.clone(), ld.pass.clone());
                        once_on_login(|_, _, status, name, _| match status {
                            spacetimedb_sdk::reducer::Status::Committed => {
                                let name = name.clone();
                                OperationsPlugin::add(move |world| {
                                    let user = User::filter_by_name(name).unwrap();
                                    Self::save_user(user, world);
                                });
                            }
                            spacetimedb_sdk::reducer::Status::Failed(e) => {
                                let text = format!("Login failed: {e}");
                                error!("{text}");
                                Notification::new(text).error().push_op();
                            }
                            _ => panic!(),
                        });
                    }
                }
            });
        });
    }
}
