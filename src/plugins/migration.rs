use std::path::PathBuf;

use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
};

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
fn stash_save(world: &mut World) {
    info!("Stash save start");
    Stash::create().store();
    world.send_event(AppExit);
}

fn stash_upload() {
    info!("Stash upload start");
    let (arena_archive, arena_pool, users) = Stash::load().unwrap().get_data();
    migrate_data(arena_archive, arena_pool, users);
    once_on_migrate_data(|_, _, s, _, _, _| {
        info!("Upload finish with status {s:?}");
        OperationsPlugin::add(|w| {
            w.send_event(AppExit);
        });
    });
}

#[derive(Serialize, Deserialize)]
struct Stash {
    arena_archive: Vec<StashedArenaArchive>,
    arena_pool: Vec<StashedArenaPool>,
    users: Vec<StashedUser>,
}

impl Stash {
    fn get_data(self) -> (Vec<ArenaArchive>, Vec<ArenaPool>, Vec<User>) {
        let arena_archive = self
            .arena_archive
            .into_iter()
            .map(|v| v.into())
            .collect_vec();
        let arena_pool = self.arena_pool.into_iter().map(|v| v.into()).collect_vec();
        let users = self.users.into_iter().map(|v| v.into()).collect_vec();
        (arena_archive, arena_pool, users)
    }

    fn path() -> PathBuf {
        let mut path = home::home_dir().unwrap();
        path.push(HOME_DIR);
        std::fs::create_dir_all(&path).unwrap();
        path.push(ARCHIVE_FILE);
        path
    }

    fn create() -> Self {
        let arena_archive = ArenaArchive::iter().map(|v| v.into()).collect_vec();
        let arena_pool = ArenaPool::iter().map(|v| v.into()).collect_vec();
        let users = User::iter().map(|v| v.into()).collect_vec();

        Self {
            arena_archive,
            arena_pool,
            users,
        }
    }

    fn store(self) {
        let path = Self::path();

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
            season: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct StashedArenaPool {
    id: u64,
    owner: u64,
    round: u32,
    team: Vec<PackedUnit>,
}

impl From<ArenaPool> for StashedArenaPool {
    fn from(value: ArenaPool) -> Self {
        Self {
            id: value.id,
            owner: value.owner,
            round: value.round,
            team: value.team.into_iter().map(|u| u.into()).collect_vec(),
        }
    }
}

impl From<StashedArenaPool> for ArenaPool {
    fn from(value: StashedArenaPool) -> Self {
        Self {
            id: value.id,
            owner: value.owner,
            round: value.round,
            team: value.team.into_iter().map(|u| u.into()).collect_vec(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct StashedUser {
    id: u64,
    name: String,
    identities: Vec<Vec<u8>>,
    pass_hash: Option<String>,
    last_login: u64,
}

impl From<User> for StashedUser {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            name: value.name,
            identities: value
                .identities
                .into_iter()
                .map(|i| i.bytes().to_vec())
                .collect_vec(),
            pass_hash: value.pass_hash,
            last_login: value.last_login.into(),
        }
    }
}

impl From<StashedUser> for User {
    fn from(value: StashedUser) -> Self {
        Self {
            id: value.id,
            name: value.name,
            identities: value
                .identities
                .into_iter()
                .map(|i| Identity::from_bytes(i))
                .collect_vec(),
            pass_hash: value.pass_hash,
            online: false,
            last_login: value.last_login.into(),
        }
    }
}
