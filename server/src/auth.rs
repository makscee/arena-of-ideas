use spacetimedb::{ReducerContext, Table};

use crate::{player, Player};

#[spacetimedb::reducer]
pub fn register(ctx: &ReducerContext, name: String) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if name.len() > 50 {
        return Err("Name too long (max 50 characters)".to_string());
    }

    // Check if player already registered
    if ctx.db.player().identity().find(ctx.sender()).is_some() {
        return Err("Player already registered".to_string());
    }

    ctx.db.player().insert(Player {
        identity: ctx.sender(),
        name,
        created_at: ctx.timestamp,
    });

    log::info!("Player registered: {:?}", ctx.sender());
    Ok(())
}

#[spacetimedb::reducer]
pub fn login_by_identity(ctx: &ReducerContext) -> Result<(), String> {
    match ctx.db.player().identity().find(ctx.sender()) {
        Some(player) => {
            log::info!("Player logged in: {}", player.name);
            Ok(())
        }
        None => Err("Player not registered".to_string()),
    }
}
