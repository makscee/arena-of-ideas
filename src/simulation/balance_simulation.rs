use super::*;
use crate::simulation::simulation_config::RegexUnit;

pub struct BalanceSimulation {
    unit: UnitType,
    repeats: usize,
    tier: usize,
    all_units: Vec<UnitTemplate>,
    all_clans: Vec<Clan>,
    config: Config,
}

impl BalanceSimulation {
    pub fn new(
        unit: UnitType,
        repeats: usize,
        tier: usize,
        all_units: Vec<UnitTemplate>,
        all_clans: Vec<Clan>,
        config: Config,
    ) -> Self {
        Self {
            unit,
            repeats,
            tier,
            all_units,
            all_clans,
            config,
        }
    }

    fn same_tier(&self, unit: &UnitTemplate) -> Vec<BattleConfig> {
        let mut battles: Vec<BattleConfig> = vec![];
        let same_tier = self
            .all_units
            .clone()
            .into_iter()
            .filter(|other| other.tier == unit.tier);
        for enemy in same_tier {
            let round = GameRound {
                name: "".to_string(),
                statuses: vec![],
                enemies: vec![enemy.name.clone()],
            };

            (1..=6).for_each(|i| {
                unit.clans.clone().into_iter().for_each(|clan| {
                    enemy.clans.clone().into_iter().for_each(|enemy_clan| {
                        battles.push(BattleConfig {
                            unit: Some(unit.name.clone()),
                            player: vec![unit.name.clone()],
                            round: round.clone(),
                            repeats: self.repeats,
                            clans: hashmap! {clan => i},
                            enemy_clans: hashmap! {enemy_clan => i},
                            group: SimulationGroup::SameTier,
                        })
                    })
                });
            });
        }
        battles
    }

    fn upper_tier(&self, unit: &UnitTemplate) -> Vec<BattleConfig> {
        let mut battles: Vec<BattleConfig> = vec![];
        if unit.tier == MAX_TIER {
            return battles;
        };

        let first_tier = self
            .all_units
            .clone()
            .into_iter()
            .filter(|other| other.tier == 1);
        let upper_tier = self
            .all_units
            .clone()
            .into_iter()
            .filter(|other| other.tier == unit.tier + 1);
        for enemy in upper_tier {
            let round = GameRound {
                name: "".to_string(),
                statuses: vec![],
                enemies: vec![enemy.name.clone()],
            };
            for ally in first_tier.clone() {
                (1..=6).for_each(|i| {
                    unit.clans.clone().into_iter().for_each(|clan| {
                        enemy.clans.clone().into_iter().for_each(|enemy_clan| {
                            battles.push(BattleConfig {
                                unit: Some(unit.name.clone()),
                                player: vec![unit.name.clone(), ally.name.clone()],
                                round: round.clone(),
                                repeats: self.repeats,
                                clans: hashmap! {clan => i},
                                enemy_clans: hashmap! {enemy_clan => i},
                                group: SimulationGroup::UpperTier,
                            });
                            battles.push(BattleConfig {
                                unit: Some(unit.name.clone()),
                                player: vec![ally.name.clone(), unit.name.clone()],
                                round: round.clone(),
                                repeats: self.repeats,
                                clans: hashmap! {clan => i},
                                enemy_clans: hashmap! {enemy_clan => i},
                                group: SimulationGroup::UpperTier,
                            });
                        })
                    });
                });
            }
        }
        battles
    }

    fn lower_tier(&self, unit: &UnitTemplate) -> Vec<BattleConfig> {
        let mut battles: Vec<BattleConfig> = vec![];
        if unit.tier == 1 {
            return battles;
        };

        let first_tier = self
            .all_units
            .clone()
            .into_iter()
            .filter(|other| other.tier == 1);
        let lower_tier = self
            .all_units
            .clone()
            .into_iter()
            .filter(|other| other.tier == unit.tier - 1);
        for enemy in lower_tier.clone() {
            for second_enemy in first_tier.clone() {
                (1..=6).for_each(|i| {
                    unit.clans.clone().into_iter().for_each(|clan| {
                        enemy.clans.clone().into_iter().for_each(|enemy_clan| {
                            let round = GameRound {
                                name: "".to_string(),
                                statuses: vec![],
                                enemies: vec![enemy.name.clone(), second_enemy.name.clone()],
                            };
                            battles.push(BattleConfig {
                                unit: Some(unit.name.clone()),
                                player: vec![unit.name.clone()],
                                round: round.clone(),
                                repeats: self.repeats,
                                clans: hashmap! {clan => i},
                                enemy_clans: hashmap! {enemy_clan => i},
                                group: SimulationGroup::LowerTier,
                            });
                            let round = GameRound {
                                name: "".to_string(),
                                statuses: vec![],
                                enemies: vec![second_enemy.name.clone(), enemy.name.clone()],
                            };
                            battles.push(BattleConfig {
                                unit: Some(unit.name.clone()),
                                player: vec![unit.name.clone()],
                                round: round.clone(),
                                repeats: self.repeats,
                                clans: hashmap! {clan => i},
                                enemy_clans: hashmap! {enemy_clan => i},
                                group: SimulationGroup::LowerTier,
                            });
                        })
                    });
                });
            }
        }
        battles
    }

    fn to_templates(&self, unit: RegexUnit, all_units: &Vec<UnitTemplate>) -> Vec<UnitTemplate> {
        let regex = regex::Regex::new(&unit).expect("Failed to parse a regular expression");
        all_units
            .iter()
            .filter(move |unit| unit.tier == self.tier && regex.is_match(&unit.long_name))
            .cloned()
            .collect()
    }
}

impl SimulationVariant for BalanceSimulation {
    fn result(&self, battles: Vec<BattleView>) -> Vec<SimulationView> {
        let mut balance: Vec<SimulationView> = vec![];
        let mut counters: HashMap<TeamView, HashMap<Group, HashMap<String, AvgCounter>>> =
            hashmap! {};

        battles.into_iter().for_each(|battle| {
            let units = counters.entry(battle.unit.unwrap()).or_insert(hashmap! {});
            let group = units.entry(battle.group.to_string()).or_insert(hashmap! {});
            let clans = group
                .entry(format!("{:?} VS {:?}", battle.clans, battle.enemy_clans))
                .or_insert(AvgCounter::new());
            if battle.win {
                clans.sum += 1.0;
            };
            clans.count += 1;
        });

        for (unit, counters) in counters {
            let groups: BTreeMap<Group, ClansGroupView> = counters
                .iter()
                .map(|(key, value)| {
                    let clans: BTreeMap<String, f64> = value
                        .iter()
                        .map(|(key, value)| (key.clone(), value.avg()))
                        .collect();
                    (
                        key.clone(),
                        ClansGroupView {
                            koef: clans.values().sum::<f64>() / value.values().len() as f64,
                            clans,
                        },
                    )
                })
                .collect();
            let koef =
                groups.values().map(|value| value.koef).sum::<f64>() / groups.values().len() as f64;
            balance.push(SimulationView {
                player: unit,
                koef,
                groups,
            });
        }
        balance
    }
    fn battles(&self) -> Vec<BattleConfig> {
        let mut battles: Vec<BattleConfig> = vec![];
        let units = self.to_templates(self.unit.clone(), &self.all_units);
        for unit in units {
            battles.append(&mut self.same_tier(&unit));
            battles.append(&mut self.lower_tier(&unit));
            battles.append(&mut self.upper_tier(&unit));
        }
        battles
    }
}
