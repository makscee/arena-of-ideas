use spacetimedb::ReducerContext;
#[allow(unused_imports)]
use spacetimedb::Table;

#[spacetimedb::table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    pub identity: spacetimedb::Identity,
    pub name: String,
}

#[spacetimedb::reducer(init)]
pub fn init(_ctx: &ReducerContext) {
    log::info!("Arena of Ideas server initialized");
}

#[spacetimedb::reducer(client_connected)]
pub fn client_connected(_ctx: &ReducerContext) {
    log::info!("Client connected");
}

#[spacetimedb::reducer(client_disconnected)]
pub fn client_disconnected(_ctx: &ReducerContext) {
    log::info!("Client disconnected");
}
