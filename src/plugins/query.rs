use spacetimedb_sdk::subscribe_owned;

use super::*;

pub struct QueryPlugin;

impl Plugin for QueryPlugin {
    fn build(&self, _: &mut App) {}
}

static SERVER_DATA: Mutex<QueryData> = Mutex::new(QueryData {
    subscribed_queries: Vec::new(),
});

#[derive(Resource)]
struct QueryData {
    subscribed_queries: Vec<String>,
}

pub const QUERY_LEADERBOARD: &str = "select * from TArenaLeaderboard";
pub const QUERY_BATTLE_HISTORY: &str = "select * from TBattle";
pub const QUERY_BASE_UNITS: &str = "select * from TBaseUnit";

impl QueryPlugin {
    pub fn subscribe(queries: Vec<String>) -> bool {
        let subscribed = &mut SERVER_DATA.lock().unwrap().subscribed_queries;
        let mut added = false;
        for q in queries {
            if !subscribed.contains(&q) {
                added = true;
                info!(
                    "{} {}",
                    "New table subscription:".dimmed(),
                    q.bold().purple()
                );
                subscribed.push(q);
            }
        }
        if !added {
            return false;
        }
        if let Err(e) = subscribe_owned(subscribed.clone()) {
            panic!("Failed to subscribe: {e}");
        }
        return added;
    }
    pub fn is_subscribed(q: &str) -> bool {
        SERVER_DATA
            .lock()
            .unwrap()
            .subscribed_queries
            .iter()
            .any(|d| d.eq(q))
    }
    pub fn subscribe_connect() {
        Self::subscribe(
            [
                "select * from TUser".to_owned(),
                "select * from GlobalData".to_owned(),
            ]
            .into(),
        );
    }
    pub fn subscribe_game() {
        let uid = user_id();
        let q = [
            format!("select * from TArenaRun where owner = {uid}"),
            "select * from TBaseUnit".to_owned(),
            "select * from TRepresentation".to_owned(),
            "select * from GlobalSettings".to_owned(),
            "select * from THouse".to_owned(),
            "select * from TAbility".to_owned(),
            "select * from TStatus".to_owned(),
            "select * from TTeam".to_owned(),
            "select * from TArenaLeaderboard".to_owned(),
            format!("select * from TBattle where owner = {uid}"),
        ];
        Self::subscribe(q.into());
    }
}
