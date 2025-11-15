use super::*;
use crate::tests::battle_builder::*;

#[test]
fn test_simple_1v1_battle() {
    let mut builder = TestBuilder::new();

    let unit1 = builder.create_unit("Unit1", 1, 1);
    let unit2 = builder.create_unit("Unit2", 1, 2);

    let house1 = builder.create_simple_house("House1", vec![unit1]);
    let house2 = builder.create_simple_house("House2", vec![unit2]);

    let left_team = builder
        .create_team()
        .add_house(house1)
        .add_fusion(FusionBuilder::single(0)); // Reference first unit in house

    let right_team = builder
        .create_team()
        .add_house(house2)
        .add_fusion(FusionBuilder::single(0)); // Reference first unit in house

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Right);
    result.assert_units_alive(TeamSide::Right, 1);
    result.assert_units_alive(TeamSide::Left, 0);
}

#[test]
fn test_fatigue_1v1_battle() {
    let mut builder = TestBuilder::new();

    let unit1 = builder.create_unit("Unit1", 0, 1);
    let unit2 = builder.create_unit("Unit2", 0, 2);

    let house1 = builder.create_simple_house("House1", vec![unit1]);
    let house2 = builder.create_simple_house("House2", vec![unit2]);

    let left_team = builder
        .create_team()
        .add_house(house1)
        .add_fusion(FusionBuilder::single(0)); // Reference first unit in house

    let right_team = builder
        .create_team()
        .add_house(house2)
        .add_fusion(FusionBuilder::single(0)); // Reference first unit in house

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Right);
    result.assert_units_alive(TeamSide::Right, 1);
    result.assert_units_alive(TeamSide::Left, 0);
}

#[test]
fn test_equal_units_draw() {
    let mut builder = TestBuilder::new();

    let unit1 = builder.create_unit("Unit1", 1, 1);
    let unit2 = builder.create_unit("Unit2", 1, 1);

    let house1 = builder.create_simple_house("House1", vec![unit1]);
    let house2 = builder.create_simple_house("House2", vec![unit2]);

    let left_team = builder
        .create_team()
        .add_house(house1)
        .add_fusion(FusionBuilder::single(0));

    let right_team = builder
        .create_team()
        .add_house(house2)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_units_alive(TeamSide::Left, 0);
    result.assert_units_alive(TeamSide::Right, 0);
}

#[test]
fn test_2v1_battle() {
    let mut builder = TestBuilder::new();

    let unit1 = builder.create_unit("Unit1", 1, 1);
    let unit2 = builder.create_unit("Unit2", 1, 1);
    let unit3 = builder.create_unit("Unit3", 2, 3);

    let house1 = builder.create_simple_house("House1", vec![unit1, unit2]);
    let house2 = builder.create_simple_house("House2", vec![unit3]);

    let left_team = builder
        .create_team()
        .add_house(house1)
        .add_fusion(FusionBuilder::single(0)) // First unit
        .add_fusion(FusionBuilder::single(1)); // Second unit

    let right_team = builder
        .create_team()
        .add_house(house2)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Right);
}

#[test]
fn test_fusion_battle() {
    let mut builder = TestBuilder::new();

    let unit1 = builder.create_unit("Unit1", 1, 1);
    let unit2 = builder.create_unit("Unit2", 1, 1);
    let unit3 = builder.create_unit("Unit3", 2, 2);

    let house1 = builder.create_simple_house("House1", vec![unit1, unit2]);
    let house2 = builder.create_simple_house("House2", vec![unit3]);

    let fusion = FusionBuilder::new(vec![0, 1]); // Fuse first two units from house1

    let left_team = builder.create_team().add_house(house1).add_fusion(fusion);

    let right_team = builder
        .create_team()
        .add_house(house2)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_units_alive(TeamSide::Left, 0);
    result.assert_units_alive(TeamSide::Right, 0);
}

