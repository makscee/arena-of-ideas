use axum::extract::{Query, State};
use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, RedirectUrl, TokenResponse, TokenUrl,
    basic::BasicClient,
};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    SharedCache,
    secret::{APP_SECRET, APPLICATION_ID},
};

#[derive(Deserialize)]
pub struct AuthResponse {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct DiscordUser {
    username: String,
}

pub(crate) async fn disco_auth(
    State(state): State<SharedCache>,
    query: Query<AuthResponse>,
) -> Result<String, StatusCode> {
    let mut cache = state.lock().await;
    if cache.take_by_state(&query.state).is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    } else {
        // set oauth2 client
        let client = BasicClient::new(ClientId::new(APPLICATION_ID.into()))
            .set_client_secret(ClientSecret::new(APP_SECRET.into()))
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

        let user_response = http_client
            .get("https://discord.com/api/v10/users/@me")
            .header(
                "Authorization",
                format!("Bearer {}", token.access_token().secret()),
            )
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if !user_response.status().is_success() {
            return Err(StatusCode::from_u16(user_response.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR));
        }

        let user_data: DiscordUser = user_response
            .json()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let username = user_data.username;
        return Ok(format!(
            "Welcome {}. You can now close this Tab and switch to the Game",
            username
        ));
    }
}
