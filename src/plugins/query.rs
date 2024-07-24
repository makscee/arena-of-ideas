use spacetimedb_sdk::subscribe_owned;

use super::*;

pub struct StdbQueryPlugin;

impl Plugin for StdbQueryPlugin {
    fn build(&self, _: &mut App) {}
}

static SERVER_DATA: Mutex<QueryData> = Mutex::new(QueryData {
    subscribed_queries: Vec::new(),
});

#[derive(Resource)]
struct QueryData {
    subscribed_queries: Vec<StdbQuery>,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Display, Debug, Serialize)]
pub enum StdbQuery {
    Connect,
    Game(GID),
    GameFull,
    BaseUnits,
    BattleHistory,
}

impl StdbQuery {
    pub fn subscribe(self) -> bool {
        StdbQueryPlugin::subscribe_vec([self].into())
    }
    pub fn is_subscribed(self) -> bool {
        StdbQueryPlugin::is_subscribed(self)
    }
    fn get_strings(&self) -> Vec<String> {
        match self {
            StdbQuery::Connect => [
                "select * from TUser".into(),
                "select * from GlobalData".into(),
            ]
            .into(),
            StdbQuery::Game(uid) => [
                format!("select * from TArenaRun where owner = {uid}"),
                format!("select * from TBattle where owner = {uid}"),
                "select * from TBaseUnit".into(),
                "select * from TRepresentation".into(),
                "select * from GlobalSettings".into(),
                "select * from THouse".into(),
                "select * from TAbility".into(),
                "select * from TStatus".into(),
                "select * from TTeam".into(),
                "select * from TArenaLeaderboard".into(),
            ]
            .into(),
            StdbQuery::GameFull => [
                "select * from GlobalSettings".into(),
                "select * from GlobalData".into(),
                "select * from TUser".into(),
                "select * from TBaseUnit".into(),
                "select * from THouse".into(),
                "select * from TAbility".into(),
                "select * from TStatus".into(),
                "select * from TRepresentation".into(),
                "select * from TArenaRun".into(),
                "select * from TArenaRunArchive".into(),
                "select * from TArenaLeaderboard".into(),
                "select * from TArenaPool".into(),
                "select * from TTeam".into(),
                "select * from TBattle".into(),
            ]
            .into(),
            StdbQuery::BaseUnits => ["select * from TBaseUnit".into()].into(),
            StdbQuery::BattleHistory => ["select * from TBattle".into()].into(),
        }
    }
}

impl StdbQueryPlugin {
    pub fn subscribe_vec(queries: Vec<StdbQuery>) -> bool {
        let subscribed = &mut SERVER_DATA.lock().unwrap().subscribed_queries;
        let mut added = false;
        for q in queries {
            if !subscribed.contains(&q) {
                added = true;
                subscribed.push(q);
                info!("Add {q} to table subscriptions");
            }
        }
        if !added {
            return false;
        }
        let strings = subscribed
            .iter()
            .flat_map(|q| q.get_strings())
            .collect_vec();
        for q in &strings {
            info!("{} {}", "Subscribe:".dimmed(), q.purple());
        }
        if let Err(e) = subscribe_owned(strings) {
            panic!("Failed to subscribe: {e}");
        }
        return added;
    }
    pub fn is_subscribed(q: StdbQuery) -> bool {
        SERVER_DATA.lock().unwrap().subscribed_queries.contains(&q)
    }
}
