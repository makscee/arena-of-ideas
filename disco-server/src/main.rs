use axum::{Router, routing::get};
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

use crate::{
    authorize::disco_auth,
    csrf::{CsrfCache, get_csrf, start_cleanup},
    module_bindings::DbConnection,
    stdb::connect_to_db,
};

mod authorize;
mod csrf;
mod module_bindings;
mod secret;
mod stdb;

static DB_CONNECTION: OnceLock<DbConnection> = OnceLock::new();

pub(crate) type SharedCache = Arc<Mutex<CsrfCache>>;

// Accessor function for the database connection
pub fn db() -> &'static DbConnection {
    DB_CONNECTION.get().expect("Database not connected")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to SpacetimeDB
    let ctx = connect_to_db();
    ctx.run_threaded();
    DB_CONNECTION.set(ctx).ok().unwrap();
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