#[test]
fn test_damage_dealer_unit() {
    let mut builder = TestBuilder::new();

    let damage_dealer = builder.create_unit_with_behavior("Unit1", 1, 1, reaction_deal_damage(2));
    let normal_unit = builder.create_unit("Unit2", 1, 3);

    let house1 = builder.create_simple_house("House1", vec![damage_dealer]);
    let house2 = builder.create_simple_house("House2", vec![normal_unit]);

    let left_team = builder
        .create_team()
        .add_house(house1)
        .add_fusion(FusionBuilder::single(0));

    let right_team = builder
        .create_team()
        .add_house(house2)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Right);
}

#[test]
fn test_healer_unit() {
    let mut builder = TestBuilder::new();

    let healer = builder.create_unit_with_behavior(
        "Unit1",
        1,
        2,
        Reaction {
            trigger: Trigger::TurnEnd,
            actions: [
                Action::add_target(Expression::owner.into()),
                Action::add_value(Expression::one.into()),
                Action::heal_damage,
            ]
            .into(),
        },
    );
    let attacker = builder.create_unit("Unit2", 1, 3);

    let house1 = builder.create_simple_house("House1", vec![healer]);
    let house2 = builder.create_simple_house("House2", vec![attacker]);

    let left_team = builder
        .create_team()
        .add_house(house1)
        .add_fusion(FusionBuilder::single(0));

    let right_team = builder
        .create_team()
        .add_house(house2)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Left);
}

#[test]
fn test_multi_fusion_battle() {
    let mut builder = TestBuilder::new();

    let units_left = vec![
        builder.create_unit("Unit1", 1, 1),
        builder.create_unit("Unit2", 1, 1),
        builder.create_unit("Unit3", 1, 1),
    ];

    let units_right = vec![
        builder.create_unit("Unit4", 2, 1),
        builder.create_unit("Unit5", 1, 2),
    ];

    let house1 = builder.create_simple_house("House1", units_left);
    let house2 = builder.create_simple_house("House2", units_right);

    let left_fusion = FusionBuilder::new(vec![0, 1, 2]); // All three units
    let right_fusion = FusionBuilder::new(vec![0, 1]); // Both units

    let left_team = builder
        .create_team()
        .add_house(house1)
        .add_fusion(left_fusion);

    let right_team = builder
        .create_team()
        .add_house(house2)
        .add_fusion(right_fusion);

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_units_alive(TeamSide::Left, 0);
    result.assert_units_alive(TeamSide::Right, 0);
}

#[test]
fn test_battle_with_abilities() {
    let mut builder = TestBuilder::new();

    let ability = builder.create_ability("Ability1").deal_3_damage();

    let house = builder
        .create_house("House1", "#FFFF00")
        .ability(ability)
        .add_unit(
            builder.create_unit("Unit1", 1, 2).behavior(Reaction {
                trigger: Trigger::BattleStart,
                actions: [
                    Action::add_target(
                        Expression::random_unit(Expression::all_enemy_units.into()).into(),
                    ),
                    Action::use_ability,
                ]
                .into(),
            }),
        );

    let enemy_unit = builder.create_unit("Unit2", 1, 4);
    let enemy_house = builder.create_simple_house("EnemyHouse", vec![enemy_unit]);

    let left_team = builder
        .create_team()
        .add_house(house)
        .add_fusion(FusionBuilder::single(0));

    let right_team = builder
        .create_team()
        .add_house(enemy_house)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Left);
}

#[test]
fn test_battle_with_status_effects() {
    let mut builder = TestBuilder::new();

    let status = builder
        .create_status("Status1")
        .description("Deals damage over time")
        .add_reaction(
            Trigger::TurnEnd,
            vec![
                Action::add_target(
                    Expression::random_unit(Expression::all_enemy_units.into()).into(),
                ),
                Action::set_value(Box::new(Expression::i32(3))),
                Action::deal_damage,
            ],
        );

    let unit = builder.create_unit("Unit1", 0, 3).behavior(Reaction {
        trigger: Trigger::BattleStart,
        actions: vec![
            Action::add_target(Expression::owner.into()),
            Action::apply_status,
        ],
    });
    let house = builder
        .create_house("House1", "#00FF00")
        .status(status)
        .add_unit(unit);

    let enemy_unit = builder.create_unit("Unit2", 1, 2);
    let enemy_house = builder.create_simple_house("EnemyHouse", vec![enemy_unit]);

    let left_team = builder
        .create_team()
        .add_house(house)
        .add_fusion(FusionBuilder::single(0));

    let right_team = builder
        .create_team()
        .add_house(enemy_house)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Left);
}

