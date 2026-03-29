use bevy::prelude::*;

use shared::battle::BattleSide;
use shared::target::TargetType;
use shared::trigger::Trigger;

use crate::plugins::battle::{BattleAbility, BattleUnit, simulate_battle};
use crate::plugins::battle_scene::{BattleSceneState, BattleUnitVisual};
use crate::resources::game_state::GameState;

pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        // Run on first Update in Title state so egui has initialized
        app.add_systems(Update, check_demo_mode.run_if(in_state(GameState::Title)));
    }
}

/// Check for --demo CLI flag and launch a battle scene if present.
fn check_demo_mode(
    mut scene: ResMut<BattleSceneState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let args: Vec<String> = std::env::args().collect();
    if !args.iter().any(|a| a == "--demo" || a == "--battle") {
        return;
    }

    info!("Demo mode: launching battle scene");

    // Build two teams
    let left_team = vec![
        make_unit(
            1,
            "Footsoldier",
            3,
            2,
            Trigger::BeforeStrike,
            vec![ability(
                1,
                "Strike",
                TargetType::RandomEnemy,
                "deal_damage(target[\"id\"], X * level);",
            )],
            BattleSide::Left,
            0,
        ),
        make_unit(
            2,
            "Shieldbearer",
            4,
            1,
            Trigger::BattleStart,
            vec![ability(
                2,
                "Guard",
                TargetType::Owner,
                "add_shield(owner[\"id\"], X * level);",
            )],
            BattleSide::Left,
            1,
        ),
        make_unit(
            3,
            "Medic",
            3,
            3,
            Trigger::TurnEnd,
            vec![ability(
                3,
                "Heal",
                TargetType::RandomAlly,
                "heal_damage(target[\"id\"], X * level);",
            )],
            BattleSide::Left,
            2,
        ),
        make_unit(
            4,
            "Paladin",
            8,
            6,
            Trigger::BeforeStrike,
            vec![
                ability(
                    1,
                    "Strike",
                    TargetType::RandomEnemy,
                    "deal_damage(target[\"id\"], X * level);",
                ),
                ability(
                    2,
                    "Guard",
                    TargetType::Owner,
                    "add_shield(owner[\"id\"], X * level);",
                ),
            ],
            BattleSide::Left,
            3,
        ),
    ];

    let right_team = vec![
        make_unit(
            10,
            "Hexer",
            2,
            3,
            Trigger::BattleStart,
            vec![ability(
                4,
                "Curse",
                TargetType::RandomEnemy,
                "change_stat(target[\"id\"], \"pwr\", -level);",
            )],
            BattleSide::Right,
            0,
        ),
        make_unit(
            11,
            "Knight",
            6,
            4,
            Trigger::BeforeStrike,
            vec![ability(
                1,
                "Strike",
                TargetType::RandomEnemy,
                "deal_damage(target[\"id\"], X * level);",
            )],
            BattleSide::Right,
            1,
        ),
        make_unit(
            12,
            "Priest",
            5,
            5,
            Trigger::DamageTaken,
            vec![ability(
                3,
                "Heal",
                TargetType::RandomAlly,
                "heal_damage(target[\"id\"], X * level);",
            )],
            BattleSide::Right,
            2,
        ),
        make_unit(
            13,
            "Warlock",
            7,
            7,
            Trigger::TurnEnd,
            vec![
                ability(
                    1,
                    "Strike",
                    TargetType::RandomEnemy,
                    "deal_damage(target[\"id\"], X * level);",
                ),
                ability(
                    4,
                    "Curse",
                    TargetType::RandomEnemy,
                    "change_stat(target[\"id\"], \"pwr\", -level);",
                ),
            ],
            BattleSide::Right,
            3,
        ),
    ];

    // Build visuals
    let colors_left = [
        egui::Color32::from_rgb(170, 68, 68),
        egui::Color32::from_rgb(68, 68, 170),
        egui::Color32::from_rgb(68, 170, 68),
        egui::Color32::from_rgb(200, 200, 68),
    ];
    let colors_right = [
        egui::Color32::from_rgb(170, 68, 170),
        egui::Color32::from_rgb(200, 100, 68),
        egui::Color32::from_rgb(100, 200, 100),
        egui::Color32::from_rgb(136, 68, 200),
    ];

    let mut visuals = Vec::new();
    for (i, u) in left_team.iter().enumerate() {
        visuals.push(BattleUnitVisual {
            id: u.id,
            name: u.name.clone(),
            hp: u.hp,
            pwr: u.pwr,
            dmg: 0,
            alive: true,
            side: BattleSide::Left,
            slot: u.slot,
            color: colors_left[i % colors_left.len()],
        });
    }
    for (i, u) in right_team.iter().enumerate() {
        visuals.push(BattleUnitVisual {
            id: u.id,
            name: u.name.clone(),
            hp: u.hp,
            pwr: u.pwr,
            dmg: 0,
            alive: true,
            side: BattleSide::Right,
            slot: u.slot,
            color: colors_right[i % colors_right.len()],
        });
    }

    // Run simulation
    let result = simulate_battle(left_team, right_team);
    info!(
        "Battle result: {:?} wins in {} turns ({} actions)",
        result.winner,
        result.turns,
        result.actions.len()
    );

    scene.load(result, visuals);
    next_state.set(GameState::Battle);
}

use bevy_egui::egui;

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

fn ability(id: u64, name: &str, target_type: TargetType, script: &str) -> BattleAbility {
    BattleAbility {
        id,
        name: name.to_string(),
        target_type,
        effect_script: script.to_string(),
    }
}
