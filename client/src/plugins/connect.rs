use bevy::prelude::*;
use spacetimedb_sdk::{DbContext, Table};

use crate::module_bindings::*;
use crate::plugins::collection::{AbilityData, GameContent, UnitData};
use crate::resources::game_state::GameState;

pub struct ConnectPlugin;

impl Plugin for ConnectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StdbConnection>()
            .add_systems(OnEnter(GameState::Login), start_connection)
            .add_systems(
                Update,
                (tick_connection, check_connected).run_if(in_state(GameState::Login)),
            )
            .add_systems(Update, tick_connection.run_if(in_state(GameState::Home)))
            .add_systems(Update, tick_connection.run_if(in_state(GameState::Shop)))
            .add_systems(Update, tick_connection.run_if(in_state(GameState::Create)))
            .add_systems(Update, tick_connection.run_if(in_state(GameState::Incubator)));
    }
}

/// Holds the SpacetimeDB connection as a Bevy resource.
#[derive(Resource, Default)]
pub struct StdbConnection {
    pub conn: Option<DbConnection>,
    pub connected: bool,
    pub token: Option<String>,
    pub error: Option<String>,
}

const DEFAULT_HOST: &str = "http://127.0.0.1:3000";
const DEFAULT_DB: &str = "aoi-test";

fn start_connection(mut stdb: ResMut<StdbConnection>) {
    if stdb.conn.is_some() {
        return;
    }

    // Read host/db from environment or use defaults
    let host = std::env::var("STDB_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
    let db_name = std::env::var("STDB_DB").unwrap_or_else(|_| DEFAULT_DB.to_string());

    info!("Connecting to SpacetimeDB at {} db={}", host, db_name);

    let mut builder = DbConnection::builder()
        .with_uri(&host)
        .with_database_name(&db_name)
        .on_connect(|_conn, identity, token| {
            info!("Connected to SpacetimeDB as {:?}", identity);
            let _ = token; // Token saved via the resource in check_connected
        })
        .on_disconnect(|_ctx, err| {
            if let Some(err) = err {
                warn!("Disconnected from SpacetimeDB: {:?}", err);
            }
        });

    // Restore saved token if available
    if let Some(ref token) = stdb.token {
        builder = builder.with_token(Some(token.clone()));
    }

    match builder.build() {
        Ok(conn) => {
            // Subscribe to all public tables
            conn.subscription_builder()
                .subscribe(vec![
                    "SELECT * FROM ability",
                    "SELECT * FROM unit",
                    "SELECT * FROM player",
                    "SELECT * FROM game_match",
                    "SELECT * FROM vote",
                    "SELECT * FROM floor_boss",
                    "SELECT * FROM arena_state",
                    "SELECT * FROM global_settings",
                    "SELECT * FROM season",
                    "SELECT * FROM feature_request",
                    "SELECT * FROM gen_request",
                    "SELECT * FROM gen_result",
                    "SELECT * FROM floor_pool_team",
                ]);

            // Start processing messages in a background thread
            conn.run_threaded();

            stdb.conn = Some(conn);
            info!("SpacetimeDB connection initiated");
        }
        Err(e) => {
            error!("Failed to connect to SpacetimeDB: {:?}", e);
            stdb.error = Some(format!("{:?}", e));
        }
    }
}

fn tick_connection(stdb: Res<StdbConnection>) {
    if let Some(ref conn) = stdb.conn {
        // frame_tick is not needed when using run_threaded, but doesn't hurt
        let _ = conn.frame_tick();
    }
}

fn check_connected(
    stdb: Res<StdbConnection>,
    mut next_state: ResMut<NextState<GameState>>,
    mut content: ResMut<GameContent>,
) {
    // If connection failed, show error and allow retry
    if let Some(ref err) = stdb.error {
        warn!("Connection error: {}", err);
        // Fall through to Home with mock data
        next_state.set(GameState::Home);
        return;
    }

    let Some(ref conn) = stdb.conn else { return };

    // Check if we have data by seeing if abilities are loaded
    let ability_count = conn.db.ability().count();
    if ability_count == 0 {
        return; // Still loading
    }

    // Sync data from SpacetimeDB to GameContent resource
    content.abilities = conn
        .db
        .ability()
        .iter()
        .map(|a| AbilityData {
            id: a.id,
            name: a.name.clone(),
            description: a.description.clone(),
            target_type: format!("{:?}", a.target_type),
            effect_script: a.effect_script.clone(),
            parent_a: a.parent_a,
            parent_b: a.parent_b,
            rating: a.rating,
            status: format!("{:?}", a.status),
        })
        .collect();

    content.units = conn
        .db
        .unit()
        .iter()
        .map(|u| {
            let ability_names: Vec<String> = u
                .abilities
                .iter()
                .filter_map(|aid| conn.db.ability().id().find(aid).map(|a| a.name.clone()))
                .collect();
            UnitData {
                id: u.id,
                name: u.name.clone(),
                description: u.description.clone(),
                hp: u.hp,
                pwr: u.pwr,
                tier: u.tier,
                trigger: format!("{:?}", u.trigger),
                ability_names,
                rating: u.rating,
                status: format!("{:?}", u.status),
            }
        })
        .collect();

    info!(
        "Loaded {} abilities and {} units from SpacetimeDB",
        content.abilities.len(),
        content.units.len()
    );

    next_state.set(GameState::Home);
}
