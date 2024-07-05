use spacetimedb_sdk::subscribe_owned;

use super::*;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {}
}

static SERVER_DATA: Mutex<ServerData> = Mutex::new(ServerData {
    subscribed_queries: Vec::new(),
});

#[derive(Resource)]
struct ServerData {
    subscribed_queries: Vec<String>,
}

impl ServerPlugin {
    pub fn subscribe(query: String) {
        let q = &mut SERVER_DATA.lock().unwrap().subscribed_queries;
        q.push(query);
        if let Err(e) = subscribe_owned(q.clone()) {
            panic!("Failed to subscribe: {e}");
        }
    }
    pub fn subscribe_users() {
        Self::subscribe("select * from User".to_owned());
    }
    pub fn subscribe_run() {
        Self::subscribe(format!("select * from Run where user_id = {}", user_id()));
    }
}
