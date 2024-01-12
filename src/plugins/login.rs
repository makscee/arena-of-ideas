use std::{sync::Mutex, thread::sleep};

use spacetimedb_sdk::{
    identity::{load_credentials, save_credentials},
    reducer::Status,
    subscribe,
};

use crate::module_bindings::{
    login, login_by_identity, once_on_login, once_on_login_by_identity, once_on_register,
    once_on_register_empty, register, register_empty, GlobalData, User,
};

use super::*;

pub struct LoginPlugin;

// const SPACETIMEDB_URI: &str = "http://localhost:3001";
const SPACETIMEDB_URI: &str = "http://178.62.220.183:3000";
#[cfg(debug_assertions)]
const DB_NAME: &str = "aoi_dev";
#[cfg(not(debug_assertions))]
const DB_NAME: &str = "aoi";
const CREDS_DIR: &str = ".aoi";

static IS_CONNECTED: Mutex<bool> = Mutex::new(false);
static LOGIN_EVENT_DISPATCHED: Mutex<bool> = Mutex::new(false);
pub static CURRENT_USER: Mutex<Option<String>> = Mutex::new(None);

fn on_connected(creds: &Credentials, _client_address: Address) {
    *IS_CONNECTED.lock().unwrap() = true;
    debug!("Current identity: {}", hex::encode(creds.identity.bytes()));
    if let Err(e) = save_credentials(CREDS_DIR, creds) {
        eprintln!("Failed to save credentials: {:?}", e);
    }
}
fn register_callbacks() {
    once_on_connect(on_connected);
}
// fn identity_leading_hex(id: &Identity) -> String {
//     hex::encode(&id.bytes()[0..8])
// }

#[derive(Resource, Default)]
pub struct LoginData {
    pub name: String,
    pub pass: String,
    pub login_sent: bool,
}
#[derive(Resource, Default)]
pub struct RegisterData {
    pub name: String,
    pub pass: String,
    pub pass_repeat: String,
}

#[derive(BevyEvent)]
pub struct LoginEvent;

impl Plugin for LoginPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::setup)
            .add_systems(Update, Self::update.run_if(in_state(GameState::MainMenu)))
            .init_resource::<LoginData>()
            .init_resource::<RegisterData>()
            .add_event::<LoginEvent>();
    }
}

impl LoginPlugin {
    fn load_credentials() -> Option<Credentials> {
        load_credentials(CREDS_DIR).expect("Failed to load credentials")
    }

    fn update(world: &mut World) {
        if *IS_CONNECTED.lock().unwrap() {
            let mut data = world.resource_mut::<LoginData>();
            if !data.login_sent {
                let creds = Self::load_credentials().unwrap();
                data.login_sent = true;
                if User::find(|u| u.identities.contains(&creds.identity)).is_some() {
                    Self::login_by_identity();
                } else {
                    register_empty();
                    once_on_register_empty(|_, _, status| {
                        debug!("Register empty: {status:?}");
                        match status {
                            Status::Committed => Self::login_by_identity(),
                            Status::Failed(e) => AlertPlugin::add_error(
                                Some("REGISTER ERROR".to_owned()),
                                e.to_owned(),
                                None,
                            ),
                            _ => panic!(),
                        }
                    });
                }
            }
            if Self::get_username().is_some() {
                let mut led = LOGIN_EVENT_DISPATCHED.lock().unwrap();
                if !*led && !VERSION.eq(&GlobalData::filter_by_always_zero(0).unwrap().game_version)
                {
                    AlertPlugin::add_error(
                        Some("GAME VERSION ERROR".to_owned()),
                        "Game version is too old".to_owned(),
                        Some(Box::new(|w| {
                            egui_context(w).open_url(egui::OpenUrl {
                                url: "https://makscee.itch.io/arena-of-ideas".to_owned(),
                                new_tab: true,
                            });
                            w.send_event(AppExit);
                        })),
                    );
                    *led = true;
                    *CURRENT_USER.lock().unwrap() = None;
                }
                if !*led {
                    world.send_event(LoginEvent);
                    *led = true;
                }
            }
        }
    }

    pub fn is_connected() -> bool {
        *IS_CONNECTED.lock().unwrap()
    }
    pub fn get_username() -> Option<String> {
        CURRENT_USER.lock().unwrap().clone()
    }

    fn setup() {
        register_callbacks();
        Self::connect();
    }

