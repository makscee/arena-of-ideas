use crate::model::UnitTemplate;
use crate::simulation::balance_simulation::BalanceSimulation;
use crate::simulation::round_simulation::RoundSimulation;
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

use super::*;

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
            SimulationType::Balance { unit, repeats } => Box::new(BalanceSimulation::new(
                unit,
                repeats,
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
                            units_alive: result
                                .units_alive
                                .into_iter()
                                .map(|unit| unit.unit_type)
                                .collect(),
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
