use super::*;
use crate::simulation::simulation_config::RegexUnit;
use crate::simulation::simulation::match_units;

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
}

impl SimulationVariant for UnitsSimulation {
    fn battles(&self) -> Vec<BattleConfig> {
        let mut player_variants = vec![];
        player_variants = match_units(&self.all_units, &self.squad, 0, player_variants);

        let mut unit_vars = vec![];
        unit_vars = match_units(&self.all_units, &self.enemies, 0, unit_vars);

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

        player_variants
            .into_iter()
            .flat_map(|player| {
                let mut rounds = vec![];
                for round in &game_rounds {
                    self.clan_size.clone().into_iter().for_each(|i| {
                        let unit_clans: HashSet<Clan> = player
                            .clone()
                            .into_iter()
                            .flat_map(|unit| unit.clans)
                            .collect();
                        unit_clans.clone().into_iter().for_each(|clan| {
                            let enemy_clans: HashSet<Clan> = round
                                .enemies
                                .clone()
                                .into_iter()
                                .flat_map(|enemy| {
                                    let enemy = &self
                                        .all_units
                                        .clone()
                                        .into_iter()
                                        .find(|unit| unit.name == enemy)
                                        .unwrap();
                                    enemy.clans.clone()
                                })
                                .collect();
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
