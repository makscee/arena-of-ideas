use spacetimedb_sdk::{Error, Identity};

use crate::module_bindings::{DbConnection, ErrorContext};

const HOST: &str = "https://game-server.izaforge.com";
const DB_NAME: &str = "aoidev";

pub(crate) fn connect_to_db() -> DbConnection {
    DbConnection::builder()
        .on_connect(on_connected)
        .on_connect_error(on_connect_error)
        .with_module_name(DB_NAME)
        .with_uri(HOST)
        .build()
        .expect("Failed to connect")
}

pub(crate) fn on_connected(_ctx: &DbConnection, _identity: Identity, _token: &str) {
    println!("Connected to SpacetimeDB.");
}

pub(crate) fn on_connect_error(_ctx: &ErrorContext, err: Error) {
    eprintln!("Connection error: {:?}", err);
    std::process::exit(1);
}
