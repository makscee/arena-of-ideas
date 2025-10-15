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
                Action::add_target(Box::new(Expression::random_unit(Box::new(
                    Expression::all_enemy_units,
                )))),
                Action::set_value(Box::new(Expression::i32(2))),
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
fn test_large_scale_battle() {
    let mut builder = TestBuilder::new();

    let mut left_units = vec![];
    for i in 0..5 {
        left_units.push(builder.create_unit(&format!("Unit{}", i), 1, 2));
    }

    let mut right_units = vec![];
    for i in 5..10 {
        right_units.push(builder.create_unit(&format!("Unit{}", i), 2, 1));
    }

    let house1 = builder.create_simple_house("House1", left_units);
    let house2 = builder.create_simple_house("House2", right_units);

    let left_team = builder
        .create_team()
        .add_house(house1)
        .add_fusion(FusionBuilder::single(0))
        .add_fusion(FusionBuilder::single(1))
        .add_fusion(FusionBuilder::single(2))
        .add_fusion(FusionBuilder::single(3))
        .add_fusion(FusionBuilder::single(4));

    let right_team = builder
        .create_team()
        .add_house(house2)
        .add_fusion(FusionBuilder::single(0))
        .add_fusion(FusionBuilder::single(1))
        .add_fusion(FusionBuilder::single(2))
        .add_fusion(FusionBuilder::single(3))
        .add_fusion(FusionBuilder::single(4));

    let battle = builder.create_battle(left_team, right_team);
    let result = battle.run();

    assert!(result.iterations < 1000, "Battle should not timeout");
}

#[test]
fn test_log_verification() {
    let mut builder = TestBuilder::new();

    let unit1 = builder.create_unit("Unit1", 2, 2);
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

    assert!(!result.log.actions.is_empty(), "Log should contain actions");

    let has_damage = result
        .log
        .actions
        .iter()
        .any(|action| matches!(action, BattleAction::damage { .. }));

    assert!(has_damage, "Log should contain damage actions");
}

#[test]
fn test_zero_power_unit() {
    let mut builder = TestBuilder::new();

    let zero_power = builder.create_unit("Unit1", 0, 3);
    let normal = builder.create_unit("Unit2", 1, 1);

    let house1 = builder.create_simple_house("House1", vec![zero_power]);
    let house2 = builder.create_simple_house("House2", vec![normal]);

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
    result.assert_iterations(3);
}

#[test]
fn test_high_hp_tank() {
    let mut builder = TestBuilder::new();

    let tank = builder.create_unit("Unit1", 1, 10);
    let glass_cannon = builder.create_unit("Unit2", 5, 1);

    let house1 = builder.create_simple_house("House1", vec![tank]);
    let house2 = builder.create_simple_house("House2", vec![glass_cannon]);

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
    result.assert_iterations(1);
}
