use spacetimedb_sdk::subscribe_owned;

use super::*;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, _: &mut App) {}
}

static SERVER_DATA: Mutex<ServerData> = Mutex::new(ServerData {
    subscribed_queries: Vec::new(),
});

#[derive(Resource)]
struct ServerData {
    subscribed_queries: Vec<String>,
}

impl ServerPlugin {
    pub fn subscribe(queries: Vec<String>) {
        let q = &mut SERVER_DATA.lock().unwrap().subscribed_queries;
        q.extend(queries.into_iter());
        if let Err(e) = subscribe_owned(q.clone()) {
            panic!("Failed to subscribe: {e}");
        }
    }
    pub fn subscribe_connect() {
        Self::subscribe(
            [
                "select * from User".to_owned(),
                "select * from GlobalData".to_owned(),
            ]
            .into(),
        );
    }
    pub fn subscribe_game() {
        let uid = user_id();
        let q = [
            format!("select * from Run where owner = {uid}"),
            "select * from BaseUnit".to_owned(),
            "select * from TRepresentation".to_owned(),
            "select * from GlobalSettings".to_owned(),
            format!("select * from TTeam where owner = {uid}"),
            format!("select * from TBattle where owner = {uid}"),
        ];
        Self::subscribe(q.into());
    }
}
