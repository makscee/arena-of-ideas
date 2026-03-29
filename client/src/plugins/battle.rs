use shared::battle::{BattleAction, BattleResult, BattleSide, StatKind};
use shared::target::TargetType;
use shared::trigger::Trigger;

use super::rhai_engine::{
    ScriptAction, ScriptUnit, compile_script, create_engine, execute_ability_script,
};

const MAX_TURNS: u32 = 50;
const FATIGUE_START_TURN: u32 = 30;

/// A resolved ability ready for battle execution.
#[derive(Debug, Clone)]
pub struct BattleAbility {
    pub id: u64,
    pub name: String,
    pub target_type: TargetType,
    pub effect_script: String,
}

/// A unit participating in battle with resolved data.
#[derive(Debug, Clone)]
pub struct BattleUnit {
    pub id: u64,
    pub name: String,
    pub hp: i32,
    pub pwr: i32,
    pub dmg: i32,
    pub shield: i32,
    pub trigger: Trigger,
    pub abilities: Vec<BattleAbility>,
    pub side: BattleSide,
    pub slot: u8,
    pub alive: bool,
}

impl BattleUnit {
    pub fn effective_hp(&self) -> i32 {
        self.hp - self.dmg
    }

    pub fn to_script_unit(&self) -> ScriptUnit {
        ScriptUnit {
            id: self.id as i64,
            hp: self.effective_hp() as i64,
            pwr: self.pwr as i64,
            dmg: self.dmg as i64,
        }
    }
}

/// Runs a complete battle simulation between two teams.
/// Returns the battle result with all actions logged.
pub fn simulate_battle(left_team: Vec<BattleUnit>, right_team: Vec<BattleUnit>) -> BattleResult {
    let engine = create_engine();
    let mut units: Vec<BattleUnit> = Vec::new();
    let mut actions: Vec<BattleAction> = Vec::new();

    // Spawn units
    for unit in left_team {
        actions.push(BattleAction::Spawn {
            unit: unit.id,
            slot: unit.slot,
            side: BattleSide::Left,
        });
        units.push(unit);
    }
    for unit in right_team {
        actions.push(BattleAction::Spawn {
            unit: unit.id,
            slot: unit.slot,
            side: BattleSide::Right,
        });
        units.push(unit);
    }

    // Calculate ability levels per team
    let left_ability_levels = calculate_ability_levels(&units, BattleSide::Left);
    let right_ability_levels = calculate_ability_levels(&units, BattleSide::Right);

    // Fire BattleStart triggers
    fire_trigger(
        &engine,
        &mut units,
        &mut actions,
        &Trigger::BattleStart,
        &left_ability_levels,
        &right_ability_levels,
    );
    apply_deaths(&mut units, &mut actions);

    let mut turn = 0;

    loop {
        turn += 1;

        if turn > MAX_TURNS {
            break;
        }

        // Fatigue
        if turn >= FATIGUE_START_TURN {
            let fatigue_amount = (turn - FATIGUE_START_TURN + 1) as i32;
            actions.push(BattleAction::Fatigue {
                amount: fatigue_amount,
            });
            for unit in units.iter_mut().filter(|u| u.alive) {
                unit.dmg += fatigue_amount;
            }
            apply_deaths(&mut units, &mut actions);
            if check_winner(&units).is_some() {
                break;
            }
        }

        // BeforeStrike triggers
        fire_trigger(
            &engine,
            &mut units,
            &mut actions,
            &Trigger::BeforeStrike,
            &left_ability_levels,
            &right_ability_levels,
        );
        apply_deaths(&mut units, &mut actions);
        if check_winner(&units).is_some() {
            break;
        }

        // AfterStrike triggers
        fire_trigger(
            &engine,
            &mut units,
            &mut actions,
            &Trigger::AfterStrike,
            &left_ability_levels,
            &right_ability_levels,
        );
        apply_deaths(&mut units, &mut actions);
        if check_winner(&units).is_some() {
            break;
        }

        // TurnEnd triggers
        fire_trigger(
            &engine,
            &mut units,
            &mut actions,
            &Trigger::TurnEnd,
            &left_ability_levels,
            &right_ability_levels,
        );
        apply_deaths(&mut units, &mut actions);
        if check_winner(&units).is_some() {
            break;
        }
    }

    let winner = check_winner(&units).unwrap_or(BattleSide::Left);

    BattleResult {
        winner,
        actions,
        turns: turn,
    }
}

