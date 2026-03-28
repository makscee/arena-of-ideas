use spacetimedb::{ReducerContext, Table};

use crate::{Ability, ContentStatus, Season, ability, season, unit};

const ACTIVE_ABILITY_COUNT: usize = 20;
const ACTIVE_UNIT_COUNT: usize = 100;
const MIN_UNITS_PER_ABILITY: usize = 5;
const RATING_DECAY_AMOUNT: i32 = 1;

/// Start a new season: select top abilities, activate eligible units.
#[spacetimedb::reducer]
pub fn season_start(ctx: &ReducerContext) -> Result<(), String> {
    // Count units per ability
    let mut ability_unit_counts: std::collections::HashMap<u64, usize> =
        std::collections::HashMap::new();
    for unit in ctx.db.unit().iter() {
        if unit.status == ContentStatus::Active || unit.status == ContentStatus::Incubator {
            for &ability_id in &unit.abilities {
                *ability_unit_counts.entry(ability_id).or_insert(0) += 1;
            }
        }
    }

    // Filter abilities with enough units and sort by rating
    let mut qualifying_abilities: Vec<Ability> = ctx
        .db
        .ability()
        .iter()
        .filter(|a| {
            let count = ability_unit_counts.get(&a.id).copied().unwrap_or(0);
            count >= MIN_UNITS_PER_ABILITY
                && (a.status == ContentStatus::Active || a.status == ContentStatus::Incubator)
        })
        .collect();

    qualifying_abilities.sort_by(|a, b| b.rating.cmp(&a.rating));
    qualifying_abilities.truncate(ACTIVE_ABILITY_COUNT);

    let active_ability_ids: Vec<u64> = qualifying_abilities.iter().map(|a| a.id).collect();

    // Activate selected abilities, retire others
    for mut ability in ctx.db.ability().iter() {
        let new_status = if active_ability_ids.contains(&ability.id) {
            ContentStatus::Active
        } else if ability.status == ContentStatus::Active {
            ContentStatus::Retired
        } else {
            continue;
        };

        if ability.status != new_status {
            ability.status = new_status;
            ctx.db.ability().id().update(ability);
        }
    }

    // Activate top units that use active abilities
    let mut eligible_units: Vec<crate::Unit> = ctx
        .db
        .unit()
        .iter()
        .filter(|u| {
            (u.status == ContentStatus::Active || u.status == ContentStatus::Incubator)
                && u.abilities
                    .iter()
                    .all(|aid| active_ability_ids.contains(aid))
        })
        .collect();

    eligible_units.sort_by(|a, b| b.rating.cmp(&a.rating));

    let active_unit_ids: Vec<u64> = eligible_units
        .iter()
        .take(ACTIVE_UNIT_COUNT)
        .map(|u| u.id)
        .collect();

    for mut unit in ctx.db.unit().iter() {
        let new_status = if active_unit_ids.contains(&unit.id) {
            ContentStatus::Active
        } else if unit.status == ContentStatus::Active {
            ContentStatus::Retired
        } else {
            continue;
        };

        if unit.status != new_status {
            unit.status = new_status;
            ctx.db.unit().id().update(unit);
        }
    }

    // Record the season
    ctx.db.season().insert(Season {
        id: 0,
        active_ability_ids,
        created_at: ctx.timestamp,
    });

    log::info!("New season started");
    Ok(())
}

/// Apply rating decay to all active units (call periodically).
#[spacetimedb::reducer]
pub fn apply_rating_decay(ctx: &ReducerContext) -> Result<(), String> {
    for mut unit in ctx.db.unit().iter() {
        if unit.status == ContentStatus::Active && unit.rating > 0 {
            unit.rating = (unit.rating - RATING_DECAY_AMOUNT).max(0);
            ctx.db.unit().id().update(unit);
        }
    }
    log::info!("Rating decay applied");
    Ok(())
}

/// Promote a unit from Incubator to Active (if space available).
#[spacetimedb::reducer]
pub fn promote_unit(ctx: &ReducerContext, unit_id: u64) -> Result<(), String> {
    let mut unit = ctx
        .db
        .unit()
        .id()
        .find(unit_id)
        .ok_or_else(|| format!("Unit {} not found", unit_id))?;

    if unit.status != ContentStatus::Incubator {
        return Err("Unit must be in Incubator to promote".to_string());
    }

    let active_count = ctx
        .db
        .unit()
        .iter()
        .filter(|u| u.status == ContentStatus::Active)
        .count();

    if active_count >= ACTIVE_UNIT_COUNT {
        // Demote lowest rated active unit
        let mut lowest: Option<crate::Unit> = None;
        for u in ctx.db.unit().iter() {
            if u.status == ContentStatus::Active {
                if lowest.as_ref().is_none_or(|l| u.rating < l.rating) {
                    lowest = Some(u);
                }
            }
        }
        if let Some(mut low) = lowest {
            low.status = ContentStatus::Retired;
            ctx.db.unit().id().update(low);
        }
    }

    unit.status = ContentStatus::Active;
    ctx.db.unit().id().update(unit);
    Ok(())
}
