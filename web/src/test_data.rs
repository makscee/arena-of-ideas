use shared::battle::{BattleAction, BattleResult, BattleSide, StatKind};
use shared::content_status::ContentStatus;
use shared::tier::Tier;
use shared::trigger::Trigger;
use shared::unit::Unit;
use std::collections::HashMap;

pub fn sample_units() -> HashMap<u64, Unit> {
    let mut units = HashMap::new();

    let make_unit = |id, name: &str, hp, pwr, trigger| Unit {
        id,
        name: name.to_string(),
        description: format!("A mighty {name}"),
        hp,
        pwr,
        tier: Tier::new(2).unwrap(),
        trigger,
        abilities: vec![],
        painter_script: String::new(),
        rating: 0,
        status: ContentStatus::Active,
        is_fused: false,
    };

    // Left team (ally)
    units.insert(1, make_unit(1, "Fire Golem", 8, 4, Trigger::BeforeStrike));
    units.insert(2, make_unit(2, "Shadow Mage", 5, 6, Trigger::AfterStrike));
    units.insert(3, make_unit(3, "Iron Knight", 10, 3, Trigger::DamageTaken));

    // Right team (enemy)
    units.insert(4, make_unit(4, "Ice Sprite", 6, 5, Trigger::BattleStart));
    units.insert(5, make_unit(5, "Storm Hawk", 4, 7, Trigger::BeforeStrike));
    units.insert(6, make_unit(6, "Earth Titan", 12, 2, Trigger::TurnEnd));

    units
}

pub fn sample_battle() -> BattleResult {
    use BattleAction::*;

    BattleResult {
        winner: BattleSide::Left,
        turns: 4,
        actions: vec![
            // Spawns
            Spawn {
                unit: 1,
                slot: 0,
                side: BattleSide::Left,
            },
            Spawn {
                unit: 2,
                slot: 1,
                side: BattleSide::Left,
            },
            Spawn {
                unit: 3,
                slot: 2,
                side: BattleSide::Left,
            },
            Spawn {
                unit: 4,
                slot: 0,
                side: BattleSide::Right,
            },
            Spawn {
                unit: 5,
                slot: 1,
                side: BattleSide::Right,
            },
            Spawn {
                unit: 6,
                slot: 2,
                side: BattleSide::Right,
            },
            // Turn 1
            AbilityUsed {
                source: 4,
                ability_name: "Frost Nova".into(),
            },
            Damage {
                source: 4,
                target: 1,
                amount: 3,
            },
            Wait { seconds: 0.3 },
            AbilityUsed {
                source: 1,
                ability_name: "Flame Burst".into(),
            },
            Damage {
                source: 1,
                target: 4,
                amount: 4,
            },
            Wait { seconds: 0.3 },
            // Strike phase
            Damage {
                source: 1,
                target: 4,
                amount: 4,
            },
            StatChange {
                unit: 1,
                stat: StatKind::Pwr,
                delta: 1,
            },
            Damage {
                source: 4,
                target: 1,
                amount: 5,
            },
            Wait { seconds: 0.5 },
            // Turn 2
            AbilityUsed {
                source: 2,
                ability_name: "Shadow Bolt".into(),
            },
            Damage {
                source: 2,
                target: 5,
                amount: 6,
            },
            Heal {
                source: 2,
                target: 2,
                amount: 2,
            },
            Wait { seconds: 0.3 },
            Damage {
                source: 5,
                target: 2,
                amount: 7,
            },
            Damage {
                source: 2,
                target: 5,
                amount: 6,
            },
            Death { unit: 5 },
            Wait { seconds: 0.5 },
            // Turn 3
            Damage {
                source: 1,
                target: 6,
                amount: 5,
            },
            AbilityUsed {
                source: 6,
                ability_name: "Earthquake".into(),
            },
            Damage {
                source: 6,
                target: 1,
                amount: 2,
            },
            Damage {
                source: 6,
                target: 2,
                amount: 2,
            },
            Damage {
                source: 6,
                target: 3,
                amount: 2,
            },
            Wait { seconds: 0.3 },
            Damage {
                source: 3,
                target: 6,
                amount: 3,
            },
            Death { unit: 4 },
            Wait { seconds: 0.5 },
            // Turn 4
            Fatigue { amount: 1 },
            Damage {
                source: 1,
                target: 6,
                amount: 5,
            },
            Damage {
                source: 3,
                target: 6,
                amount: 3,
            },
            Death { unit: 6 },
        ],
    }
}
