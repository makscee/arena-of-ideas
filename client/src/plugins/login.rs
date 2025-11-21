use bevy::ecs::{event::Event, message::MessageWriter, observer::On};
use bevy_http_client::{
    HttpClient, HttpClientPlugin, HttpRequest, HttpResponse, HttpResponseError,
};
use crossbeam_channel::bounded;
use spacetimedb_sdk::Table;

use crate::{
    auth_utils::{generate_csrf_state, pkce_challenge, pkce_verifier},
    login,
};

use super::*;

pub const HOME_DIR: &str = ".aoi";
pub fn home_dir_path() -> PathBuf {
    let mut path = home::home_dir().unwrap();
    path.push(HOME_DIR);
    std::fs::create_dir_all(&path).unwrap();
    path
}

pub struct LoginPlugin;

impl Plugin for LoginPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HttpClientPlugin)
            .add_systems(OnEnter(GameState::Login), Self::login)
            .add_observer(spacetimeauth_login)
            .add_systems(Update, (handle_response, handle_error))
            .init_resource::<LoginData>();
    }
}

#[derive(Resource, Default)]
pub struct LoginData {
    login_requested: bool,
    pub id_token: Option<String>,
    access_token: Option<String>,
}

#[derive(Event)]
struct SpaceLogin;

impl LoginPlugin {
    fn login() {
        cn().reducers.on_login_by_identity(|e| {
            if !e.check_identity() {
                return;
            }
            e.event.on_success_error(LoginPlugin::complete, || {
                op(|world| {
                    world.resource_mut::<LoginData>().login_requested = false;
                    pd_mut(|pd| pd.client_state.last_logged_in = None);
                });
            });
        });
        cn().reducers.on_login(|e, _, _| {
            if !e.check_identity() {
                return;
            }
            e.event.on_success_error(LoginPlugin::complete, || {
                op(|world| {
                    world.resource_mut::<LoginData>().login_requested = false;
                    pd_mut(|pd| pd.client_state.last_logged_in = None);
                });
            });
        });
    }
    fn complete() {
        subscribe_game(Self::on_subscribed);
        subscribe_reducers();
    }
    fn find_identity_node(identity: Identity) -> Option<NPlayerIdentity> {
        let data = NPlayerIdentity::new(0, 0, Some(identity.to_string())).get_data();
        cn().db
            .nodes_world()
            .iter()
            .find(|n| n.data == data)
            .map(|n| n.to_node().unwrap())
    }
    fn on_subscribed() {
        op(|world| {
            let identity = ConnectOption::get(world).identity;
            let Some(identity_node) = Self::find_identity_node(identity) else {
                "Failed to find Player after login".notify_error(world);
                return;
            };
            dbg!(&identity_node);

            if let Some(player) = cn()
                .db
                .node_links()
                .iter()
                .find(|link| link.child == identity_node.id)
                .and_then(|link| NPlayer::db_load(link.parent).ok())
            {
                pd_mut(|pd| {
                    pd.client_state.last_logged_in = Some((player.player_name.clone(), identity));
                });
                LoginOption { player }.save(world);
            } else {
                error!("Failed to load NPlayer by Identity");
            }
            GameState::proceed(world);
        });
    }
    pub fn pane_login(ui: &mut Ui, world: &mut World) {
        ui.vertical_centered_justified(|ui| {
            let mut ld = world.resource_mut::<LoginData>();
            ui.add_space(ui.available_height() * 0.3);
            ui.set_width(350.0.at_most(ui.available_width()));
            let cs = pd().client_state.clone();
            if let Some((name, identity)) = &cs.last_logged_in {
                format!("Login as {name}")
                    .cstr_cs(high_contrast_text(), CstrStyle::Heading2)
                    .label(ui);
                if (pd().client_settings.auto_login
                    || Button::new("Login")
                        .enabled(!ld.login_requested)
                        .ui(ui)
                        .clicked())
                    && !ld.login_requested
                {
                    ld.login_requested = true;
                    //let _ = cn().reducers.login_by_identity();
                }
                br(ui);
                if Button::new("Logout")
                    .enabled(!ld.login_requested)
                    .gray(ui)
                    .ui(ui)
                    .clicked()
                    || ConnectOption::get(world).identity != *identity
                {
                    pd_mut(|data| data.client_state.last_logged_in = None);
                }
            } else {
                "Arena of Ideas"
                    .cstr_cs(high_contrast_text(), CstrStyle::Heading)
                    .label(ui);
                space(ui);
                br(ui);
                if Button::new("Login via SpacetimeAuth").ui(ui).clicked() {
                    world.trigger(SpaceLogin);
                }
            }
        });
    }
}

