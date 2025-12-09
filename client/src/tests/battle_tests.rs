use super::*;
use crate::tests::battle_builder::*;

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
            vec![
                Action::set_value(Box::new(Expression::i32(2))),
                Action::deal_damage,
            ],
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
            vec![
                Action::add_target(Expression::owner.into()),
                Action::add_value(Box::new(Expression::i32(1))),
                Action::heal_damage,
            ],
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
            vec![
                Action::set_value(Box::new(Expression::i32(3))),
                Action::deal_damage,
            ],
        )
        .add_unit(1200, 1, 2)
        .add_reaction(
            Trigger::BattleStart,
            vec![
                Action::add_target(
                    Expression::random_unit(Expression::all_enemy_units.into()).into(),
                ),
                Action::use_ability(
                    "test_house".to_string(),
                    "test_ability".to_string(),
                    HexColor::default(),
                ),
            ],
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
            vec![
                Action::add_target(
                    Expression::random_unit(Expression::all_enemy_units.into()).into(),
                ),
                Action::set_value(Box::new(Expression::i32(3))),
                Action::deal_damage,
            ],
        )
        .add_unit(1200, 0, 3)
        .add_reaction(
            Trigger::BattleStart,
            vec![
                Action::add_target(Expression::owner.into()),
                Action::apply_status(
                    "test_house".to_string(),
                    "test_status".to_string(),
                    HexColor::default(),
                ),
            ],
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
            vec![Action::add_value(Box::new(Expression::i32(5)))],
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
            vec![Action::set_value(Expression::zero.into())],
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
            vec![Action::add_value(Box::new(Expression::i32(10)))],
        )
        .add_unit(1200, 1, 1)
        .add_reaction(
            Trigger::BattleStart,
            vec![
                Action::add_target(Expression::owner.into()),
                Action::apply_status(
                    "test_house".to_string(),
                    "test_status".to_string(),
                    HexColor::default(),
                ),
            ],
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
            vec![
                Action::add_target(
                    Expression::random_unit(Expression::all_enemy_units.into()).into(),
                ),
                Action::add_value(Box::new(Expression::i32(3))),
                Action::deal_damage,
            ],
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
            vec![
                Action::add_target(Expression::owner.into()),
                Action::add_value(Box::new(Expression::i32(1))),
                Action::heal_damage,
            ],
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
            vec![
                Action::add_target(Expression::owner.into()),
                Action::add_value(Box::new(Expression::i32(1))),
                Action::heal_damage,
            ],
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
            vec![
                Action::add_target(Expression::owner.into()),
                Action::add_value(Box::new(Expression::i32(1))),
                Action::heal_damage,
            ],
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
        .add_status(1120, Trigger::BattleStart, vec![])
        .add_unit(1200, 1, 2)
        .add_reaction(
            Trigger::StatusGained,
            [
                Action::add_target(Expression::owner.into()),
                Action::add_value(Box::new(Expression::i32(1))),
                Action::heal_damage,
            ],
        )
        .add_unit(1300, 0, 1)
        .add_reaction(
            Trigger::TurnEnd,
            [
                Action::add_target(Expression::adjacent_front.into()),
                Action::apply_status(
                    "test_house".to_string(),
                    "test_status".to_string(),
                    HexColor::default(),
                ),
            ],
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
            vec![
                Action::add_target(
                    Expression::random_unit(Expression::all_enemy_units.into()).into(),
                ),
                Action::set_value(Box::new(Expression::i32(2))),
                Action::deal_damage,
            ],
        )
        .add_status(
            1120,
            Trigger::TurnEnd,
            vec![
                Action::add_target(Expression::owner.into()),
                Action::add_value(Box::new(Expression::i32(1))),
                Action::heal_damage,
            ],
        )
        .add_unit(1200, 1, 2)
        .add_reaction(
            Trigger::BattleStart,
            vec![
                Action::add_target(Expression::owner.into()),
                Action::apply_status(
                    "test_house".to_string(),
                    "test_status".to_string(),
                    HexColor::default(),
                ),
            ],
        )
        .add_unit(1300, 1, 1)
        .add_reaction(
            Trigger::BattleStart,
            vec![
                Action::add_target(
                    Expression::random_unit(Expression::all_enemy_units.into()).into(),
                ),
                Action::use_ability(
                    "test_house".to_string(),
                    "test_ability".to_string(),
                    HexColor::default(),
                ),
            ],
        )
        .add_team(2000)
        .add_house(2100)
        .add_unit(2200, 1, 4)
        .run_battle()
        .assert_winner(TeamSide::Left);
}
