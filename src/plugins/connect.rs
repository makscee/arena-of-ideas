use spacetimedb_lib::{de::Deserialize, ser::Serialize, Identity};
use spacetimedb_sdk::credentials;

use super::*;

pub struct ConnectPlugin;

impl Plugin for ConnectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Connect), Self::run_connect);
    }
}

static CONNECTION: OnceCell<DbConnection> = OnceCell::new();

pub fn cn() -> &'static DbConnection {
    CONNECTION.get().unwrap()
}
pub fn is_connected() -> bool {
    CONNECTION.get().is_some()
}

static ON_CONNECT_OPERATIONS: OnceCell<Mutex<VecDeque<Operation>>> = OnceCell::new();
pub fn on_connect(operation: impl FnOnce(&mut World) + Send + Sync + 'static) {
    if is_connected() {
        OperationsPlugin::add(operation);
    } else {
        ON_CONNECT_OPERATIONS
            .get_or_init(|| Mutex::new(default()))
            .lock()
            .push_back(Box::new(operation));
    }
}

#[derive(Serialize, Deserialize)]
struct Credentials {
    identity: Identity,
    token: String,
}

fn creds_store() -> credentials::File {
    credentials::File::new("aoi")
}

impl ConnectPlugin {
    fn run_connect() {
        info!("Connect start");
        Self::connect(|_, identity, token| {
            info!("Connected {identity}");
            let token = token.to_owned();
            save_player_identity(identity);
            creds_store().save(token.clone()).unwrap();
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
                .cstr_cs(tokens_global().high_contrast_text(), CstrStyle::Heading)
                .label(ui);
        });
    }
    pub fn connect(on_connect: fn(&DbConnection, Identity, &str)) {
        let (uri, module) = current_server();
        info!("Connect start {} {}", uri, module);
        let c = DbConnection::builder()
            .with_token(creds_store().load().expect("Error loading credentials"))
            .with_uri(uri)
            .with_module_name(module)
            .on_connect(on_connect)
            .build()
            .unwrap();
        c.run_threaded();
        CONNECTION.set(c).ok().unwrap();
        for op in ON_CONNECT_OPERATIONS.get().unwrap().lock().drain(..) {
            OperationsPlugin::add(op);
        }
    }
}
