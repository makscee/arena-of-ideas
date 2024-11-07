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
        cn().reducers.upload_game_data(572125, gd);
        cn().reducers.on_upload_game_data(|e, _, _| {
            info!("Upload finished");
            app_exit_op();
        });
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