#[test]
fn test_change_out_dmg_trigger() {
    let mut builder = TestBuilder::new();

    let unit = builder.create_unit_with_behavior(
        "Unit1",
        1,
        2,
        Reaction {
            trigger: Trigger::ChangeOutgoingDamage,
            actions: vec![Action::add_value(Box::new(Expression::i32(5)))],
        },
    );

    let enemy = builder.create_unit("Unit2", 1, 6);

    let house1 = builder.create_simple_house("House1", vec![unit]);
    let house2 = builder.create_simple_house("House2", vec![enemy]);

    let left_team = builder
        .create_team()
        .add_house(house1)
        .add_fusion(FusionBuilder::single(0));

    let right_team = builder
        .create_team()
        .add_house(house2)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Left);
}

#[test]
fn test_change_in_dmg_trigger() {
    let mut builder = TestBuilder::new();

    let unit = builder.create_unit_with_behavior(
        "Unit1",
        1,
        1,
        Reaction {
            trigger: Trigger::ChangeIncomingDamage,
            actions: vec![Action::set_value(Expression::zero.into())],
        },
    );

    let enemy = builder.create_unit("Unit2", 10, 3);

    let house1 = builder.create_simple_house("House1", vec![unit]);
    let house2 = builder.create_simple_house("House2", vec![enemy]);

    let left_team = builder
        .create_team()
        .add_house(house1)
        .add_fusion(FusionBuilder::single(0));

    let right_team = builder
        .create_team()
        .add_house(house2)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Left);
}

#[test]
fn test_change_stats_status() {
    let mut builder = TestBuilder::new();

    let unit = builder.create_unit_with_behavior(
        "Unit1",
        1,
        1,
        Reaction {
            trigger: Trigger::BattleStart,
            actions: [
                Action::add_target(Expression::owner.into()),
                Action::apply_status,
            ]
            .into(),
        },
    );

    let enemy = builder.create_unit("Unit2", 5, 2);

    let house1 = builder.create_simple_house("House1", vec![unit]).status(
        builder.create_status("add 10 hp").add_reaction(
            Trigger::ChangeStat(VarName::hp),
            [Action::add_value(Expression::i32(10).into())].into(),
        ),
    );
    let house2 = builder.create_simple_house("House2", vec![enemy]);

    let left_team = builder
        .create_team()
        .add_house(house1)
        .add_fusion(FusionBuilder::single(0));

    let right_team = builder
        .create_team()
        .add_house(house2)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Left);
}

#[test]
fn test_before_strike_trigger() {
    let mut builder = TestBuilder::new();

    let ability = builder.create_ability("dmg").deal_3_damage();

    let striker = builder.create_unit_with_behavior(
        "Striker",
        1,
        2,
        Reaction {
            trigger: Trigger::BattleStart,
            actions: [
                Action::add_target(
                    Expression::random_unit(Expression::all_enemy_units.into()).into(),
                ),
                Action::use_ability,
            ]
            .into(),
        },
    );

    let boosted_unit = builder.create_unit_with_behavior(
        "BoostedUnit",
        1,
        2,
        Reaction {
            trigger: Trigger::BeforeStrike,
            actions: vec![Action::add_value(Box::new(Expression::i32(5)))],
        },
    );

    let house = builder
        .create_house("House1", "#FFFF00")
        .ability(ability)
        .add_unit(striker)
        .add_unit(boosted_unit);

    let enemy = builder.create_unit("Enemy", 1, 8);
    let enemy_house = builder.create_simple_house("House2", vec![enemy]);

    let left_team = builder
        .create_team()
        .add_house(house)
        .add_fusion(FusionBuilder::single(0));

    let right_team = builder
        .create_team()
        .add_house(enemy_house)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Left);
}

