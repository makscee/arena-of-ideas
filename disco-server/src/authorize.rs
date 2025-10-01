use axum::extract::{Query, State};
use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, RedirectUrl, TokenResponse, TokenUrl,
    basic::BasicClient,
};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::SharedCache;
use std::env;

#[derive(Deserialize)]
pub struct AuthResponse {
    code: String,
    state: String,
}

#[derive(Deserialize, Debug)]
struct GuildMember {
    roles: Vec<String>,
    user: DiscordUserInfo,
}

#[derive(Deserialize, Debug)]
struct DiscordUserInfo {
    id: String,
    username: String,
}

pub(crate) async fn disco_auth(
    State(state): State<SharedCache>,
    query: Query<AuthResponse>,
) -> Result<String, StatusCode> {
    let mut cache = state.lock().await;
    let id = cache.take_by_state(&query.state);
    if id.is_none() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    } else {
        // set oauth2 client
        let application_id =
            env::var("APPLICATION_ID").expect("APPLICATION_ID must be set in .env");
        let app_secret = env::var("APP_SECRET").expect("APP_SECRET must be set in .env");

        let client = BasicClient::new(ClientId::new(application_id))
            .set_client_secret(ClientSecret::new(app_secret))
            .set_redirect_uri(
                RedirectUrl::new("http://localhost:42069/".into())
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            )
            .set_token_uri(
                TokenUrl::new("https://discord.com/api/oauth2/token".into())
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            );

        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Client should build");
        let token = client
            .exchange_code(AuthorizationCode::new(query.code.clone()))
            .request_async(&http_client)
            .await
            .unwrap();

        let role_response = http_client
            .get("https://discord.com/api/v10/users/@me/guilds/1034174161679044660/member")
            .header(
                "Authorization",
                format!("Bearer {}", token.access_token().secret()),
            )
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if !role_response.status().is_success() {
            return Err(StatusCode::from_u16(role_response.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR));
        }

        let role_data: GuildMember = role_response
            .json()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        return Ok(format!("Welcome {:#?}", role_data));
    }
}
