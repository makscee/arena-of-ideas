use super::*;
use crate::tests::battle_builder::*;

fn damage_script(amount: i32, target: &str) -> String {
    format!(
        r#"
        let target_id = {};
        status_actions.deal_damage(target_id, {});
        "#,
        target, amount
    )
}

fn heal_script(amount: i32, target: &str) -> String {
    format!(
        r#"
        let target_id = {};
        status_actions.heal_damage(target_id, {});
        "#,
        target, amount
    )
}

fn apply_status_script(status_name: &str, house_id: u64, status_id: u64, target: &str) -> String {
    format!(
        r#"
        let target_id = {};
        unit_actions.apply_status("{}", target_id, 1);
        "#,
        target, status_name
    )
}

fn use_ability_script(ability_name: &str, house_id: u64, ability_id: u64, target: &str) -> String {
    format!(
        r#"
        let target_id = {};
        unit_actions.use_ability("{}", target_id);
        "#,
        target, ability_name
    )
}

#[test]
fn test_simple_1v1_battle() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_unit(1200, 1, 1)
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 1, 2)
        .run_battle()
        .assert_winner(TeamSide::Right);
}

#[test]
fn test_fatigue_1v1_battle() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_unit(1200, 0, 1)
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 0, 2)
        .run_battle()
        .assert_winner(TeamSide::Right);
}

#[test]
fn test_equal_units_draw() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_unit(1200, 1, 1)
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 1, 1)
        .run_battle()
        .assert_draw();
}

#[test]
fn test_2v1_battle() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_unit(1200, 1, 1)
        .add_unit(1300, 1, 1)
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 2, 3)
        .run_battle()
        .assert_winner(TeamSide::Right);
}

#[test]
fn test_damage_dealer_unit() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_unit(1200, 1, 1)
        .add_reaction(
            Trigger::BattleStart,
            Target::RandomEnemy,
            r#"
            let enemies = ctx.get_enemies(owner.id);
            if enemies.len() > 0 {
                status_actions.deal_damage(enemies[0], 2);
            }
            "#
            .to_string(),
        )
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 1, 3)
        .run_battle()
        .assert_winner(TeamSide::Right);
}

#[test]
fn test_healer_unit() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_unit(1200, 1, 2)
        .add_reaction(
            Trigger::TurnEnd,
            Target::Owner,
            r#"
            status_actions.heal_damage(owner.id, 1);
            "#
            .to_string(),
        )
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 1, 3)
        .run_battle()
        .assert_winner(TeamSide::Left);
}

#[test]
fn test_battle_with_abilities() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_ability(
            1110,
            r#"
            let enemies = ctx.get_enemies(target.id);
            if enemies.len() > 0 {
                ability_actions.deal_damage(enemies[0], 3);
            }
            "#
            .to_string(),
        )
        .add_unit(1200, 1, 2)
        .add_reaction(
            Trigger::BattleStart,
            Target::RandomEnemy,
            r#"
            let enemies = ctx.get_enemies(owner.id);
            if enemies.len() > 0 {
                unit_actions.use_ability("Ability 1110", enemies[0]);
            }
            "#
            .to_string(),
        )
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 1, 4)
        .run_battle()
        .assert_winner(TeamSide::Left);
}

#[test]
fn test_battle_with_status_effects() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_status(
            1120,
            Trigger::TurnEnd,
            r#"
            let enemies = ctx.get_enemies(status.id);
            if enemies.len() > 0 {
                status_actions.deal_damage(enemies[0], 3);
            }
            "#
            .to_string(),
        )
        .add_unit(1200, 0, 3)
        .add_reaction(
            Trigger::BattleStart,
            Target::Owner,
            r#"
            unit_actions.apply_status("Status 1120", owner.id, 1);
            "#
            .to_string(),
        )
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 1, 3)
        .run_battle()
        .assert_winner(TeamSide::Left);
}

#[test]
fn test_change_out_dmg_trigger() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_unit(1200, 1, 2)
        .add_reaction(
            Trigger::ChangeOutgoingDamage,
            Target::default(),
            r#"
            // This test depends on damage modification system
            "#
            .to_string(),
        )
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 1, 6)
        .run_battle()
        .assert_winner(TeamSide::Left);
}

