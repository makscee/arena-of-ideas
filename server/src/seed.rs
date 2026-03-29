use spacetimedb::{ReducerContext, Table};

use crate::{Ability, ArenaState, ContentStatus, TargetType, Trigger, Unit, ability, arena_state, unit};

/// Seeds the database with primordial abilities and sample units.
pub fn seed_primordial_abilities(ctx: &ReducerContext) {
    // Only seed if empty
    if ctx.db.ability().iter().count() > 0 {
        return;
    }

    let now = ctx.timestamp;

    // ===== Primordial Abilities =====

    let strike = ctx.db.ability().insert(Ability {
        id: 0,
        name: "Strike".to_string(),
        description: "Deals damage to a random enemy".to_string(),
        target_type: TargetType::RandomEnemy,
        effect_script: "ability_actions.deal_damage(target, X * level);".to_string(),
        parent_a: 0,
        parent_b: 0,
        rating: 0,
        status: ContentStatus::Active,
        season: 0,
        created_by: ctx.sender(),
        created_at: now,
    });

    let guard = ctx.db.ability().insert(Ability {
        id: 0,
        name: "Guard".to_string(),
        description: "Grants a shield to self, reducing incoming damage".to_string(),
        target_type: TargetType::Owner,
        effect_script: "ability_actions.add_shield(owner, X * level);".to_string(),
        parent_a: 0,
        parent_b: 0,
        rating: 0,
        status: ContentStatus::Active,
        season: 0,
        created_by: ctx.sender(),
        created_at: now,
    });

    let heal = ctx.db.ability().insert(Ability {
        id: 0,
        name: "Heal".to_string(),
        description: "Restores health to a random ally".to_string(),
        target_type: TargetType::RandomAlly,
        effect_script: "ability_actions.heal_damage(target, X * level);".to_string(),
        parent_a: 0,
        parent_b: 0,
        rating: 0,
        status: ContentStatus::Active,
        season: 0,
        created_by: ctx.sender(),
        created_at: now,
    });

    let curse = ctx.db.ability().insert(Ability {
        id: 0,
        name: "Curse".to_string(),
        description: "Reduces a random enemy's power".to_string(),
        target_type: TargetType::RandomEnemy,
        effect_script: "ability_actions.change_stat(target, \"pwr\", -level);".to_string(),
        parent_a: 0,
        parent_b: 0,
        rating: 0,
        status: ContentStatus::Active,
        season: 0,
        created_by: ctx.sender(),
        created_at: now,
    });

    log::info!(
        "Seeded primordial abilities: Strike({}), Guard({}), Heal({}), Curse({})",
        strike.id,
        guard.id,
        heal.id,
        curse.id
    );

    // ===== Sample Units =====

    // Tier 1 units (1 ability each, budget = 5)
    ctx.db.unit().insert(Unit {
        id: 0,
        name: "Footsoldier".to_string(),
        description: "A basic melee fighter".to_string(),
        hp: 3,
        pwr: 2,
        tier: 1,
        trigger: Trigger::BeforeStrike,
        abilities: vec![strike.id],
        painter_script: "painter.circle(25.0, \"#aa4444\");".to_string(),
        rating: 0,
        status: ContentStatus::Active,
        created_by: ctx.sender(),
        created_at: now,
    });

    ctx.db.unit().insert(Unit {
        id: 0,
        name: "Shieldbearer".to_string(),
        description: "A defensive unit that protects itself".to_string(),
        hp: 4,
        pwr: 1,
        tier: 1,
        trigger: Trigger::BattleStart,
        abilities: vec![guard.id],
        painter_script: "painter.circle(25.0, \"#4444aa\");".to_string(),
        rating: 0,
        status: ContentStatus::Active,
        created_by: ctx.sender(),
        created_at: now,
    });

    ctx.db.unit().insert(Unit {
        id: 0,
        name: "Medic".to_string(),
        description: "Heals allies at the end of each turn".to_string(),
        hp: 2,
        pwr: 3,
        tier: 1,
        trigger: Trigger::TurnEnd,
        abilities: vec![heal.id],
        painter_script: "painter.circle(25.0, \"#44aa44\");".to_string(),
        rating: 0,
        status: ContentStatus::Active,
        created_by: ctx.sender(),
        created_at: now,
    });

    ctx.db.unit().insert(Unit {
        id: 0,
        name: "Hexer".to_string(),
        description: "Curses enemies, weakening their attacks".to_string(),
        hp: 2,
        pwr: 3,
        tier: 1,
        trigger: Trigger::BattleStart,
        abilities: vec![curse.id],
        painter_script: "painter.circle(25.0, \"#aa44aa\");".to_string(),
        rating: 0,
        status: ContentStatus::Active,
        created_by: ctx.sender(),
        created_at: now,
    });

    // Tier 2 units (1 ability, budget = 10)
    ctx.db.unit().insert(Unit {
        id: 0,
        name: "Knight".to_string(),
        description: "A powerful striker with high health".to_string(),
        hp: 6,
        pwr: 4,
        tier: 2,
        trigger: Trigger::BeforeStrike,
        abilities: vec![strike.id],
        painter_script: "painter.circle(30.0, \"#cc6644\");".to_string(),
        rating: 0,
        status: ContentStatus::Active,
        created_by: ctx.sender(),
        created_at: now,
    });

    ctx.db.unit().insert(Unit {
        id: 0,
        name: "Priest".to_string(),
        description: "A strong healer who restores allies after damage".to_string(),
        hp: 5,
        pwr: 5,
        tier: 2,
        trigger: Trigger::DamageTaken,
        abilities: vec![heal.id],
        painter_script: "painter.circle(30.0, \"#66cc66\");".to_string(),
        rating: 0,
        status: ContentStatus::Active,
        created_by: ctx.sender(),
        created_at: now,
    });

    // Tier 3 units (2 abilities, budget = 15)
    ctx.db.unit().insert(Unit {
        id: 0,
        name: "Paladin".to_string(),
        description: "A holy warrior that strikes and guards".to_string(),
        hp: 8,
        pwr: 6,
        tier: 3,
        trigger: Trigger::BeforeStrike,
        abilities: vec![strike.id, guard.id],
        painter_script: "painter.circle(35.0, \"#cccc44\");".to_string(),
        rating: 0,
        status: ContentStatus::Active,
        created_by: ctx.sender(),
        created_at: now,
    });

    ctx.db.unit().insert(Unit {
        id: 0,
        name: "Warlock".to_string(),
        description: "A dark caster that strikes and curses".to_string(),
        hp: 7,
        pwr: 7,
        tier: 3,
        trigger: Trigger::TurnEnd,
        abilities: vec![strike.id, curse.id],
        painter_script: "painter.circle(35.0, \"#8844cc\");".to_string(),
        rating: 0,
        status: ContentStatus::Active,
        created_by: ctx.sender(),
        created_at: now,
    });

    log::info!("Seeded 8 sample units across tiers 1-3");

    // Initialize arena state
    ctx.db.arena_state().insert(ArenaState {
        always_zero: 0,
        last_floor: 1,
    });
    log::info!("Arena state initialized with last_floor=1");
}
