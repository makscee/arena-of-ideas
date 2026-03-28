use spacetimedb::{ReducerContext, Table};

use crate::{Ability, ContentStatus, TargetType, Trigger, Unit, ability, unit};

// ===== Tier Validation =====

const BASE_BUDGET: i32 = 5;

fn tier_stat_budget(tier: u8) -> i32 {
    tier as i32 * BASE_BUDGET
}

fn tier_max_abilities(tier: u8) -> usize {
    match tier {
        1..=2 => 1,
        3..=4 => 2,
        5 => 3,
        _ => 0,
    }
}

// ===== Ability CRUD =====

#[spacetimedb::reducer]
pub fn ability_create(
    ctx: &ReducerContext,
    name: String,
    description: String,
    target_type: TargetType,
    effect_script: String,
    parent_a: u64,
    parent_b: u64,
    season: u32,
) -> Result<(), String> {
    if name.is_empty() {
        return Err("Ability name cannot be empty".to_string());
    }
    if name.len() > 100 {
        return Err("Ability name too long (max 100)".to_string());
    }
    if effect_script.is_empty() {
        return Err("Effect script cannot be empty".to_string());
    }
    if effect_script.len() > 2000 {
        return Err("Effect script too long (max 2000 chars)".to_string());
    }

    // Check name uniqueness
    for existing in ctx.db.ability().iter() {
        if existing.name == name {
            return Err(format!("Ability '{}' already exists", name));
        }
    }

    // Validate parents exist if specified
    if parent_a != 0 && ctx.db.ability().id().find(parent_a).is_none() {
        return Err(format!("Parent ability {} not found", parent_a));
    }
    if parent_b != 0 && ctx.db.ability().id().find(parent_b).is_none() {
        return Err(format!("Parent ability {} not found", parent_b));
    }
    if parent_a != 0 && parent_a == parent_b {
        return Err("Cannot breed an ability with itself".to_string());
    }

    ctx.db.ability().insert(Ability {
        id: 0,
        name: name.clone(),
        description,
        target_type,
        effect_script,
        parent_a,
        parent_b,
        rating: 0,
        status: ContentStatus::Draft,
        season,
        created_by: ctx.sender(),
        created_at: ctx.timestamp,
    });

    log::info!("Ability created: {}", name);
    Ok(())
}

#[spacetimedb::reducer]
pub fn ability_update_status(
    ctx: &ReducerContext,
    ability_id: u64,
    new_status: ContentStatus,
) -> Result<(), String> {
    let mut ability = ctx
        .db
        .ability()
        .id()
        .find(ability_id)
        .ok_or_else(|| format!("Ability {} not found", ability_id))?;

    if ability.created_by != ctx.sender() {
        return Err("Only the creator can update this ability".to_string());
    }

    ability.status = new_status;
    ctx.db.ability().id().update(ability);
    Ok(())
}

#[spacetimedb::reducer]
pub fn ability_delete(ctx: &ReducerContext, ability_id: u64) -> Result<(), String> {
    let ability = ctx
        .db
        .ability()
        .id()
        .find(ability_id)
        .ok_or_else(|| format!("Ability {} not found", ability_id))?;

    if ability.created_by != ctx.sender() {
        return Err("Only the creator can delete this ability".to_string());
    }

    ctx.db.ability().id().delete(ability_id);
    log::info!("Ability deleted: {}", ability.name);
    Ok(())
}

// ===== Unit CRUD =====

#[spacetimedb::reducer]
pub fn unit_create(
    ctx: &ReducerContext,
    name: String,
    description: String,
    hp: i32,
    pwr: i32,
    tier: u8,
    trigger: Trigger,
    abilities: Vec<u64>,
    painter_script: String,
) -> Result<(), String> {
    if name.is_empty() {
        return Err("Unit name cannot be empty".to_string());
    }
    if name.len() > 100 {
        return Err("Unit name too long (max 100)".to_string());
    }

    // Check name uniqueness
    for existing in ctx.db.unit().iter() {
        if existing.name == name {
            return Err(format!("Unit '{}' already exists", name));
        }
    }

    // Validate tier
    if !(1..=5).contains(&tier) {
        return Err(format!("Invalid tier {} (must be 1-5)", tier));
    }

    // Validate stats
    if hp <= 0 || pwr <= 0 {
        return Err("HP and PWR must be positive".to_string());
    }
    let budget = tier_stat_budget(tier);
    if hp + pwr > budget {
        return Err(format!(
            "Stats {}hp + {}pwr = {} exceeds tier {} budget of {}",
            hp,
            pwr,
            hp + pwr,
            tier,
            budget
        ));
    }

    // Validate ability count
    let max = tier_max_abilities(tier);
    if abilities.is_empty() || abilities.len() > max {
        return Err(format!(
            "Tier {} units need 1-{} abilities, got {}",
            tier,
            max,
            abilities.len()
        ));
    }

    // Validate all abilities exist
    for &ability_id in &abilities {
        if ctx.db.ability().id().find(ability_id).is_none() {
            return Err(format!("Ability {} not found", ability_id));
        }
    }

    ctx.db.unit().insert(Unit {
        id: 0,
        name: name.clone(),
        description,
        hp,
        pwr,
        tier,
        trigger,
        abilities,
        painter_script,
        rating: 0,
        status: ContentStatus::Draft,
        created_by: ctx.sender(),
        created_at: ctx.timestamp,
    });

    log::info!("Unit created: {}", name);
    Ok(())
}

#[spacetimedb::reducer]
pub fn unit_update_status(
    ctx: &ReducerContext,
    unit_id: u64,
    new_status: ContentStatus,
) -> Result<(), String> {
    let mut unit = ctx
        .db
        .unit()
        .id()
        .find(unit_id)
        .ok_or_else(|| format!("Unit {} not found", unit_id))?;

    if unit.created_by != ctx.sender() {
        return Err("Only the creator can update this unit".to_string());
    }

    unit.status = new_status;
    ctx.db.unit().id().update(unit);
    Ok(())
}

#[spacetimedb::reducer]
pub fn unit_delete(ctx: &ReducerContext, unit_id: u64) -> Result<(), String> {
    let unit = ctx
        .db
        .unit()
        .id()
        .find(unit_id)
        .ok_or_else(|| format!("Unit {} not found", unit_id))?;

    if unit.created_by != ctx.sender() {
        return Err("Only the creator can delete this unit".to_string());
    }

    ctx.db.unit().id().delete(unit_id);
    log::info!("Unit deleted: {}", unit.name);
    Ok(())
}
