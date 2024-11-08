#![allow(unused)]

use std::thread::sleep;

use super::*;
use spacetimedb_lib::{
    de::{serde::DeserializeWrapper, Deserialize},
    ser::{serde::SerializeWrapper, Serialize},
};

const DOWNLOAD_FOLDER: &str = "migration_download/";
const UPLOAD_FOLDER: &str = "migration_upload/";
pub struct MigrationPlugin;

impl Plugin for MigrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MigrationDownload), Self::download)
            .add_systems(OnEnter(GameState::MigrationUpload), Self::upload);
    }
}
pub fn save_to_download_folder(file: &str, json: String) {
    let path = MigrationPlugin::path_download(file);
    match std::fs::write(path, json) {
        Ok(_) => {
            info!("{} {}", "Store successful:".dimmed(), file.green())
        }
        Err(e) => {
            error!("{} {}", "Store error:", e.to_string().red())
        }
    }
}
impl MigrationPlugin {
    fn path_download(name: &str) -> PathBuf {
        let mut path = home_dir_path();
        path.push(DOWNLOAD_FOLDER);
        path.push(format!("{name}.json"));
        path
    }
    fn path_upload() -> PathBuf {
        let mut path = home_dir_path();
        path.push(UPLOAD_FOLDER);
        path
    }
    fn path_create() {
        let mut path = home_dir_path();
        path.push(UPLOAD_FOLDER);
        std::fs::create_dir_all(path);
        let mut path = home_dir_path();
        path.push(DOWNLOAD_FOLDER);
        std::fs::create_dir_all(path);
    }
    fn download() {
        Self::path_create();
        StdbQuery::subscribe(StdbTable::iter().map(|t| t.full()), |world| {
            for table in StdbTable::iter() {
                let json = table.get_json_data();
                save_to_download_folder(table.as_ref(), json);
            }
            app_exit(world);
        });
    }
    fn upload() {
        Self::path_create();
        let paths = std::fs::read_dir(Self::path_upload()).unwrap();
        let mut gd = GameData::default();
        for path in paths {
            let path = path.unwrap();
            let table = path
                .file_name()
                .into_string()
                .unwrap()
                .trim_end_matches(".json")
                .to_string();
            let table = StdbTable::from_str(&table).unwrap();
            let json = std::fs::read_to_string(&path.path()).unwrap();
            table.fill_from_json_data(&json, &mut gd);
        }
        let mut nid = 572125;
        let mut next_id = || {
            nid += 1;
            nid
        };

        let mut logins: HashMap<u64, u64> = default();
        let mut player_stats: HashMap<u64, TPlayerStats> =
            HashMap::from_iter(gd.player.iter().map(|p| {
                (
                    p.id,
                    TPlayerStats {
                        id: next_id(),
                        season: 1,
                        owner: p.id,
                        time_played: 0,
                        quests_completed: 0,
                        credits_earned: 0,
                    },
                )
            }));
        let mut player_game_stats: HashMap<(u64, u32, GameMode), TPlayerGameStats> = default();
        for event in gd.global_event.iter().sorted_by_key(|e| e.ts) {
            match &event.event {
                GlobalEvent::LogIn => {
                    logins.insert(event.owner, event.ts);
                }
                GlobalEvent::LogOut => {
                    if let Some(ts) = logins.remove(&event.owner) {
                        player_stats.get_mut(&event.owner).unwrap().time_played += event.ts - ts;
                    }
                }
                GlobalEvent::Register => {}
                GlobalEvent::RunFinish(_) => {}
                GlobalEvent::RunStart(_) => {}
                GlobalEvent::BattleFinish(_) => {}
                GlobalEvent::MetaShopBuy(_) => {}
                GlobalEvent::GameShopBuy(_) => {}
                GlobalEvent::GameShopSkip(_) => {}
                GlobalEvent::GameShopSell(_) => {}
                GlobalEvent::AuctionBuy(_) => {}
                GlobalEvent::AuctionCancel(_) => {}
                GlobalEvent::AuctionPost(_) => {}
                GlobalEvent::ReceiveUnit(_) => {}
                GlobalEvent::DismantleUnit(_) => {}
                GlobalEvent::CraftUnit(_) => {}
                GlobalEvent::ReceiveUnitShard(_) => {}
                GlobalEvent::ReceiveRainbowShard(_) => {}
                GlobalEvent::ReceiveLootbox(_) => {}
                GlobalEvent::OpenLootbox(_) => {}
                GlobalEvent::QuestAccepted(_) => {}
                GlobalEvent::QuestComplete(_) => {
                    player_stats.get_mut(&event.owner).unwrap().quests_completed += 1;
                }
                GlobalEvent::Fuse(_) => {}
            }
        }
        for run in &gd.arena_run_archive {
            let stats = player_game_stats
                .entry((run.owner, run.season, run.mode))
                .or_insert_with(|| TPlayerGameStats {
                    id: next_id(),
                    season: run.season,
                    owner: run.owner,
                    mode: run.mode,
                    runs: 0,
                    floors: default(),
                    champion: 0,
                    boss: 0,
                });
            let floor = run.floor as usize;
            if stats.floors.len() < floor + 1 {
                stats.floors.resize(floor + 1, 0);
            }
            stats.floors[floor] += 1;
            stats.runs += 1;
        }
        gd.player_stats = player_stats.into_values().collect_vec();
        gd.player_game_stats = player_game_stats.into_values().collect_vec();
        dbg!(nid);
        cn().reducers.upload_game_data(nid, gd);
        cn().reducers.on_upload_game_data(|e, _, _| {
            info!("Upload finished");
            app_exit_op();
        });
    }
}
