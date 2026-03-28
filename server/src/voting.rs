use spacetimedb::{ReducerContext, Table};

use crate::{Vote, ability, unit, vote};

#[spacetimedb::reducer]
pub fn vote_cast(
    ctx: &ReducerContext,
    entity_kind: String,
    entity_id: u64,
    value: i8,
) -> Result<(), String> {
    // Validate entity kind
    if entity_kind != "ability" && entity_kind != "unit" {
        return Err("entity_kind must be 'ability' or 'unit'".to_string());
    }

    // Validate value
    if value != 1 && value != -1 {
        return Err("Vote value must be 1 or -1".to_string());
    }

    // Validate entity exists
    match entity_kind.as_str() {
        "ability" => {
            if ctx.db.ability().id().find(entity_id).is_none() {
                return Err(format!("Ability {} not found", entity_id));
            }
        }
        "unit" => {
            if ctx.db.unit().id().find(entity_id).is_none() {
                return Err(format!("Unit {} not found", entity_id));
            }
        }
        _ => unreachable!(),
    }

    // Check for existing vote by this player on this entity
    for existing in ctx.db.vote().iter() {
        if existing.player == ctx.sender()
            && existing.entity_kind == entity_kind
            && existing.entity_id == entity_id
        {
            // Update existing vote
            let old_value = existing.value;
            let delta = value - old_value;

            // Update vote record
            let mut updated = existing;
            updated.value = value;
            updated.created_at = ctx.timestamp;
            ctx.db.vote().id().update(updated);

            // Update entity rating
            update_rating(ctx, &entity_kind, entity_id, delta as i32);
            return Ok(());
        }
    }

    // New vote
    ctx.db.vote().insert(Vote {
        id: 0,
        player: ctx.sender(),
        entity_kind: entity_kind.clone(),
        entity_id,
        value,
        created_at: ctx.timestamp,
    });

    // Update entity rating
    update_rating(ctx, &entity_kind, entity_id, value as i32);
    Ok(())
}

#[spacetimedb::reducer]
pub fn vote_retract(
    ctx: &ReducerContext,
    entity_kind: String,
    entity_id: u64,
) -> Result<(), String> {
    for existing in ctx.db.vote().iter() {
        if existing.player == ctx.sender()
            && existing.entity_kind == entity_kind
            && existing.entity_id == entity_id
        {
            let old_value = existing.value;
            ctx.db.vote().id().delete(existing.id);
            update_rating(ctx, &entity_kind, entity_id, -(old_value as i32));
            return Ok(());
        }
    }
    Err("No vote found to retract".to_string())
}

fn update_rating(ctx: &ReducerContext, entity_kind: &str, entity_id: u64, delta: i32) {
    match entity_kind {
        "ability" => {
            if let Some(mut ability) = ctx.db.ability().id().find(entity_id) {
                ability.rating += delta;
                ctx.db.ability().id().update(ability);
            }
        }
        "unit" => {
            if let Some(mut unit) = ctx.db.unit().id().find(entity_id) {
                unit.rating += delta;
                ctx.db.unit().id().update(unit);
            }
        }
        _ => {}
    }
}
