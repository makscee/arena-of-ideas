use std::thread::sleep;

use spacetimedb_sdk::{
    identity::{load_credentials, once_on_connect},
    once_on_subscription_applied,
};

use super::*;

pub struct ConnectPlugin;

impl Plugin for ConnectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Connect), Self::connect);
    }
}

impl ConnectPlugin {
    fn load_credentials() -> Option<Credentials> {
        load_credentials(HOME_DIR).expect("Failed to load credentials")
    }
    fn connect() {
        let thread_pool = IoTaskPool::get();
        info!("Connect start");
        once_on_connect(|creds, _| {
            let creds = creds.clone();
            info!("Connected {}", hex::encode(creds.identity.bytes()));
            StdbQuery::Connect.subscribe();
            once_on_subscription_applied(|| {
                OperationsPlugin::add(|world| {
                    ConnectOption { creds }.save(world);
                    GameState::proceed(world);
                });
            });
        });
        thread_pool
            .spawn(async {
                let creds: Option<Credentials> = Self::load_credentials();
                let mut tries = 5;
                let server = if cfg!(debug_assertions) {
                    client_settings().dev_server.clone()
                } else {
                    client_settings().prod_server.clone()
                };
                info!("Connect start {} {}", server.0, server.1);
                while let Err(e) = connect(&server.0, &server.1, creds.clone()) {
                    error!("Connection error: {e}");
                    sleep(Duration::from_secs(1));
                    tries -= 1;
                    if tries <= 0 {
                        return;
                    }
                }
            })
            .detach();
    }
    pub fn ui(ui: &mut Ui) {
        center_window("status", ui, |ui| {
            "Connecting..."
                .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading)
                .label(ui);
        });
    }
}