#[test]
fn test_after_strike_trigger() {
    let mut builder = TestBuilder::new();

    let ability = builder.create_ability("Strike").deal_3_damage();

    let striker = builder.create_unit_with_behavior(
        "Striker",
        1,
        2,
        Reaction {
            trigger: Trigger::BattleStart,
            actions: [
                Action::add_target(
                    Expression::random_unit(Expression::all_enemy_units.into()).into(),
                ),
                Action::use_ability,
            ]
            .into(),
        },
    );

    let healer = builder.create_unit_with_behavior(
        "Healer",
        1,
        2,
        Reaction {
            trigger: Trigger::AfterStrike,
            actions: vec![Action::add_value(Box::new(Expression::i32(3)))],
        },
    );

    let house = builder
        .create_house("House1", "#FFFF00")
        .ability(ability)
        .add_unit(striker)
        .add_unit(healer);

    let enemy = builder.create_unit("Enemy", 1, 5);
    let enemy_house = builder.create_simple_house("House2", vec![enemy]);

    let left_team = builder
        .create_team()
        .add_house(house)
        .add_fusion(FusionBuilder::single(0));

    let right_team = builder
        .create_team()
        .add_house(enemy_house)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Left);
}

#[test]
fn test_damage_taken_trigger() {
    let mut builder = TestBuilder::new();

    let ability = builder.create_ability("Strike").deal_3_damage();

    let attacker = builder.create_unit_with_behavior(
        "Attacker",
        1,
        3,
        Reaction {
            trigger: Trigger::BattleStart,
            actions: [
                Action::add_target(
                    Expression::random_unit(Expression::all_enemy_units.into()).into(),
                ),
                Action::use_ability,
            ]
            .into(),
        },
    );

    let defender = builder.create_unit_with_behavior(
        "Defender",
        1,
        1,
        Reaction {
            trigger: Trigger::DamageTaken,
            actions: vec![Action::add_value(Box::new(Expression::i32(4)))],
        },
    );

    let house1 = builder
        .create_house("House1", "#FFFF00")
        .ability(ability)
        .add_unit(attacker);

    let house2 = builder.create_house("House2", "#0000FF").add_unit(defender);

    let left_team = builder
        .create_team()
        .add_house(house1)
        .add_fusion(FusionBuilder::single(0));

    let right_team = builder
        .create_team()
        .add_house(house2)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Right);
}

#[test]
fn test_damage_dealt_trigger() {
    let mut builder = TestBuilder::new();

    let ability = builder.create_ability("Strike").deal_3_damage();

    let attacker = builder.create_unit_with_behavior(
        "Attacker",
        1,
        2,
        Reaction {
            trigger: Trigger::BattleStart,
            actions: [
                Action::add_target(
                    Expression::random_unit(Expression::all_enemy_units.into()).into(),
                ),
                Action::use_ability,
            ]
            .into(),
        },
    );

    let boosted = builder.create_unit_with_behavior(
        "Boosted",
        1,
        2,
        Reaction {
            trigger: Trigger::DamageDealt,
            actions: vec![Action::add_value(Box::new(Expression::i32(3)))],
        },
    );

    let house = builder
        .create_house("House1", "#FFFF00")
        .ability(ability)
        .add_unit(attacker)
        .add_unit(boosted);

    let enemy = builder.create_unit("Enemy", 1, 6);
    let enemy_house = builder.create_simple_house("House2", vec![enemy]);

    let left_team = builder
        .create_team()
        .add_house(house)
        .add_fusion(FusionBuilder::single(0));

    let right_team = builder
        .create_team()
        .add_house(enemy_house)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Left);
}

