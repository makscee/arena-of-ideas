use bevy::ecs::{event::Event, message::MessageWriter, observer::On};
use bevy_http_client::{
    HttpClient, HttpClientPlugin, HttpRequest, HttpResponse, HttpResponseError,
};
use crossbeam_channel::bounded;

use super::*;
use crate::auth_utils::{generate_csrf_state, pkce_challenge, pkce_verifier};

pub struct StdbAuthPlugin;

impl Plugin for StdbAuthPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HttpClientPlugin)
            .init_resource::<AuthOption>()
            .add_observer(spacetimeauth_login)
            .add_systems(Update, (handle_response, handle_error));
    }
}

impl StdbAuthPlugin {
    pub fn pane_auth(ui: &mut Ui, world: &mut World) {
        ui.vertical_centered_justified(|ui| {
            let mut ld = world.resource_mut::<AuthOption>();
            ui.add_space(ui.available_height() * 0.3);
            ui.set_width(350.0.at_most(ui.available_width()));
            let cs = pd().client_state.clone();
            if let Some((name, identity)) = &cs.last_logged_in {
                format!("Login as {name}")
                    .cstr_cs(high_contrast_text(), CstrStyle::Heading2)
                    .label(ui);
                if (pd().client_settings.auto_login
                    || Button::new("Login")
                        //.enabled(!ld.id_token.is_none())
                        .ui(ui)
                        .clicked())
                // && !ld.id_token.is_none()
                {
                    ld.id_token = creds_store().load().expect("Token not found");
                    let _ = cn().reducers.login_by_identity();
                }
                br(ui);
                if Button::new("Logout")
                    //.enabled(!ld.id_token.is_none())
                    .gray(ui)
                    .ui(ui)
                    .clicked()
                {
                    pd_mut(|data| data.client_state.last_logged_in = None);
                }
            } else {
                "Arena of Ideas"
                    .cstr_cs(high_contrast_text(), CstrStyle::Heading)
                    .label(ui);
                space(ui);
                if Button::new("Login via SpacetimeAuth").ui(ui).clicked() {
                    world.trigger(SpaceLogin);
                }
            }
        });
    }
}

#[derive(Event)]
pub(crate) struct SpaceLogin;

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
            dbg!("Server started at http://127.0.0.1:42069");
            let request = server.recv().unwrap();
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

fn handle_response(mut ev_resp: MessageReader<HttpResponse>, mut cmd: Commands) {
    for response in ev_resp.read() {
        let token_response: TokenResponse = response.json().unwrap();
        info!("response {:#?}", token_response);
        cmd.insert_resource(AuthOption {
            id_token: Some(token_response.id_token),
        });
        op(|world| {
            GameState::proceed(world);
        });
    }
}

fn handle_error(mut ev_error: MessageReader<HttpResponseError>) {
    for error in ev_error.read() {
        println!("Error retrieving IP: {}", error.err);
    }
}