fn spacetimeauth_login(_: On<SpaceLogin>, mut ev_request: MessageWriter<HttpRequest>) {
    let verifier = pkce_verifier();
    let challenge = pkce_challenge(&verifier);
    let state = generate_csrf_state();
    let stdb_client_id = "client_031LPfBM8jHxvge4NX9CNr";
    let redirect_uri = "http://127.0.0.1:42069";
    let auth_url = format!(
        "https://auth.spacetimedb.com/oidc/auth?client_id={}&redirect_uri={}&scope=openid%20email%20profile&response_type=code&response_mode=query&code_challenge_method=S256&code_challenge={}&state={}",
        stdb_client_id, redirect_uri, challenge, state
    );
    let expected_state = state.clone();
    let (s1, r1) = bounded::<String>(2);
    IoTaskPool::get()
        .spawn(async move {
            let server = tiny_http::Server::http("127.0.0.1:42069").unwrap();
            info!("Server started at http://127.0.0.1:42069");
            let request = server.recv().unwrap();
            info!("Received request: {:?}", request);
            if *request.method() == tiny_http::Method::Get {
                let url = request.url();
                // Parse query parameters from URL
                let query_string = url.trim_start_matches("/?");
                let mut code = None;
                let mut state = None;

                for param in query_string.split('&') {
                    let parts: Vec<&str> = param.split('=').collect();
                    if parts.len() == 2 {
                        match parts[0] {
                            "code" => code = Some(parts[1].to_string()),
                            "state" => state = Some(parts[1].to_string()),
                            _ => {}
                        }
                    }
                }
                let response = tiny_http::Response::from_string(
                    "Login successful! You can close this window.",
                );
                if let Err(e) = request.respond(response) {
                    error!("Failed to respond to request: {}", e);
                }
                if let (Some(received_state), Some(auth_code)) = (state, code) {
                    if received_state == expected_state {
                        info!("Successfully received auth code: {}", auth_code);
                        s1.send(auth_code).unwrap();
                    }
                }
            }
        })
        .detach();
    let _jh = open::that_in_background(auth_url);
    let url = format!("https://auth.spacetimedb.com/oidc/token");
    let auth_code = r1.recv().unwrap();
    let verifier_str =
        String::from_utf8(verifier).expect("Code verifier bytes were not valid UTF-8");
    let body = format!(
        "client_id={}&\
                      code={}&\
                      code_verifier={}&\
                      grant_type=authorization_code&\
                      redirect_uri=http://127.0.0.1:42069",
        stdb_client_id, auth_code, &verifier_str
    );
    match HttpClient::new().post(url).form_encoded(&body).try_build() {
        Ok(request) => {
            ev_request.write(request);
        }
        Err(e) => {
            eprintln!("Failed to build request: {}", e);
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
    pub id_token: String,
}

fn handle_response(mut ev_resp: MessageReader<HttpResponse>, mut login_data: ResMut<LoginData>) {
    for response in ev_resp.read() {
        // info!("response {}", response.text().unwrap());
        let token_response: TokenResponse = response.json().unwrap();
        info!("response {:#?}", token_response);
        login_data.access_token = Some(token_response.access_token);
        login_data.id_token = Some(token_response.id_token);
    }
}

fn handle_error(mut ev_error: MessageReader<HttpResponseError>) {
    for error in ev_error.read() {
        println!("Error retrieving IP: {}", error.err);
    }
}
