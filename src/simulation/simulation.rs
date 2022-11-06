use crate::model::UnitTemplate;
use crate::simulation::balance_simulation::BalanceSimulation;
use crate::simulation::round_simulation::RoundSimulation;
use crate::simulation::simulation_config::RegexUnit;
use crate::simulation::simulation_config::SimulationType;
use crate::simulation::units_simulation::UnitsSimulation;
use crate::simulation::Battle;
use crate::simulation::BattleConfig;
use crate::simulation::BattleView;
use crate::simulation::SimulationVariant;
use crate::Assets;
use crate::Clan;
use crate::Config;
use crate::GameRound;
use crate::MAX_LIVES;

pub use super::*;

pub struct Simulation<'a> {
    progress: &'a mut ProgressTracker,
    variant: Box<dyn SimulationVariant>,
    config: Config,
    clan_effects: ClanEffects,
    statuses: Statuses,
    units: UnitTemplates,
}

impl<'a> Simulation<'a> {
    pub fn new(
        progress: &'a mut ProgressTracker,
        config: Config,
        clan_effects: ClanEffects,
        statuses: Statuses,
        units: UnitTemplates,
        simulation_type: SimulationType,
        rounds: Vec<GameRound>,
        all_units: Vec<UnitTemplate>,
        all_clans: Vec<Clan>,
    ) -> Self {
        let variant: Box<dyn SimulationVariant> = match simulation_type {
            SimulationType::Balance { unit, repeats, tier } => Box::new(BalanceSimulation::new(
                unit,
                repeats,
                tier,
                all_units,
                all_clans,
                config.clone(),
            )),
            SimulationType::Units {
                squad,
                enemies,
                repeats,
                clan_bonuses,
            } => Box::new(UnitsSimulation::new(
                squad,
                enemies,
                repeats,
                clan_bonuses,
                all_units,
                all_clans,
                config.clone(),
            )),
            SimulationType::Rounds {
                squad,
                from,
                to,
                repeats,
                clan_bonuses,
            } => Box::new(RoundSimulation::new(
                squad,
                clan_bonuses,
                rounds.iter().take(to).skip(from - 1).cloned().collect(),
                repeats,
                all_units,
                all_clans,
                config.clone(),
            )),
        };
        Simulation {
            progress,
            variant,
            config,
            clan_effects,
            statuses,
            units,
        }
    }

    pub fn run(mut self) -> SimulationResult {
        let battles = self.variant.battles();
        self.progress.battles_remains.1 = battles.len();
        self.progress.battles_remains.0 = 0;
        let battle_views: Vec<BattleView> = battles
            .into_iter()
            .flat_map(|battle| {
                let results: Vec<BattleView> = (1..=battle.repeats)
                    .map(|i| {
                        let result = Battle::new(
                            Config {
                                player: battle.player.clone(),
                                clans: battle.clans.clone(),
                                enemy_clans: battle.enemy_clans.clone(),
                                ..self.config.clone()
                            },
                            self.clan_effects.clone(),
                            self.statuses.clone(),
                            battle.round.clone(),
                            self.units.clone(),
                            0.02 as f64,
                            MAX_LIVES,
                        )
                        .run();
                        BattleView {
                            unit: battle.unit.clone(),
                            team: battle.player.clone(),
                            round: battle.round.clone(),
                            clans: battle.clans.clone(),
                            enemy_clans: battle.enemy_clans.clone(),
                            group: battle.group.clone(),
                            win: result.player_won,
                            units_alive: result.units_alive.clone(),
                        }
                    })
                    .collect();

                self.progress.battles_remains.0 += 1;
                self.progress.log_progress();
                results
            })
            .collect();
        let mut simulations = self.variant.result(battle_views);
        simulations.sort_by(|a, b| b.koef.partial_cmp(&a.koef).unwrap());
        let len = simulations.len() as f64;
        let koef = simulations
            .clone()
            .into_iter()
            .map(|value| value.koef)
            .sum::<f64>()
            / len;
        SimulationResult {
            koef,
            results: simulations.clone(),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct SimulationResult {
    pub koef: f64,
    pub results: Vec<SimulationView>,
}

pub fn to_templates(unit: RegexUnit, all_units: &Vec<UnitTemplate>) -> Vec<UnitTemplate> {
    let regex = regex::Regex::new(&unit).expect("Failed to parse a regular expression");
    all_units
        .iter()
        .filter(move |unit| regex.is_match(&unit.long_name))
        .cloned()
        .collect()
}

pub fn match_units(
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

    let regex_units = to_templates(units[index].clone(), all_units);
    let mut regex_peek = regex_units.into_iter().peekable();
    while let Some(unit) = regex_peek.next() {
        let mut last_index = cloned.len() - 1;
        cloned[last_index].push(unit);
        cloned = match_units(all_units, units, index + 1, cloned);
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
