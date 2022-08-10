use super::*;
use crate::simulation::simulation_config::RegexUnit;

pub struct UnitsSimulation {
    squad: Vec<UnitType>,
    enemies: Vec<UnitType>,
    repeats: usize,
    clan_size: Vec<usize>,
    all_units: Vec<UnitTemplate>,
    all_clans: Vec<Clan>,
    config: Config,
}

impl UnitsSimulation {
    pub fn new(
        squad: Vec<UnitType>,
        enemies: Vec<UnitType>,
        repeats: usize,
        clan_size: Vec<usize>,
        all_units: Vec<UnitTemplate>,
        all_clans: Vec<Clan>,
        config: Config,
    ) -> Self {
        Self {
            squad,
            enemies,
            repeats,
            clan_size,
            all_units,
            all_clans,
            config,
        }
    }

    fn to_templates(&self, unit: RegexUnit, all_units: &Vec<UnitTemplate>) -> Vec<UnitTemplate> {
        let regex = regex::Regex::new(&unit).expect("Failed to parse a regular expression");
        all_units
            .iter()
            .filter(move |unit| regex.is_match(&unit.long_name))
            .cloned()
            .collect()
    }

    fn match_units(
        &self,
        all_units: &Vec<UnitTemplate>,
        units: &Vec<RegexUnit>,
        index: usize,
        result: Vec<Vec<UnitTemplate>>,
    ) -> Vec<Vec<UnitTemplate>> {
        let mut cloned = result.clone();
        if index == units.len() {
            return cloned;
        }

        if cloned.is_empty() {
            cloned.push(vec![]);
        }

        let regex_units = self.to_templates(units[index].clone(), all_units);
        let mut regex_peek = regex_units.into_iter().peekable();
        while let Some(unit) = regex_peek.next() {
            let mut last_index = cloned.len() - 1;
            cloned[last_index].push(unit);
            cloned = self.match_units(all_units, units, index + 1, cloned);
            last_index = cloned.len() - 1;
            if regex_peek.peek().is_some() {
                //copy last line and truncate unnessesary elements
                let mut copied_line = cloned[last_index].clone();
                copied_line.truncate(index);
                cloned.push(copied_line);
            }
        }
        cloned.clone()
    }
}

impl SimulationVariant for UnitsSimulation {
    fn battles(&self) -> Vec<BattleConfig> {
        let mut player_variants = vec![];
        player_variants = self.match_units(&self.all_units, &self.squad, 0, player_variants);

        let mut unit_vars = vec![];
        unit_vars = self.match_units(&self.all_units, &self.enemies, 0, unit_vars);

        let game_rounds: Vec<GameRound> = unit_vars
            .clone()
            .into_iter()
            .map(|variant| GameRound {
                name: "".to_string(),
                statuses: vec![],
                enemies: variant
                    .into_iter()
                    .map(|template| template.name.clone())
                    .collect(),
            })
            .collect();

        let mut enemy_clans: HashSet<Clan> = hashset! {};
        unit_vars.into_iter().for_each(|variant| {
            variant.into_iter().for_each(|enemy| {
                enemy.clans.into_iter().for_each(|clan| {
                    enemy_clans.insert(clan.clone());
                })
            })
        });
        player_variants
            .into_iter()
            .flat_map(|player| {
                let mut rounds = vec![];
                for round in &game_rounds {
                    player.clone().into_iter().for_each(|unit| {
                        self.clan_size.clone().into_iter().for_each(|i| {
                            unit.clans.clone().into_iter().for_each(|clan| {
                                enemy_clans.clone().into_iter().for_each(|enemy_clan| {
                                    rounds.push(BattleConfig {
                                        unit: None,
                                        player: player
                                            .clone()
                                            .into_iter()
                                            .map(|template| template.name)
                                            .collect(),
                                        round: round.clone(),
                                        repeats: self.repeats,
                                        clans: hashmap! {clan => i},
                                        enemy_clans: hashmap! {enemy_clan=>i},
                                        group: SimulationGroup::Enemies,
                                    })
                                })
                            });
                        });
                    });
                }

                rounds
            })
            .collect()
    }
    fn result(&self, battles: Vec<BattleView>) -> Vec<SimulationView> {
        let mut balance: Vec<SimulationView> = vec![];
        let mut counters: HashMap<TeamView, HashMap<Group, HashMap<String, AvgCounter>>> =
            hashmap! {};
        let mut i = 0;
        battles.into_iter().for_each(|battle| {
            let group = if battle.group == SimulationGroup::Round {
                format!("{}: {:?}", battle.round.name, battle.round.enemies)
            } else {
                format!("{:?}", battle.round.enemies)
            };
            let units = counters
                .entry(format!("{:?}", battle.team))
                .or_insert(hashmap! {});
            let group = units.entry(group).or_insert(hashmap! {});
            let clans = group
                .entry(format!("{:?}", battle.clans))
                .or_insert(AvgCounter::new());
            if battle.win {
                clans.sum += 1.0;
            };
            clans.count += 1;
        });

        for (team, counters) in counters {
            let groups: BTreeMap<Group, ClansGroupView> = counters
                .iter()
                .map(|(key, value)| {
                    let clans: BTreeMap<String, f64> = value
                        .iter()
                        .map(|(key, value)| (key.clone(), value.avg()))
                        .collect();
                    (
                        key.to_string(),
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
                player: team,
                koef,
                groups,
            });
        }
        balance
    }
}
