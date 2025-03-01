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
pub fn is_connected() -> bool {
    CONNECTION.get().is_some()
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
        let creds = bsatn::to_vec(&Credentials { identity, token })
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
            save_player_identity(identity);
            Self::save_credentials(identity.clone(), token.clone())
                .expect("Failed to save credentials");
            OperationsPlugin::add(move |world| {
                ConnectOption { identity, token }.save(world);
                GameState::proceed(world);
            });
        });
    }
    pub fn tab(ui: &mut Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.add_space(ui.available_height() * 0.5 - 25.0);
            format!("Connecting{}", (0..(gt().secs() % 4)).map(|_| ".").join(""))
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
        CONNECTION.set(c).ok().unwrap();
    }
}
