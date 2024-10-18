use std::thread::sleep;

use spacetimedb_sdk::{
    identity::{load_credentials, once_on_connect, save_credentials},
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
        info!("Connect start");
        once_on_connect(|creds, _| {
            let creds = creds.clone();
            info!("Connected {}", hex::encode(creds.identity.as_bytes()));
            StdbQuery::Connect.subscribe();
            save_credentials(HOME_DIR, &creds).expect("Failed to save credentials");
            once_on_subscription_applied(|| {
                let server_version = GlobalData::current().game_version;
                if server_version == VERSION {
                    OperationsPlugin::add(|world| {
                        ConnectOption { creds }.save(world);
                        GameState::proceed(world);
                    });
                } else {
                    OperationsPlugin::add(move |w| {
                        let ctx = &egui_context(w).unwrap();
                        Confirmation::new(
                            "Wrong game version: "
                                .cstr_c(VISIBLE_LIGHT)
                                .push(
                                    format!("{} != {}", VERSION, server_version)
                                        .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold),
                                )
                                .take(),
                        )
                        .accept(|w| {
                            egui_context(w).unwrap().open_url(egui::OpenUrl {
                                url: "https://github.com/makscee/arena-of-ideas/releases"
                                    .to_owned(),
                                new_tab: true,
                            });
                            app_exit(w);
                        })
                        .cancel(|w| app_exit(w))
                        .accept_name("Update")
                        .cancel_name("Exit")
                        .push(ctx);
                    });
                }
            });
        });
        let thread_pool = IoTaskPool::get();
        thread_pool
            .spawn(async {
                let creds: Option<Credentials> = Self::load_credentials();
                let mut tries = 3;
                let server = current_server();
                info!("Connect start {} {}", server.0, server.1);
                while let Err(e) = connect(server.0, server.1, creds.clone()) {
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