    pub fn connect() {
        if Self::is_connected() {
            return;
        }
        let creds = Self::load_credentials();
        let mut tries = 5;
        while let Err(e) = connect(SPACETIMEDB_URI, DB_NAME, creds.clone()) {
            error!("Connection error: {e}");
            sleep(Duration::from_secs(1));
            tries -= 1;
            if tries <= 0 {
                return;
            }
        }
        subscribe_to_tables();
    }

    pub fn clear_saved_credentials() {
        let mut path = home::home_dir().expect("Failed to get home dir");
        path.push(CREDS_DIR);
        std::fs::remove_dir_all(path).expect("Failed to clear credentials dir");
    }

    pub fn save_current_user(name: String) {
        *CURRENT_USER.lock().unwrap() = Some(name)
    }

    fn on_login(status: &Status, name: &String) {
        debug!("Login: {status:?} {name}");
        match status {
            Status::Committed => Self::save_current_user(name.clone()),
            Status::Failed(e) => {
                AlertPlugin::add_error(
                    Some("LOGIN ERROR".to_owned()),
                    format!("Failed to login {e}"),
                    None,
                );
            }
            _ => panic!(),
        }
    }

    fn send_login(name: String, pass: String) {
        login(name, pass);
        once_on_login(|_, _, status, name, _| {
            Self::on_login(status, name);
        });
    }

    fn login_by_identity() {
        login_by_identity();
        once_on_login_by_identity(|identity, _, status| {
            let name = User::find(|u| u.identities.contains(identity))
                .unwrap()
                .name;
            Self::on_login(status, &name);
        });
    }

    pub fn login(ui: &mut Ui, world: &mut World) {
        let mut login_data = world.resource_mut::<LoginData>();

        frame(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("name:");
                    ui.label("password:");
                });
                ui.vertical(|ui| {
                    TextEdit::singleline(&mut login_data.name)
                        .desired_width(ui.available_width())
                        .margin(egui::Vec2::ZERO)
                        .ui(ui);
                    TextEdit::singleline(&mut login_data.pass)
                        .password(true)
                        .desired_width(ui.available_width())
                        .margin(egui::Vec2::ZERO)
                        .ui(ui);
                });
            });
            ui.set_enabled(!login_data.name.is_empty() && !login_data.pass.is_empty());
            if ui.button("LOGIN").clicked() {
                Self::send_login(login_data.name.clone(), login_data.pass.clone());
            }
        });
    }

    pub fn register(ui: &mut Ui, world: &mut World) {
        frame(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical_centered_justified(|ui| {
                let mut register_data = world.resource_mut::<RegisterData>();
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("name:");
                        ui.label("password:");
                        ui.label("repeat:");
                    });
                    ui.vertical(|ui| {
                        TextEdit::singleline(&mut register_data.name)
                            .desired_width(ui.available_width())
                            .margin(egui::Vec2::ZERO)
                            .ui(ui);
                        TextEdit::singleline(&mut register_data.pass)
                            .password(true)
                            .desired_width(ui.available_width())
                            .margin(egui::Vec2::ZERO)
                            .ui(ui);
                        TextEdit::singleline(&mut register_data.pass_repeat)
                            .password(true)
                            .desired_width(ui.available_width())
                            .margin(egui::Vec2::ZERO)
                            .ui(ui);
                    });
                });
                ui.set_enabled(
                    !register_data.name.is_empty()
                        && !register_data.pass.is_empty()
                        && register_data.pass.eq(&register_data.pass_repeat),
                );
                if ui.button("REGISTER").clicked() {
                    debug!(
                        "Register start: {} {}",
                        register_data.name, register_data.pass
                    );
                    register(register_data.name.clone(), register_data.pass.clone());
                    once_on_register(|_, _, status, name, pass| {
                        debug!("Register: {status:?} {name}");
                        match status {
                            Status::Committed => Self::send_login(name.to_owned(), pass.to_owned()),
                            Status::Failed(e) => AlertPlugin::add_error(
                                Some("REGISTER ERROR".to_owned()),
                                e.to_owned(),
                                None,
                            ),
                            _ => panic!(),
                        }
                    });
                    set_context_bool(world, "register", false);
                }
            })
        });
    }
}

fn subscribe_to_tables() {
    subscribe(&[
        "SELECT * FROM User",
        "SELECT * FROM Unit",
        "SELECT * FROM House",
        "SELECT * FROM Statuses",
        "SELECT * FROM Ability",
        "SELECT * FROM Vfx",
        "SELECT * FROM GlobalTower",
        "SELECT * FROM GlobalData",
    ])
    .unwrap();
}
