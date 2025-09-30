use axum::{Router, routing::get};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    authorize::disco_auth,
    csrf::{CsrfCache, get_csrf, start_cleanup},
};

mod authorize;
mod csrf;

pub(crate) type SharedCache = Arc<Mutex<CsrfCache>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    // Axum Routes
    let addr = "0.0.0.0:42069";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router()).await.unwrap();
    Ok(())
}

fn router() -> Router {
    let cache = SharedCache::new(Mutex::new(CsrfCache::new()));
    let cleanup_cache = cache.clone();
    tokio::spawn(async move {
        start_cleanup(cleanup_cache).await;
    });
    Router::new()
        .route("/", get(disco_auth))
        .route("/csrf/{identity}", get(get_csrf))
        .with_state(cache)
}
