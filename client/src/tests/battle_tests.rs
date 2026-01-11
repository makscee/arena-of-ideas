use super::*;
use crate::tests::battle_builder::*;

fn ability_path(house_id: u64, ability_id: u64) -> String {
    format!("House {}/Ability {}", house_id, ability_id)
}

fn status_path(house_id: u64, status_id: u64) -> String {
    format!("House {}/Status {}", house_id, status_id)
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
        .add_status(
            1115,
            Trigger::BattleStart,
            "status_actions.deal_damage(target.id, 2);".to_string(),
        )
        .add_unit(1200, 1, 1)
        .add_reaction(
            Trigger::BattleStart,
            Target::RandomEnemy,
            "unit_actions.apply_status(\"House 1100/Status 1115\", target.id, 1);".to_string(),
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
        .add_status(
            1115,
            Trigger::TurnEnd,
            "status_actions.heal_damage(owner.id, 1);".to_string(),
        )
        .add_unit(1200, 1, 2)
        .add_reaction(
            Trigger::BattleStart,
            Target::Owner,
            "unit_actions.apply_status(\"House 1100/Status 1115\", owner.id, 1);".to_string(),
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
            "ability_actions.deal_damage(target.id, 3);".to_string(),
        )
        .add_unit(1200, 1, 2)
        .add_reaction(
            Trigger::BattleStart,
            Target::RandomEnemy,
            format!(
                "unit_actions.use_ability(\"{}\", target.id);",
                ability_path(1100, 1110)
            ),
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
            "status_actions.deal_damage(target.id, 3);".to_string(),
        )
        .add_unit(1200, 0, 3)
        .add_reaction(
            Trigger::BattleStart,
            Target::Owner,
            "unit_actions.apply_status(\"House 1100/Status 1120\", owner.id, 1);".to_string(),
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
        .add_status(
            1115,
            Trigger::ChangeOutgoingDamage,
            "value += 5;".to_string(),
        )
        .add_unit(1200, 1, 2)
        .add_reaction(
            Trigger::BattleStart,
            Target::Owner,
            "unit_actions.apply_status(\"House 1100/Status 1115\", owner.id, 1);".to_string(),
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
        .add_status(
            1115,
            Trigger::ChangeIncomingDamage,
            "value = 0;".to_string(),
        )
        .add_unit(1200, 1, 1)
        .add_reaction(
            Trigger::BattleStart,
            Target::Owner,
            "unit_actions.apply_status(\"House 1100/Status 1115\", owner.id, 1);".to_string(),
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
            "value += 10;".to_string(),
        )
        .add_unit(1200, 1, 1)
        .add_reaction(
            Trigger::BattleStart,
            Target::Owner,
            "unit_actions.apply_status(\"House 1100/Status 1120\", owner.id, 1);".to_string(),
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
        .add_status(
            1115,
            Trigger::BeforeStrike,
            "status_actions.deal_damage(target.id, 3);".to_string(),
        )
        .add_unit(1200, 1, 3)
        .add_reaction(
            Trigger::BattleStart,
            Target::Owner,
            format!(
                "unit_actions.apply_status(\"{}\", owner.id, 1);",
                status_path(1100, 1115)
            ),
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
        .add_status(
            1115,
            Trigger::AfterStrike,
            "status_actions.heal_damage(owner.id, 1);".to_string(),
        )
        .add_unit(1200, 1, 2)
        .add_reaction(
            Trigger::BattleStart,
            Target::Owner,
            format!(
                "unit_actions.apply_status(\"{}\", owner.id, 1);",
                status_path(1100, 1115)
            ),
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
        .add_status(
            1115,
            Trigger::DamageTaken,
            "status_actions.heal_damage(owner.id, 1);".to_string(),
        )
        .add_unit(1200, 1, 3)
        .add_reaction(
            Trigger::BattleStart,
            Target::Owner,
            format!(
                "unit_actions.apply_status(\"{}\", owner.id, 1);",
                status_path(1100, 1115)
            ),
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
        .add_status(
            1115,
            Trigger::DamageDealt,
            "status_actions.heal_damage(owner.id, 1);".to_string(),
        )
        .add_unit(1200, 1, 3)
        .add_reaction(
            Trigger::BattleStart,
            Target::Owner,
            format!(
                "unit_actions.apply_status(\"{}\", owner.id, 1);",
                status_path(1100, 1115)
            ),
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
        .add_unit(1200, 1, 2)
        .add_reaction(
            Trigger::BattleStart,
            Target::Owner,
            "unit_actions.apply_status(\"House 1100/Status 1115\", owner.id, 1);".to_string(),
        )
        .add_status(
            1115,
            Trigger::StatusGained,
            "status_actions.heal_damage(owner.id, 1);".to_string(),
        )
        .add_unit(1300, 0, 1)
        .add_reaction(
            Trigger::TurnEnd,
            Target::AdjacentFront,
            format!(
                "unit_actions.apply_status(\"{}\", target.id, 1);",
                status_path(1100, 1115)
            ),
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
            "ability_actions.deal_damage(target.id, 2);".to_string(),
        )
        .add_status(
            1120,
            Trigger::TurnEnd,
            "status_actions.heal_damage(owner.id, 1);".to_string(),
        )
        .add_unit(1200, 1, 2)
        .add_reaction(
            Trigger::BattleStart,
            Target::Owner,
            format!(
                "unit_actions.apply_status(\"{}\", owner.id, 1);",
                status_path(1100, 1120)
            ),
        )
        .add_unit(1300, 1, 1)
        .add_reaction(
            Trigger::BattleStart,
            Target::RandomEnemy,
            format!(
                "unit_actions.use_ability(\"{}\", target.id);",
                ability_path(1100, 1110)
            ),
        )
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 1, 4)
        .run_battle()
        .assert_winner(TeamSide::Left);
}
