use spacetimedb_lib::{bsatn, de::Deserialize, ser::Serialize, Identity};

use super::*;

pub struct ConnectPlugin;

impl Plugin for ConnectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Connect), Self::run_connect);
    }
}

static CONNECTION: OnceCell<DbConnection> = OnceCell::new();
const CREDENTIALS_FILE: &str = "credentials";

pub fn cn() -> &'static DbConnection {
    CONNECTION.get().unwrap()
}

#[derive(Serialize, Deserialize)]
struct Credentials {
    identity: Identity,
    token: String,
}

impl ConnectPlugin {
    fn load_credentials() -> Result<Option<(Identity, String)>> {
        let mut path = home_dir_path();
        path.push(CREDENTIALS_FILE);
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) if matches!(e.kind(), std::io::ErrorKind::NotFound) => return Ok(None),
            Err(e) => {
                return Err(e).with_context(|| {
                    format!("Error reading BSATN-serialized credentials from file {path:?}")
                })
            }
        };
        let creds = bsatn::from_slice::<Credentials>(&bytes).context(format!(
            "Error deserializing credentials from bytes stored in file {path:?}",
        ))?;
        Ok(Some((creds.identity, creds.token)))
    }
    fn save_credentials(identity: Identity, token: String) -> Result<()> {
        let mut path = home_dir_path();
        path.push(CREDENTIALS_FILE);
        let creds = bsatn::to_vec(&Credentials {
            identity,
            token: token.to_string(),
        })
        .context("Error serializing credentials for storage in file")?;
        std::fs::write(&path, creds).with_context(|| {
            format!("Error writing BSATN-serialized credentials to file {path:?}")
        })?;
        Ok(())
    }
    fn run_connect() {
        info!("Connect start");
        Self::connect(|_, identity, token| {
            info!("Connected {identity}");
            let token = token.to_owned();
            save_identity(identity);
            StdbQuery::subscribe(StdbQuery::queries_login(), move |world| {
                info!("On subscribe");
                let server_version = cn().db.global_data().current().game_version;
                Self::save_credentials(identity, token.clone())
                    .expect("Failed to save credentials");
                if server_version == VERSION {
                    ConnectOption { identity, token }.save(world);
                    GameState::proceed(world);
                } else {
                    Confirmation::new(
                        "Wrong game version: "
                            .cstr_c(VISIBLE_LIGHT)
                            .push(
                                format!("{} != {}", VERSION, server_version)
                                    .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold),
                            )
                            .take(),
                    )
                    .accept(|world| {
                        egui_context(world).unwrap().open_url(egui::OpenUrl {
                            url: "https://github.com/makscee/arena-of-ideas/releases".to_owned(),
                            new_tab: true,
                        });
                        app_exit(world);
                    })
                    .cancel(|world| app_exit(world))
                    .accept_name("Update")
                    .cancel_name("Exit")
                    .push(world);
                }
            });
        });
    }
    pub fn ui(ui: &mut Ui) {
        center_window("status", ui, |ui| {
            "Connecting..."
                .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading)
                .label(ui);
        });
    }
    pub fn connect(on_connect: fn(&DbConnection, Identity, &str)) {
        let (uri, module) = current_server();
        info!("Connect start {} {}", uri, module);
        let credentials = Self::load_credentials().unwrap_or_default();
        let c = DbConnection::builder()
            .with_credentials(credentials)
            .with_uri(uri)
            .with_module_name(module)
            .on_connect(on_connect)
            .build()
            .unwrap();
        c.run_threaded();
        apply_subscriptions(&c);
        CONNECTION.set(c).ok().unwrap();
    }
}