#[test]
fn test_change_in_dmg_trigger() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_unit(1200, 1, 1)
        .add_reaction(
            Trigger::ChangeIncomingDamage,
            Target::default(),
            r#"
            // This test depends on damage modification system
            "#
            .to_string(),
        )
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 10, 3)
        .run_battle()
        .assert_winner(TeamSide::Left);
}

#[test]
fn test_change_stats_status() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_status(
            1120,
            Trigger::ChangeStat(VarName::hp),
            r#"
            // Stat modification handler
            "#
            .to_string(),
        )
        .add_unit(1200, 1, 1)
        .add_reaction(
            Trigger::BattleStart,
            Target::Owner,
            r#"
            unit_actions.apply_status("Status 1120", owner.id, 1);
            "#
            .to_string(),
        )
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 5, 2)
        .run_battle()
        .assert_winner(TeamSide::Left);
}

#[test]
fn test_before_strike_trigger() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_unit(1200, 1, 3)
        .add_reaction(
            Trigger::BeforeStrike,
            Target::RandomEnemy,
            r#"
            let enemies = ctx.get_enemies(owner.id);
            if enemies.len() > 0 {
                status_actions.deal_damage(enemies[0], 3);
            }
            "#
            .to_string(),
        )
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 1, 5)
        .run_battle()
        .assert_winner(TeamSide::Left);
}

#[test]
fn test_after_strike_trigger() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_unit(1200, 1, 2)
        .add_reaction(
            Trigger::AfterStrike,
            Target::Owner,
            r#"
            status_actions.heal_damage(owner.id, 1);
            "#
            .to_string(),
        )
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 1, 3)
        .run_battle()
        .assert_winner(TeamSide::Left);
}

#[test]
fn test_damage_taken_trigger() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_unit(1200, 1, 3)
        .add_reaction(
            Trigger::DamageTaken,
            Target::Owner,
            r#"
            status_actions.heal_damage(owner.id, 1);
            "#
            .to_string(),
        )
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 1, 5)
        .run_battle()
        .assert_winner(TeamSide::Left);
}

#[test]
fn test_damage_dealt_trigger() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_unit(1200, 1, 3)
        .add_reaction(
            Trigger::DamageDealt,
            Target::Owner,
            r#"
            status_actions.heal_damage(owner.id, 1);
            "#
            .to_string(),
        )
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 1, 3)
        .run_battle()
        .assert_winner(TeamSide::Left);
}

#[test]
fn test_status_applied_trigger() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_status(1120, Trigger::BattleStart, "".to_string())
        .add_unit(1200, 1, 2)
        .add_reaction(
            Trigger::StatusGained,
            Target::Owner,
            r#"
            status_actions.heal_damage(owner.id, 1);
            "#
            .to_string(),
        )
        .add_unit(1300, 0, 1)
        .add_reaction(
            Trigger::TurnEnd,
            Target::AdjacentFront,
            r#"
            let allies = ctx.get_allies(owner.id);
            if allies.len() > 0 {
                unit_actions.apply_status("Status 1120", allies[0], 1);
            }
            "#
            .to_string(),
        )
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 1, 6)
        .run_battle()
        .assert_winner(TeamSide::Left);
}

#[test]
fn test_combined_triggers() {
    TestBuilder::new()
        .add_team(1000)
        .add_house(1100)
        .add_ability(
            1110,
            r#"
            let enemies = ctx.get_enemies(target.id);
            if enemies.len() > 0 {
                ability_actions.deal_damage(enemies[0], 2);
            }
            "#
            .to_string(),
        )
        .add_status(
            1120,
            Trigger::TurnEnd,
            r#"
            status_actions.heal_damage(status.id, 1);
            "#
            .to_string(),
        )
        .add_unit(1200, 1, 2)
        .add_reaction(
            Trigger::BattleStart,
            Target::Owner,
            r#"
            unit_actions.apply_status("Status 1120", owner.id, 1);
            "#
            .to_string(),
        )
        .add_unit(1300, 1, 1)
        .add_reaction(
            Trigger::BattleStart,
            Target::RandomEnemy,
            r#"
            let enemies = ctx.get_enemies(owner.id);
            if enemies.len() > 0 {
                unit_actions.use_ability("Ability 1110", enemies[0]);
            }
            "#
            .to_string(),
        )
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 1, 4)
        .run_battle()
        .assert_winner(TeamSide::Left);
}