#[test]
fn test_status_applied_trigger() {
    let mut builder = TestBuilder::new();

    let status = builder.create_status("Boost").add_reaction(
        Trigger::ChangeStat(VarName::pwr),
        vec![Action::add_value(Expression::i32(2).into())],
    );

    let applier = builder.create_unit_with_behavior(
        "Applier",
        1,
        2,
        Reaction {
            trigger: Trigger::BattleStart,
            actions: vec![
                Action::add_target(Expression::all_ally_units.into()),
                Action::apply_status,
            ],
        },
    );

    let receiver = builder.create_unit_with_behavior(
        "Receiver",
        1,
        1,
        Reaction {
            trigger: Trigger::StatusApplied,
            actions: vec![Action::add_value(Box::new(Expression::i32(3)))],
        },
    );

    let house = builder
        .create_house("House1", "#FFFF00")
        .status(status)
        .add_unit(applier)
        .add_unit(receiver);

    let enemy = builder.create_unit("Enemy", 1, 5);
    let enemy_house = builder.create_simple_house("House2", vec![enemy]);

    let left_team = builder
        .create_team()
        .add_house(house)
        .add_fusion(FusionBuilder::single(0));

    let right_team = builder
        .create_team()
        .add_house(enemy_house)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Left);
}

#[test]
fn test_combined_triggers() {
    let mut builder = TestBuilder::new();

    let ability = builder.create_ability("Strike").deal_3_damage();

    let attacker = builder.create_unit_with_behavior(
        "Attacker",
        1,
        3,
        Reaction {
            trigger: Trigger::BattleStart,
            actions: [
                Action::add_target(
                    Expression::random_unit(Expression::all_enemy_units.into()).into(),
                ),
                Action::use_ability,
            ]
            .into(),
        },
    );

    let before_striker = builder.create_unit_with_behavior(
        "BeforeStriker",
        1,
        2,
        Reaction {
            trigger: Trigger::BeforeStrike,
            actions: vec![Action::add_value(Box::new(Expression::i32(2)))],
        },
    );

    let after_striker = builder.create_unit_with_behavior(
        "AfterStriker",
        1,
        2,
        Reaction {
            trigger: Trigger::AfterStrike,
            actions: vec![Action::add_value(Box::new(Expression::i32(1)))],
        },
    );

    let house = builder
        .create_house("House1", "#FFFF00")
        .ability(ability)
        .add_unit(attacker)
        .add_unit(before_striker)
        .add_unit(after_striker);

    let enemy = builder.create_unit("Enemy", 1, 8);
    let enemy_house = builder.create_simple_house("House2", vec![enemy]);

    let left_team = builder
        .create_team()
        .add_house(house)
        .add_fusion(FusionBuilder::single(0));

    let right_team = builder
        .create_team()
        .add_house(enemy_house)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Left);
}

#[test]
fn test_before_strike_nullifies_damage() {
    let mut builder = TestBuilder::new();

    let ability = builder.create_ability("Strike").deal_3_damage();

    let attacker = builder.create_unit_with_behavior(
        "Attacker",
        1,
        1,
        Reaction {
            trigger: Trigger::BattleStart,
            actions: [
                Action::add_target(
                    Expression::random_unit(Expression::all_enemy_units.into()).into(),
                ),
                Action::use_ability,
            ]
            .into(),
        },
    );

    let saboteur = builder.create_unit_with_behavior(
        "Saboteur",
        1,
        10,
        Reaction {
            trigger: Trigger::BeforeStrike,
            actions: vec![Action::set_value(Expression::i32(0).into())],
        },
    );

    let house1 = builder
        .create_house("House1", "#FFFF00")
        .ability(ability)
        .add_unit(attacker);

    let house2 = builder.create_house("House2", "#0000FF").add_unit(saboteur);

    let left_team = builder
        .create_team()
        .add_house(house1)
        .add_fusion(FusionBuilder::single(0));

    let right_team = builder
        .create_team()
        .add_house(house2)
        .add_fusion(FusionBuilder::single(0));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    result.assert_winner(TeamSide::Right);
}