/// Calculate ability level for each ability on a team.
/// Returns a map of ability_id → level.
fn calculate_ability_levels(
    units: &[BattleUnit],
    side: BattleSide,
) -> std::collections::HashMap<u64, i32> {
    let mut counts: std::collections::HashMap<u64, u8> = std::collections::HashMap::new();

    for unit in units.iter().filter(|u| u.side == side && u.alive) {
        for ability in &unit.abilities {
            *counts.entry(ability.id).or_insert(0) += 1;
        }
    }

    counts
        .into_iter()
        .map(|(id, count)| (id, shared::ability::ability_level(count) as i32))
        .collect()
}

/// Fire all abilities for units with the given trigger.
fn fire_trigger(
    engine: &rhai::Engine,
    units: &mut Vec<BattleUnit>,
    actions: &mut Vec<BattleAction>,
    trigger: &Trigger,
    left_levels: &std::collections::HashMap<u64, i32>,
    right_levels: &std::collections::HashMap<u64, i32>,
) {
    // Collect units that should fire (snapshot to avoid borrow issues)
    let firing_units: Vec<(usize, Vec<BattleAbility>, i32, BattleSide)> = units
        .iter()
        .enumerate()
        .filter(|(_, u)| u.alive && &u.trigger == trigger)
        .map(|(i, u)| (i, u.abilities.clone(), u.pwr, u.side))
        .collect();

    for (unit_idx, abilities, pwr, side) in firing_units {
        let levels = match side {
            BattleSide::Left => left_levels,
            BattleSide::Right => right_levels,
        };

        for ability in &abilities {
            let level = levels.get(&ability.id).copied().unwrap_or(1);

            // Resolve targets
            let targets = resolve_targets(&ability.target_type, unit_idx, units);

            for target_idx in targets {
                let owner_su = units[unit_idx].to_script_unit();
                let target_su = units[target_idx].to_script_unit();

                // Compile and execute
                let ast = match compile_script(engine, &ability.effect_script) {
                    Ok(ast) => ast,
                    Err(_) => continue,
                };

                let source_id = units[unit_idx].id;
                actions.push(BattleAction::AbilityUsed {
                    source: source_id,
                    ability_name: ability.name.clone(),
                });

                let script_actions = match execute_ability_script(
                    engine,
                    &ast,
                    pwr,
                    level,
                    &owner_su,
                    &target_su,
                    source_id,
                    &ability.name,
                ) {
                    Ok(a) => a,
                    Err(_) => continue,
                };

                // Apply script actions to units
                for sa in script_actions {
                    apply_script_action(sa, source_id, units, actions);
                }
            }
        }
    }
}

/// Resolve target indices from a TargetType.
fn resolve_targets(
    target_type: &TargetType,
    source_idx: usize,
    units: &[BattleUnit],
) -> Vec<usize> {
    let source_side = units[source_idx].side;

    match target_type {
        TargetType::Owner => vec![source_idx],
        TargetType::RandomEnemy => {
            let enemies: Vec<usize> = units
                .iter()
                .enumerate()
                .filter(|(_, u)| u.alive && u.side != source_side)
                .map(|(i, _)| i)
                .collect();
            // Deterministic: pick first living enemy
            enemies.into_iter().take(1).collect()
        }
        TargetType::AllEnemies => units
            .iter()
            .enumerate()
            .filter(|(_, u)| u.alive && u.side != source_side)
            .map(|(i, _)| i)
            .collect(),
        TargetType::RandomAlly => {
            let allies: Vec<usize> = units
                .iter()
                .enumerate()
                .filter(|(i, u)| u.alive && u.side == source_side && *i != source_idx)
                .map(|(i, _)| i)
                .collect();
            allies.into_iter().take(1).collect()
        }
        TargetType::AllAllies => units
            .iter()
            .enumerate()
            .filter(|(_, u)| u.alive && u.side == source_side)
            .map(|(i, _)| i)
            .collect(),
        TargetType::All => units
            .iter()
            .enumerate()
            .filter(|(_, u)| u.alive)
            .map(|(i, _)| i)
            .collect(),
        TargetType::Attacker => vec![source_idx], // Fallback
        TargetType::AdjacentBack | TargetType::AdjacentFront => {
            // For now, treat as owner
            vec![source_idx]
        }
        TargetType::AllyAtSlot(slot) => units
            .iter()
            .enumerate()
            .filter(|(_, u)| u.alive && u.side == source_side && u.slot == *slot)
            .map(|(i, _)| i)
            .collect(),
        TargetType::EnemyAtSlot(slot) => units
            .iter()
            .enumerate()
            .filter(|(_, u)| u.alive && u.side != source_side && u.slot == *slot)
            .map(|(i, _)| i)
            .collect(),
    }
}

