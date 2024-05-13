use std::path::PathBuf;

use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
};
use spacetimedb_sdk::spacetimedb_lib::bsatn;

use crate::module_bindings::*;

use super::*;

pub struct MigrationPlugin;

impl Plugin for MigrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MigrationSave), stash_save)
            .add_systems(OnEnter(GameState::MigrationUpload), stash_upload);
    }
}

const ARCHIVE_FILE: &str = "migration_archive";
fn stash_save() {
    info!("Arena stash save start");
    let pool = ArenaPool::iter().collect_vec();
    info!("Got {} teams", pool.len());
    let pool = bsatn::to_vec(&pool).unwrap();

    let mut path = home::home_dir().unwrap();
    path.push(HOME_DIR);
    std::fs::create_dir_all(&path).unwrap();

    path.push(ARCHIVE_FILE);
    std::fs::write(&path, pool).unwrap();
    OperationsPlugin::add(|w| {
        w.send_event(AppExit);
    });
}

fn stash_upload() {
    info!("Arena stash upload start");
    let mut path = home::home_dir().unwrap();
    path.push(HOME_DIR);
    path.push(ARCHIVE_FILE);

    match std::fs::read(&path) {
        Err(e) => panic!("{e}"),
        Ok(file_contents) => {
            let pool = bsatn::from_slice::<Vec<ArenaPool>>(&file_contents).unwrap();
            info!("{} teams uploaded", pool.len());
            upload_pool(pool);
            once_on_upload_pool(|_, _, s, _| {
                debug!("{s:?}");
                OperationsPlugin::add(|w| {
                    w.send_event(AppExit);
                });
            });
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Stash {
    arena_archive: Vec<StashedArenaArchive>,
}

impl Stash {
    fn path() -> PathBuf {
        let mut path = home::home_dir().unwrap();
        path.push(HOME_DIR);
        path.push(ARCHIVE_FILE);
        path
    }

    fn create() -> Self {
        let arena_archive = ArenaArchive::iter().map(|v| v.into()).collect_vec();

        Self { arena_archive }
    }

    fn store(self) {
        let path = Self::path();
        std::fs::create_dir_all(&path).unwrap();
        match std::fs::write(
            path,
            to_string_pretty(
                &self,
                PrettyConfig::new()
                    .extensions(Extensions::IMPLICIT_SOME)
                    .compact_arrays(true),
            )
            .unwrap(),
        ) {
            Ok(_) => {
                info!("Store successful")
            }
            Err(e) => {
                error!("Store error: {e}")
            }
        }
    }

    fn load() -> Result<Self> {
        let path = Self::path();
        ron::from_str(&std::fs::read_to_string(&path)?).map_err(|e| anyhow!(e.to_string()))
    }
}

#[derive(Serialize, Deserialize)]
struct StashedArenaArchive {
    id: u64,
    user_id: u64,
    round: u32,
    wins: u32,
    loses: u32,
    team: Vec<PackedUnit>,
    timestamp: u64,
}

impl From<ArenaArchive> for StashedArenaArchive {
    fn from(value: ArenaArchive) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            round: value.round,
            wins: value.wins,
            loses: value.loses,
            team: value.team.into_iter().map(|u| u.into()).collect_vec(),
            timestamp: value.timestamp.into(),
        }
    }
}

impl From<StashedArenaArchive> for ArenaArchive {
    fn from(value: StashedArenaArchive) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            round: value.round,
            wins: value.wins,
            loses: value.loses,
            team: value.team.into_iter().map(|u| u.into()).collect_vec(),
            timestamp: value.timestamp.into(),
        }
    }
}
