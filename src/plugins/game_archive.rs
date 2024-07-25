use spacetimedb_lib::{
    de::{serde::DeserializeWrapper, Deserialize},
    ser::{serde::SerializeWrapper, Serialize},
};

use super::*;

const ARCHIVE_FILE: &str = "game_archive.json";
pub struct GameArchivePlugin;

#[derive(Serialize, Deserialize, Debug)]
struct GameArchive {
    global_settings: GlobalSettings,
    global_data: GlobalData,
    users: Vec<TUser>,
    base_units: Vec<TBaseUnit>,
    houses: Vec<THouse>,
    abilities: Vec<TAbility>,
    statuses: Vec<TStatus>,
    representations: Vec<TRepresentation>,
    arena_runs: Vec<TArenaRun>,
    arena_runs_archive: Vec<TArenaRunArchive>,
    arena_leaderboard: Vec<TArenaLeaderboard>,
    teams: Vec<TTeam>,
    battles: Vec<TBattle>,
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
            global_settings: GlobalSettings::filter_by_always_zero(0).unwrap(),
            global_data: GlobalData::filter_by_always_zero(0).unwrap(),
            users: TUser::iter().collect_vec(),
            base_units: TBaseUnit::iter().collect_vec(),
            houses: THouse::iter().collect_vec(),
            abilities: TAbility::iter().collect_vec(),
            statuses: TStatus::iter().collect_vec(),
            representations: TRepresentation::iter().collect_vec(),
            arena_runs: TArenaRun::iter().collect_vec(),
            arena_runs_archive: TArenaRunArchive::iter().collect_vec(),
            arena_leaderboard: TArenaLeaderboard::iter().collect_vec(),
            teams: TTeam::iter().collect_vec(),
            battles: TBattle::iter().collect_vec(),
        };
        let data = serde_json::to_string(&SerializeWrapper::new(ga))
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
            global_settings,
            global_data,
            users,
            base_units,
            houses,
            abilities,
            statuses,
            representations,
            arena_runs,
            arena_runs_archive,
            arena_leaderboard,
            teams,
            battles,
        } = serde_json::from_str::<DeserializeWrapper<GameArchive>>(data)
            .expect("Failed to deserialize game data")
            .0;
        upload_game_archive(
            global_settings,
            global_data,
            users,
            base_units,
            houses,
            abilities,
            statuses,
            representations,
            arena_runs,
            arena_runs_archive,
            arena_leaderboard,
            teams,
            battles,
        );
        once_on_upload_game_archive(|_, _, status, _, _, _, _, _, _, _, _, _, _, _, _, _| {
            match status {
                StdbStatus::Committed => {}
                StdbStatus::Failed(e) => e.notify_error(),
                _ => panic!(),
            };
            app_exit_op();
        });
    }
}