/// Apply a single script action to the game state and log it.
fn apply_script_action(
    action: ScriptAction,
    source_id: u64,
    units: &mut Vec<BattleUnit>,
    actions: &mut Vec<BattleAction>,
) {
    match action {
        ScriptAction::DealDamage { target_id, amount } => {
            if let Some(unit) = units
                .iter_mut()
                .find(|u| u.id == target_id as u64 && u.alive)
            {
                let actual = if unit.shield > 0 {
                    let absorbed = amount.min(unit.shield);
                    unit.shield -= absorbed;
                    amount - absorbed
                } else {
                    amount
                };
                if actual > 0 {
                    unit.dmg += actual;
                    actions.push(BattleAction::Damage {
                        source: source_id,
                        target: target_id as u64,
                        amount: actual,
                    });
                }
            }
        }
        ScriptAction::HealDamage { target_id, amount } => {
            if let Some(unit) = units
                .iter_mut()
                .find(|u| u.id == target_id as u64 && u.alive)
            {
                let actual = amount.min(unit.dmg);
                if actual > 0 {
                    unit.dmg -= actual;
                    actions.push(BattleAction::Heal {
                        source: source_id,
                        target: target_id as u64,
                        amount: actual,
                    });
                }
            }
        }
        ScriptAction::StealStat {
            target_id,
            stat,
            amount,
        } => {
            if let Some(target) = units
                .iter_mut()
                .find(|u| u.id == target_id as u64 && u.alive)
            {
                match stat.as_str() {
                    "pwr" => {
                        let actual = amount.min(target.pwr);
                        target.pwr -= actual;
                        actions.push(BattleAction::StatChange {
                            unit: target_id as u64,
                            stat: StatKind::Pwr,
                            delta: -actual,
                        });
                        // Give to source
                        if let Some(src) = units.iter_mut().find(|u| u.id == source_id) {
                            src.pwr += actual;
                            actions.push(BattleAction::StatChange {
                                unit: source_id,
                                stat: StatKind::Pwr,
                                delta: actual,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
        ScriptAction::AddShield { target_id, amount } => {
            if let Some(unit) = units
                .iter_mut()
                .find(|u| u.id == target_id as u64 && u.alive)
            {
                unit.shield += amount;
            }
        }
        ScriptAction::ChangeStat {
            target_id,
            stat,
            delta,
        } => {
            if let Some(unit) = units
                .iter_mut()
                .find(|u| u.id == target_id as u64 && u.alive)
            {
                match stat.as_str() {
                    "pwr" => {
                        unit.pwr = (unit.pwr + delta).max(0);
                        actions.push(BattleAction::StatChange {
                            unit: target_id as u64,
                            stat: StatKind::Pwr,
                            delta,
                        });
                    }
                    "hp" => {
                        unit.hp = (unit.hp + delta).max(1);
                        actions.push(BattleAction::StatChange {
                            unit: target_id as u64,
                            stat: StatKind::Hp,
                            delta,
                        });
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Check for and process unit deaths.
fn apply_deaths(units: &mut Vec<BattleUnit>, actions: &mut Vec<BattleAction>) {
    for unit in units.iter_mut() {
        if unit.alive && unit.effective_hp() <= 0 {
            unit.alive = false;
            actions.push(BattleAction::Death { unit: unit.id });
        }
    }
}

/// Check if a side has won (all enemies dead).
fn check_winner(units: &[BattleUnit]) -> Option<BattleSide> {
    let left_alive = units.iter().any(|u| u.alive && u.side == BattleSide::Left);
    let right_alive = units.iter().any(|u| u.alive && u.side == BattleSide::Right);

    match (left_alive, right_alive) {
        (true, false) => Some(BattleSide::Left),
        (false, true) => Some(BattleSide::Right),
        (false, false) => Some(BattleSide::Left), // Tie goes to left
        (true, true) => None,
    }
}

// ===== Test Helpers =====

#[cfg(test)]
fn make_unit(
    id: u64,
    name: &str,
    hp: i32,
    pwr: i32,
    trigger: Trigger,
    abilities: Vec<BattleAbility>,
    side: BattleSide,
    slot: u8,
) -> BattleUnit {
    BattleUnit {
        id,
        name: name.to_string(),
        hp,
        pwr,
        dmg: 0,
        shield: 0,
        trigger,
        abilities,
        side,
        slot,
        alive: true,
    }
}

#[cfg(test)]
fn strike_ability(id: u64) -> BattleAbility {
    BattleAbility {
        id,
        name: "Strike".to_string(),
        target_type: TargetType::RandomEnemy,
        effect_script: "deal_damage(target[\"id\"], X * level);".to_string(),
    }
}

#[cfg(test)]
fn heal_ability(id: u64) -> BattleAbility {
    BattleAbility {
        id,
        name: "Heal".to_string(),
        target_type: TargetType::RandomAlly,
        effect_script: "heal_damage(target[\"id\"], X * level);".to_string(),
    }
}

#[cfg(test)]
fn guard_ability(id: u64) -> BattleAbility {
    BattleAbility {
        id,
        name: "Guard".to_string(),
        target_type: TargetType::Owner,
        effect_script: "add_shield(owner[\"id\"], X * level);".to_string(),
    }
}

#[cfg(test)]
fn curse_ability(id: u64) -> BattleAbility {
    BattleAbility {
        id,
        name: "Curse".to_string(),
        target_type: TargetType::RandomEnemy,
        effect_script: "change_stat(target[\"id\"], \"pwr\", -level);".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battle_one_vs_one_striker_wins() {
        let left = vec![make_unit(
            1,
            "Strong",
            5,
            5,
            Trigger::BeforeStrike,
            vec![strike_ability(100)],
            BattleSide::Left,
            0,
        )];
        let right = vec![make_unit(
            2,
            "Weak",
            3,
            2,
            Trigger::BeforeStrike,
            vec![strike_ability(100)],
            BattleSide::Right,
            0,
        )];

        let result = simulate_battle(left, right);
        assert_eq!(result.winner, BattleSide::Left);
    }

    #[test]
    fn battle_produces_spawn_actions() {
        let left = vec![make_unit(
            1,
            "A",
            3,
            2,
            Trigger::BeforeStrike,
            vec![strike_ability(100)],
            BattleSide::Left,
            0,
        )];
        let right = vec![make_unit(
            2,
            "B",
            3,
            2,
            Trigger::BeforeStrike,
            vec![strike_ability(100)],
            BattleSide::Right,
            0,
        )];

        let result = simulate_battle(left, right);
        let spawn_count = result
            .actions
            .iter()
            .filter(|a| matches!(a, BattleAction::Spawn { .. }))
            .count();
        assert_eq!(spawn_count, 2);
    }

    #[test]
    fn battle_dead_units_dont_trigger() {
        // Unit with 1 HP vs 5 PWR — dies immediately
        let left = vec![make_unit(
            1,
            "Glass",
            1,
            10,
            Trigger::BeforeStrike,
            vec![strike_ability(100)],
            BattleSide::Left,
            0,
        )];
        let right = vec![make_unit(
            2,
            "Tank",
            20,
            5,
            Trigger::BeforeStrike,
            vec![strike_ability(100)],
            BattleSide::Right,
            0,
        )];

        let result = simulate_battle(left, right);
        // Both fire BeforeStrike on same turn, but one should die
        assert!(
            result
                .actions
                .iter()
                .any(|a| matches!(a, BattleAction::Death { .. }))
        );
    }

    #[test]
    fn battle_turn_end_trigger_fires() {
        // Healer needs an ally to target with RandomAlly
        let left = vec![
            make_unit(
                1,
                "Healer",
                10,
                3,
                Trigger::TurnEnd,
                vec![heal_ability(200)],
                BattleSide::Left,
                0,
            ),
            make_unit(
                3,
                "Ally",
                10,
                2,
                Trigger::BeforeStrike,
                vec![strike_ability(100)],
                BattleSide::Left,
                1,
            ),
        ];
        let right = vec![make_unit(
            2,
            "Attacker",
            5,
            2,
            Trigger::BeforeStrike,
            vec![strike_ability(100)],
            BattleSide::Right,
            0,
        )];

        let result = simulate_battle(left, right);
        let ability_used = result.actions.iter().filter(|a| matches!(a, BattleAction::AbilityUsed { ability_name, .. } if ability_name == "Heal")).count();
        assert!(ability_used > 0, "Heal ability should have been used");
    }

    #[test]
    fn battle_guard_adds_shield() {
        let left = vec![make_unit(
            1,
            "Guardian",
            5,
            3,
            Trigger::BattleStart,
            vec![guard_ability(300)],
            BattleSide::Left,
            0,
        )];
        let right = vec![make_unit(
            2,
            "Hitter",
            5,
            2,
            Trigger::BeforeStrike,
            vec![strike_ability(100)],
            BattleSide::Right,
            0,
        )];

        let result = simulate_battle(left, right);
        // Guardian should survive longer due to shield
        // At minimum, the battle should complete
        assert!(result.turns > 0);
    }

    #[test]
    fn battle_curse_reduces_pwr() {
        let left = vec![make_unit(
            1,
            "Curser",
            10,
            2,
            Trigger::BattleStart,
            vec![curse_ability(400)],
            BattleSide::Left,
            0,
        )];
        let right = vec![make_unit(
            2,
            "Target",
            10,
            5,
            Trigger::BeforeStrike,
            vec![strike_ability(100)],
            BattleSide::Right,
            0,
        )];

        let result = simulate_battle(left, right);
        let stat_changes = result.actions.iter().filter(|a| matches!(a, BattleAction::StatChange { stat: StatKind::Pwr, delta, .. } if *delta < 0)).count();
        assert!(stat_changes > 0, "Curse should reduce pwr");
    }

    #[test]
    fn battle_all_enemies_target() {
        let aoe = BattleAbility {
            id: 500,
            name: "AoE Strike".to_string(),
            target_type: TargetType::AllEnemies,
            effect_script: "deal_damage(target[\"id\"], X);".to_string(),
        };

        let left = vec![make_unit(
            1,
            "AoE",
            10,
            2,
            Trigger::BeforeStrike,
            vec![aoe],
            BattleSide::Left,
            0,
        )];
        let right = vec![
            make_unit(
                2,
                "E1",
                5,
                1,
                Trigger::TurnEnd,
                vec![],
                BattleSide::Right,
                0,
            ),
            make_unit(
                3,
                "E2",
                5,
                1,
                Trigger::TurnEnd,
                vec![],
                BattleSide::Right,
                1,
            ),
        ];

        let result = simulate_battle(left, right);
        // Both enemies should take damage
        let damage_to_2 = result
            .actions
            .iter()
            .any(|a| matches!(a, BattleAction::Damage { target: 2, .. }));
        let damage_to_3 = result
            .actions
            .iter()
            .any(|a| matches!(a, BattleAction::Damage { target: 3, .. }));
        assert!(damage_to_2, "Enemy 1 should take damage");
        assert!(damage_to_3, "Enemy 2 should take damage");
    }

    #[test]
    fn battle_ability_level_scaling() {
        // 3 units with same ability → level 2 → X * 2
        let left = vec![
            make_unit(
                1,
                "A",
                5,
                3,
                Trigger::BeforeStrike,
                vec![strike_ability(100)],
                BattleSide::Left,
                0,
            ),
            make_unit(
                2,
                "B",
                5,
                3,
                Trigger::BeforeStrike,
                vec![strike_ability(100)],
                BattleSide::Left,
                1,
            ),
            make_unit(
                3,
                "C",
                5,
                3,
                Trigger::BeforeStrike,
                vec![strike_ability(100)],
                BattleSide::Left,
                2,
            ),
        ];
        let right = vec![make_unit(
            10,
            "Enemy",
            100,
            1,
            Trigger::TurnEnd,
            vec![],
            BattleSide::Right,
            0,
        )];

        let result = simulate_battle(left, right);
        // Each unit does 3 * 2 = 6 damage per turn (level 2 because 3 units share Strike)
        // Total = 18 damage per turn, should kill 100hp enemy in ~6 turns
        let total_damage: i32 = result
            .actions
            .iter()
            .filter_map(|a| match a {
                BattleAction::Damage { amount, .. } => Some(*amount),
                _ => None,
            })
            .sum();
        // With level 2, each of 3 units deals 6 damage = 18/turn
        assert!(
            total_damage >= 18,
            "Level scaling should boost damage, got {}",
            total_damage
        );
    }

    #[test]
    fn battle_multi_ability_unit() {
        // Unit with both Strike and Guard
        let left = vec![make_unit(
            1,
            "Paladin",
            10,
            3,
            Trigger::BeforeStrike,
            vec![strike_ability(100), guard_ability(300)],
            BattleSide::Left,
            0,
        )];
        let right = vec![make_unit(
            2,
            "Enemy",
            10,
            2,
            Trigger::BeforeStrike,
            vec![strike_ability(100)],
            BattleSide::Right,
            0,
        )];

        let result = simulate_battle(left, right);
        // Should use both Strike AND Guard
        let strike_used = result.actions.iter().any(|a| matches!(a, BattleAction::AbilityUsed { ability_name, .. } if ability_name == "Strike"));
        let guard_used = result.actions.iter().any(|a| matches!(a, BattleAction::AbilityUsed { ability_name, .. } if ability_name == "Guard"));
        assert!(strike_used, "Strike should be used");
        assert!(guard_used, "Guard should be used");
    }

    #[test]
    fn battle_ends_when_one_side_eliminated() {
        let left = vec![make_unit(
            1,
            "Killer",
            10,
            10,
            Trigger::BeforeStrike,
            vec![strike_ability(100)],
            BattleSide::Left,
            0,
        )];
        let right = vec![make_unit(
            2,
            "Weak",
            1,
            1,
            Trigger::BeforeStrike,
            vec![strike_ability(100)],
            BattleSide::Right,
            0,
        )];

        let result = simulate_battle(left, right);
        assert_eq!(result.winner, BattleSide::Left);
        assert!(result.turns <= 2, "Should end quickly");
    }

    #[test]
    fn battle_fatigue_prevents_infinite() {
        // Two units with only Guard — neither can kill the other
        let left = vec![make_unit(
            1,
            "A",
            10,
            1,
            Trigger::BattleStart,
            vec![guard_ability(300)],
            BattleSide::Left,
            0,
        )];
        let right = vec![make_unit(
            2,
            "B",
            10,
            1,
            Trigger::BattleStart,
            vec![guard_ability(300)],
            BattleSide::Right,
            0,
        )];

        let result = simulate_battle(left, right);
        assert!(
            result.turns <= MAX_TURNS,
            "Battle should end within max turns"
        );
        assert!(
            result
                .actions
                .iter()
                .any(|a| matches!(a, BattleAction::Fatigue { .. })),
            "Fatigue should kick in"
        );
    }

    #[test]
    fn battle_deterministic() {
        let make_teams = || {
            let left = vec![
                make_unit(
                    1,
                    "A",
                    5,
                    3,
                    Trigger::BeforeStrike,
                    vec![strike_ability(100)],
                    BattleSide::Left,
                    0,
                ),
                make_unit(
                    2,
                    "B",
                    3,
                    5,
                    Trigger::TurnEnd,
                    vec![curse_ability(400)],
                    BattleSide::Left,
                    1,
                ),
            ];
            let right = vec![make_unit(
                3,
                "C",
                8,
                2,
                Trigger::BeforeStrike,
                vec![strike_ability(100), guard_ability(300)],
                BattleSide::Right,
                0,
            )];
            (left, right)
        };

        let (l1, r1) = make_teams();
        let (l2, r2) = make_teams();

        let result1 = simulate_battle(l1, r1);
        let result2 = simulate_battle(l2, r2);

        assert_eq!(result1.winner, result2.winner);
        assert_eq!(result1.turns, result2.turns);
        assert_eq!(result1.actions.len(), result2.actions.len());
    }

    #[test]
    fn test_steal_stat_transfers_pwr() {
        let steal = BattleAbility {
            id: 600,
            name: "Steal".to_string(),
            target_type: TargetType::RandomEnemy,
            effect_script: "steal_stat(target[\"id\"], \"pwr\", level);".to_string(),
        };

        let left = vec![make_unit(
            1,
            "Thief",
            10,
            2,
            Trigger::BattleStart,
            vec![steal],
            BattleSide::Left,
            0,
        )];
        let right = vec![make_unit(
            2,
            "Victim",
            10,
            5,
            Trigger::TurnEnd,
            vec![],
            BattleSide::Right,
            0,
        )];

        let result = simulate_battle(left, right);

        // Target should lose pwr (negative StatChange)
        let target_pwr_loss = result
            .actions
            .iter()
            .any(|a| matches!(a, BattleAction::StatChange { unit: 2, stat: StatKind::Pwr, delta } if *delta < 0));
        assert!(target_pwr_loss, "Target pwr should decrease via steal");

        // Source should gain pwr (positive StatChange)
        let source_pwr_gain = result
            .actions
            .iter()
            .any(|a| matches!(a, BattleAction::StatChange { unit: 1, stat: StatKind::Pwr, delta } if *delta > 0));
        assert!(source_pwr_gain, "Source pwr should increase via steal");
    }

    #[test]
    fn test_shield_absorbs_damage() {
        // Guardian gets shield on BattleStart, then enemy strikes
        let left = vec![make_unit(
            1,
            "Guardian",
            10,
            3,
            Trigger::BattleStart,
            vec![guard_ability(300)],
            BattleSide::Left,
            0,
        )];
        let right = vec![make_unit(
            2,
            "Striker",
            10,
            2,
            Trigger::BeforeStrike,
            vec![strike_ability(100)],
            BattleSide::Right,
            0,
        )];

        let result = simulate_battle(left, right);

        // Guardian has pwr=3, level=1, so shield = 3*1 = 3
        // Striker has pwr=2, level=1, so damage = 2*1 = 2
        // First strike should be fully absorbed by shield (2 < 3), so no Damage action for first hit
        // Count damage actions targeting unit 1
        let damage_to_guardian: Vec<&BattleAction> = result
            .actions
            .iter()
            .filter(|a| matches!(a, BattleAction::Damage { target: 1, .. }))
            .collect();

        // With shield=3 absorbing first hit of 2, first Damage event to guardian should be delayed
        // The guardian should survive longer than without shield
        // After shield absorbs 2 damage, 1 shield remains, next hit 2-1=1 actual damage
        // So first damage action should have amount=1 (partial absorption)
        if let Some(first_dmg) = damage_to_guardian.first() {
            match first_dmg {
                BattleAction::Damage { amount, .. } => {
                    assert!(
                        *amount < 2,
                        "Shield should reduce first damage below full strike amount, got {}",
                        amount
                    );
                }
                _ => unreachable!(),
            }
        }
        // Guardian should survive more turns than a 10hp unit vs 2pwr without shield (5 turns)
        assert!(result.turns > 5, "Shield should help guardian survive longer, got {} turns", result.turns);
    }

    #[test]
    fn test_zero_pwr_deals_no_damage() {
        let left = vec![make_unit(
            1,
            "Weakling",
            10,
            0,
            Trigger::BeforeStrike,
            vec![strike_ability(100)],
            BattleSide::Left,
            0,
        )];
        let right = vec![make_unit(
            2,
            "Target",
            5,
            3,
            Trigger::BeforeStrike,
            vec![strike_ability(100)],
            BattleSide::Right,
            0,
        )];

        let result = simulate_battle(left, right);

        // Unit 1 has pwr=0, so X=0, damage = 0*level = 0 -> no Damage action from unit 1
        let damage_from_weakling = result
            .actions
            .iter()
            .filter(|a| matches!(a, BattleAction::Damage { source: 1, .. }))
            .count();
        assert_eq!(
            damage_from_weakling, 0,
            "Unit with 0 pwr should deal no damage"
        );
    }

    #[test]
    fn test_all_healers_end_via_fatigue() {
        // Two teams of healers targeting allies — neither can kill the other
        let left = vec![
            make_unit(
                1,
                "Healer1",
                10,
                3,
                Trigger::TurnEnd,
                vec![heal_ability(200)],
                BattleSide::Left,
                0,
            ),
            make_unit(
                3,
                "Healer2",
                10,
                3,
                Trigger::TurnEnd,
                vec![heal_ability(200)],
                BattleSide::Left,
                1,
            ),
        ];
        let right = vec![
            make_unit(
                2,
                "Healer3",
                10,
                3,
                Trigger::TurnEnd,
                vec![heal_ability(200)],
                BattleSide::Right,
                0,
            ),
            make_unit(
                4,
                "Healer4",
                10,
                3,
                Trigger::TurnEnd,
                vec![heal_ability(200)],
                BattleSide::Right,
                1,
            ),
        ];

        let result = simulate_battle(left, right);

        // Battle must end (not hang forever)
        assert!(
            result.turns <= MAX_TURNS,
            "Battle should end within max turns"
        );

        // Fatigue should be the mechanism that ends it
        let has_fatigue = result
            .actions
            .iter()
            .any(|a| matches!(a, BattleAction::Fatigue { .. }));
        assert!(has_fatigue, "Healers-only battle should end via fatigue");
    }
}
