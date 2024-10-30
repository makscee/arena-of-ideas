#![allow(unused)]

use std::thread::sleep;

use super::*;
use spacetimedb_lib::{
    de::{serde::DeserializeWrapper, Deserialize},
    ser::{serde::SerializeWrapper, Serialize},
};

const DOWNLOAD_FOLDER: &str = "archive_download/";
const UPLOAD_FOLDER: &str = "archive_upload/";
pub struct GameArchivePlugin;

impl Plugin for GameArchivePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameArchiveDownload), Self::download)
            .add_systems(OnEnter(GameState::GameArchiveUpload), Self::upload);
    }
}
impl GameArchivePlugin {
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
    fn save_to_file(name: &str, data: String) {
        let path = Self::path_download(name);
        match std::fs::write(path, data) {
            Ok(_) => {
                info!("{} {}", "Store successful:".dimmed(), name.green())
            }
            Err(e) => {
                error!("{} {}", "Store error:", e.to_string().red())
            }
        }
    }
    fn download() {
        Self::path_create();
        StdbQuery::subscribe(StdbTable::iter().map(|t| t.full()), |world| {
            for table in StdbTable::iter() {
                let json = table.get_json_data();
                Self::save_to_file(table.as_ref(), json);
            }
            app_exit(world);
        });
    }
    fn upload() {
        Self::path_create();
        let paths = std::fs::read_dir(Self::path_upload()).unwrap();

        for path in paths {
            let table = path
                .unwrap()
                .file_name()
                .into_string()
                .unwrap()
                .trim_end_matches(".json")
                .to_string();
            let table = StdbTable::from_str(&table).unwrap();
        }
        // let data = &std::fs::read_to_string(&Self::path()).unwrap();
        // let GameArchive {
        //     // global_settings,
        //     // global_data,
        //     // base_units,
        //     // houses,
        //     // abilities,
        //     // statuses,
        //     // representations,
        //     // arena_runs,
        //     next_id,
        //     users,
        //     arena_leaderboard,
        //     teams,
        //     wallets,
        //     unit_items,
        //     unit_shards,
        //     lootboxes,
        // } = serde_json::from_str::<DeserializeWrapper<GameArchive>>(data)
        //     .expect("Failed to deserialize game data")
        //     .0;
        // info!("Start upload...");
        // upload_game_archive(
        //     next_id,
        //     users,
        //     arena_leaderboard,
        //     teams,
        //     wallets,
        //     unit_items,
        //     unit_shards,
        //     lootboxes,
        // );
        // once_on_upload_game_archive(|_, _, status, _, _, _, _, _, _, _, _| {
        //     match status {
        //         StdbStatus::Committed => info!("{}", "Upload successful".green()),
        //         StdbStatus::Failed(e) => e.notify_error_op(),
        //         _ => panic!(),
        //     };
        //     app_exit_op();
        // });
    }
}
