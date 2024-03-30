use spacetimedb_sdk::spacetimedb_lib::bsatn;

use crate::module_bindings::{once_on_upload_pool, upload_pool, ArenaPool};

use super::*;

pub struct ArenaArchivePlugin;

impl Plugin for ArenaArchivePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::ArenaArchiveSave), archive_save)
            .add_systems(OnEnter(GameState::ArenaArchiveUpload), archive_upload);
    }
}

const ARCHIVE_FILE: &str = "arena_archive";
fn archive_save() {
    info!("Arena archive save start");
    let pool = ArenaPool::iter().collect_vec();
    info!("Got {} teams", pool.len());
    let pool = bsatn::to_vec(&pool).unwrap();

    let mut path = home::home_dir().unwrap();
    path.push(HOME_DIR);
    std::fs::create_dir_all(&path).unwrap();

    path.push(ARCHIVE_FILE);
    std::fs::write(&path, pool).unwrap();
    OperationsPlugin::add(|w| w.send_event(AppExit));
}

fn archive_upload() {
    info!("Arena archive upload start");
    let mut path = home::home_dir().unwrap();
    path.push(HOME_DIR);
    path.push(ARCHIVE_FILE);

    match std::fs::read(&path) {
        Err(e) => panic!("{e}"),
        Ok(file_contents) => {
            let pool = bsatn::from_slice::<Vec<ArenaPool>>(&file_contents).unwrap();
            info!("{} teams loaded", pool.len());
            upload_pool(pool);
            once_on_upload_pool(|_, _, s, _| {
                debug!("{s:?}");
                OperationsPlugin::add(|w| w.send_event(AppExit));
            });
        }
    }
}
