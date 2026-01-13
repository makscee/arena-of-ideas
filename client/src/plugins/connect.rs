use spacetimedb_lib::Identity;
use spacetimedb_sdk::credentials;

use super::*;

pub struct ConnectPlugin;

impl Plugin for ConnectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Connect),
            (Self::run_connect, store_token_data),
        );
        app.init_resource::<LoginData>();
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
        OperationsPlugin::add(true, operation);
    } else {
        ON_CONNECT_OPERATIONS
            .get_or_init(|| Mutex::new(default()))
            .lock()
            .push_back(Box::new(operation));
    }
}

pub(crate) fn creds_store() -> credentials::File {
    credentials::File::new("aoi_creds")
}

fn store_token_data(ao: Res<AuthOption>, mut ld: ResMut<LoginData>) {
    // TODO: need secure decode with verification for prod - https://docs.rs/jsonwebtoken/latest/jsonwebtoken/fn.decode.html
    // let token = ao.id_token.as_ref().unwrap();
    // let token_data: TokenData<Claims> = insecure_decode(token).expect("Failed to decode token");
    // ld.username = token_data.claims.preferred_username;
    ld.username = "izarma".to_string();
}

impl ConnectPlugin {
    fn run_connect(world: &mut World) {
        info!("Connect start");
        let id_token = world.get_resource::<AuthOption>().unwrap().id_token.clone();
        Self::connect(id_token, |_, identity, token| {
            info!("Connected {identity}");
            let token = token.to_owned();
            save_player_identity(identity);
            creds_store().save(token.clone()).unwrap();
            op(move |world| {
                ConnectOption { identity, token }.save(world);
                GameState::proceed(world);
            });
        });
        let _ = cn().reducers.login_by_identity();
    }
    pub fn pane(ui: &mut Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.add_space(ui.available_height() * 0.5 - 25.0);
            format!("Connecting{}", (0..(gt().secs() % 4)).map(|_| ".").join(""))
                .cstr_cs(high_contrast_text(), CstrStyle::Heading)
                .label(ui);
        });
    }
    pub fn connect(default_token: Option<String>, on_connect: fn(&DbConnection, Identity, &str)) {
        let (uri, module) = current_server();
        info!(
            "Connect start {} {} with token {:?}",
            uri, module, default_token
        );
        let c = DbConnection::builder()
            .with_token(default_token)
            .with_uri(uri)
            .with_module_name(module)
            .on_connect(on_connect)
            .build()
            .unwrap();
        c.run_threaded();
        CONNECTION.set(c).ok().unwrap();
        if let Some(ops) = ON_CONNECT_OPERATIONS.get() {
            for op in ops.lock().drain(..) {
                OperationsPlugin::add(true, op);
            }
        }
    }
}

struct Claims {
    // Standard Claims
    pub sub: String,
    pub iss: String,
    pub aud: String, // Can be String or Vec<String>, your payload has a String
    pub exp: usize,
    pub iat: usize,

    // Custom & OIDC Claims
    pub email: String,
    pub email_verified: bool,
    pub picture: String,
    pub preferred_username: String,
    pub project_id: String,
    pub login_method: String,

    // Nullable fields must be Option<T>
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
}
