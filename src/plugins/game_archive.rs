#![allow(unused)]

use super::*;
use spacetimedb_lib::{
    de::{serde::DeserializeWrapper, Deserialize},
    ser::{serde::SerializeWrapper, Serialize},
};

const ARCHIVE_FILE: &str = "game_archive copy 2.json";
pub struct GameArchivePlugin;

#[derive(Serialize, Deserialize, Debug)]
struct GameArchive {
    // global_settings: GlobalSettings,
    // global_data: GlobalData,
    // base_units: Vec<TBaseUnit>,
    // houses: Vec<THouse>,
    // abilities: Vec<TAbility>,
    // statuses: Vec<TStatus>,
    // representations: Vec<TRepresentation>,
    // arena_runs: Vec<TArenaRun>,
    next_id: u64,
    users: Vec<TUser>,
    arena_runs_archive: Vec<TArenaRunArchive>,
    arena_leaderboard: Vec<TArenaLeaderboard>,
    teams: Vec<TTeam>,
    battles: Vec<TBattle>,
    wallets: Vec<TWallet>,
    unit_items: Vec<TUnitItem>,
    unit_shards: Vec<TUnitShardItem>,
    lootboxes: Vec<TLootboxItem>,
}

impl Plugin for GameArchivePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameArchiveDownload), Self::download)
            .add_systems(OnEnter(GameState::GameArchiveUpload), Self::upload);
    }
}
impl GameArchivePlugin {
    fn path() -> PathBuf {
        let mut path = home_dir_path();
        path.push(ARCHIVE_FILE);
        path
    }
    fn download() {
        let ga = GameArchive {
            // global_settings: GlobalSettings::find_by_always_zero(0).unwrap(),
            // global_data: GlobalData::find_by_always_zero(0).unwrap(),
            // base_units: TBaseUnit::iter().collect_vec(),
            // houses: THouse::iter().collect_vec(),
            // abilities: TAbility::iter().collect_vec(),
            // statuses: TStatus::iter().collect_vec(),
            // representations: TRepresentation::iter().collect_vec(),
            // arena_runs: TArenaRun::iter().collect_vec(),
            next_id: GlobalData::current().next_id,
            users: TUser::iter().collect_vec(),
            arena_runs_archive: TArenaRunArchive::iter().collect_vec(),
            arena_leaderboard: TArenaLeaderboard::iter().collect_vec(),
            teams: TTeam::iter().collect_vec(),
            battles: TBattle::iter().collect_vec(),
            wallets: TWallet::iter().collect_vec(),
            unit_items: TUnitItem::iter().collect_vec(),
            unit_shards: TUnitShardItem::iter().collect_vec(),
            lootboxes: TLootboxItem::iter().collect_vec(),
        };
        let data = serde_json::to_string_pretty(&SerializeWrapper::new(ga))
            .expect("Failed to serialize game data");

        match std::fs::write(Self::path(), data) {
            Ok(_) => {
                info!("Store successful")
            }
            Err(e) => {
                error!("Store error: {e}")
            }
        }
        app_exit_op();
    }
    fn upload() {
        let data = &std::fs::read_to_string(&Self::path()).unwrap();
        let GameArchive {
            // global_settings,
            // global_data,
            // base_units,
            // houses,
            // abilities,
            // statuses,
            // representations,
            // arena_runs,
            next_id,
            users,
            arena_runs_archive,
            arena_leaderboard,
            teams,
            battles,
            wallets,
            unit_items,
            unit_shards,
            lootboxes,
        } = serde_json::from_str::<DeserializeWrapper<GameArchive>>(data)
            .expect("Failed to deserialize game data")
            .0;
        info!("Start upload...");
        upload_game_archive(
            next_id,
            users,
            arena_runs_archive,
            arena_leaderboard,
            teams,
            battles,
            wallets,
            unit_items,
            unit_shards,
            lootboxes,
        );
        once_on_upload_game_archive(|_, _, status, _, _, _, _, _, _, _, _, _, _| {
            match status {
                StdbStatus::Committed => info!("{}", "Upload successful".green()),
                StdbStatus::Failed(e) => e.notify_error_op(),
                _ => panic!(),
            };
            app_exit_op();
        });
    }
}
