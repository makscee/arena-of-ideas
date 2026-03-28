use spacetimedb::{ReducerContext, Table};

use crate::{GenRequest, GenResult, GenStatus, GenTargetKind, ability, gen_request, gen_result};

/// System prompt for Claude when breeding abilities.
pub const ABILITY_SYSTEM_PROMPT: &str = r##"You are a game designer for Arena of Ideas, an auto-battler.
You create abilities using Rhai scripts. Abilities are named mechanics that units use in battle.

## Available Script Functions
- deal_damage(target_id, amount) — Deal damage to a unit
- heal_damage(target_id, amount) — Heal a unit
- steal_stat(target_id, stat_name, amount) — Steal a stat from target, give to owner
- add_shield(target_id, amount) — Add damage-absorbing shield
- change_stat(target_id, stat_name, delta) — Modify a stat (positive or negative)

## Available Variables
- X: the unit's power stat (i64), scales ability strength
- level: ability level from team synergy (1, 2, or 3) (i64)
- owner: map with keys "id", "hp", "pwr", "dmg" (all i64)
- target: map with keys "id", "hp", "pwr", "dmg" (all i64)

## Target Types (choose one for the ability)
- RandomEnemy, AllEnemies, RandomAlly, AllAllies, Owner, All

## Rules
- Scripts must ONLY use the functions and variables listed above
- Keep scripts under 15 lines
- X and level are i64, use them directly (no casting)
- Access owner/target fields with bracket notation: owner["hp"], target["id"]
- Ability should be balanced: higher damage = narrower targeting or conditional
- Name should be evocative and memorable (2-3 words max)

## Output Format
Respond with ONLY a JSON object, no markdown, no explanation outside the JSON:
{
  "name": "Ability Name",
  "description": "One sentence describing what it does",
  "target_type": "RandomEnemy",
  "effect_script": "deal_damage(target[\"id\"], X * level);"
}
"##;

/// System prompt for Claude when generating unit names and painter scripts.
pub const UNIT_SYSTEM_PROMPT: &str = r##"You are a game designer for Arena of Ideas, an auto-battler.
Generate a unique unit name and visual description based on the given abilities and trigger.

## Painter Script
The painter script uses these functions to draw the unit:
- painter.circle(radius, color_hex) — Draw a circle
- painter.rect(width, height, color_hex) — Draw a rectangle
- painter.triangle(size, color_hex) — Draw a triangle
- painter.offset(x, y) — Offset next shape
- painter.rotate(degrees) — Rotate next shape

Keep painter scripts simple: 2-5 shapes max.

## Rules
- Name should be 1-2 words, evocative, memorable
- Name must be unique (not in the existing names list)
- Visual should reflect the unit's abilities and theme

## Output Format
Respond with ONLY a JSON object:
{
  "name": "Unit Name",
  "painter_script": "painter.circle(25.0, \"#aa4444\");"
}
"##;

/// Build the full prompt for ability breeding.
pub fn build_ability_breeding_prompt(
    parent_a_name: &str,
    parent_a_description: &str,
    parent_a_script: &str,
    parent_b_name: &str,
    parent_b_description: &str,
    parent_b_script: &str,
    player_prompt: &str,
) -> String {
    format!(
        r#"Breed a new ability from these two parents:

Parent A: "{}" — {}
Script: {}

Parent B: "{}" — {}
Script: {}

Player's direction: {}

Create a new ability that meaningfully combines or evolves mechanics from both parents.
The new ability should be distinct from either parent — not just a stat change."#,
        parent_a_name,
        parent_a_description,
        parent_a_script,
        parent_b_name,
        parent_b_description,
        parent_b_script,
        player_prompt
    )
}

/// Build the full prompt for unit name/visual generation.
pub fn build_unit_generation_prompt(
    trigger: &str,
    ability_names: &[String],
    tier: u8,
    existing_names: &[String],
    player_prompt: &str,
) -> String {
    format!(
        r#"Generate a name and visual for a unit with:
- Trigger: {}
- Abilities: {}
- Tier: {}

Existing unit names (DO NOT reuse): {}

Player's description: {}"#,
        trigger,
        ability_names.join(", "),
        tier,
        existing_names.join(", "),
        player_prompt
    )
}

// ===== Reducers =====

/// Request AI to breed a new ability from two parents.
#[spacetimedb::reducer]
pub fn gen_breed_ability(
    ctx: &ReducerContext,
    parent_a_id: u64,
    parent_b_id: u64,
    prompt: String,
) -> Result<(), String> {
    if prompt.is_empty() {
        return Err("Prompt cannot be empty".to_string());
    }
    if prompt.len() > 500 {
        return Err("Prompt too long (max 500 chars)".to_string());
    }

    // Validate parents exist
    let parent_a = ctx
        .db
        .ability()
        .id()
        .find(parent_a_id)
        .ok_or_else(|| format!("Parent ability {} not found", parent_a_id))?;
    let parent_b = ctx
        .db
        .ability()
        .id()
        .find(parent_b_id)
        .ok_or_else(|| format!("Parent ability {} not found", parent_b_id))?;

    if parent_a_id == parent_b_id {
        return Err("Cannot breed an ability with itself".to_string());
    }

    ctx.db.gen_request().insert(GenRequest {
        id: 0,
        player: ctx.sender(),
        target_kind: GenTargetKind::Ability,
        prompt,
        parent_a: parent_a_id,
        parent_b: parent_b_id,
        context_id: 0,
        status: GenStatus::Pending,
        created_at: ctx.timestamp,
    });

    log::info!(
        "Ability breeding requested: {} + {}",
        parent_a.name,
        parent_b.name
    );
    Ok(())
}

/// Request AI to generate a unit name and painter script.
#[spacetimedb::reducer]
pub fn gen_create_unit(ctx: &ReducerContext, prompt: String) -> Result<(), String> {
    if prompt.is_empty() {
        return Err("Prompt cannot be empty".to_string());
    }
    if prompt.len() > 500 {
        return Err("Prompt too long (max 500 chars)".to_string());
    }

    ctx.db.gen_request().insert(GenRequest {
        id: 0,
        player: ctx.sender(),
        target_kind: GenTargetKind::Unit,
        prompt,
        parent_a: 0,
        parent_b: 0,
        context_id: 0,
        status: GenStatus::Pending,
        created_at: ctx.timestamp,
    });

    log::info!("Unit generation requested");
    Ok(())
}

/// Submit an AI generation result (called by AI procedure or admin).
#[spacetimedb::reducer]
pub fn gen_submit_result(
    ctx: &ReducerContext,
    request_id: u64,
    data: String,
    explanation: String,
) -> Result<(), String> {
    let mut request = ctx
        .db
        .gen_request()
        .id()
        .find(request_id)
        .ok_or_else(|| format!("GenRequest {} not found", request_id))?;

    if data.is_empty() {
        return Err("Result data cannot be empty".to_string());
    }

    request.status = GenStatus::Done;
    ctx.db.gen_request().id().update(request);

    ctx.db.gen_result().insert(GenResult {
        id: 0,
        request_id,
        data,
        explanation,
        created_at: ctx.timestamp,
    });

    Ok(())
}

/// Mark a generation request as failed.
#[spacetimedb::reducer]
pub fn gen_mark_failed(ctx: &ReducerContext, request_id: u64) -> Result<(), String> {
    let mut request = ctx
        .db
        .gen_request()
        .id()
        .find(request_id)
        .ok_or_else(|| format!("GenRequest {} not found", request_id))?;

    request.status = GenStatus::Failed;
    ctx.db.gen_request().id().update(request);
    Ok(())
}
