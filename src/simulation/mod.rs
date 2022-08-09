mod balance_simulation;
mod battle;
mod round_simulation;
mod simulation;
mod simulation_config;
mod units_simulation;

use crate::simulation::simulation::Simulation;
use crate::simulation::simulation::SimulationResult;
use battle::Battle;
use geng::prelude::itertools::Itertools;
use simulation_config::SimulationConfig;
use std::time::Instant;
use std::{collections::BTreeMap, collections::VecDeque, path::PathBuf};

use crate::{
    assets::{self, Assets, ClanEffects, Config, GameRound, KeyMapping, Statuses},
    logic::{Events, Logic},
    model::MAX_TIER,
    model::{Faction, Model, Unit, UnitTemplate, UnitTemplates, UnitType},
    render::RenderModel,
    Clan,
};
use geng::prelude::*;

#[derive(clap::Args)]
pub struct Simulate {
    config_path: PathBuf,
}

trait SimulationVariant {
    fn result(&self, battles: Vec<BattleView>) -> Vec<SimulationView>;
    fn battles(&self) -> Vec<BattleConfig>;
}

#[derive(Debug, Deserialize, Clone)]
struct BattleConfig {
    unit: Option<UnitType>,
    player: Vec<UnitType>,
    round: GameRound,
    repeats: usize,
    clans: HashMap<Clan, usize>,
    enemy_clans: HashMap<Clan, usize>,
    group: SimulationGroup,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum SimulationGroup {
    SameTier,
    UpperTier,
    LowerTier,
    Round,
    Enemies,
}

impl fmt::Display for SimulationGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SimulationGroup::SameTier => write!(f, "SameTier"),
            SimulationGroup::UpperTier => write!(f, "UpperTier"),
            SimulationGroup::LowerTier => write!(f, "LowerTier"),
            SimulationGroup::Round => write!(f, "Round"),
            SimulationGroup::Enemies => write!(f, "Enemies"),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
struct BattleView {
    unit: Option<UnitType>,
    team: Vec<UnitType>,
    round: GameRound,
    clans: HashMap<Clan, usize>,
    enemy_clans: HashMap<Clan, usize>,
    group: SimulationGroup,
    win: bool,
    units_alive: Vec<UnitType>,
}

type Group = String;
type TeamView = String;

#[derive(Debug, Serialize, Clone)]
pub struct SimulationView {
    player: TeamView,
    koef: f64,
    groups: BTreeMap<Group, ClansGroupView>,
}

#[derive(Debug, Serialize, Clone)]
struct ClansGroupView {
    koef: f64,
    clans: BTreeMap<String, f64>,
}

struct AvgCounter {
    count: i64,
    sum: f64,
}

pub struct ProgressTracker {
    pub simulations_remains: (usize, usize),
    pub battles_remains: (usize, usize),
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            simulations_remains: (1, 0),
            battles_remains: (1, 0),
        }
    }
    pub fn log_progress(&self) {
        info!(
            "Simulations: {}/{} Battles: {}/{}",
            self.simulations_remains.0,
            self.simulations_remains.1,
            self.battles_remains.0,
            self.battles_remains.1
        );
    }
}

impl AvgCounter {
    pub fn new() -> Self {
        Self { count: 0, sum: 0.0 }
    }
    pub fn avg(&self) -> f64 {
        self.sum / (self.count as f64)
    }
}

impl Simulate {
    pub fn run(self, geng: &Geng, assets: Assets, mut config: Config) {
        let start = Instant::now();
        let config_path = static_path().join(self.config_path);
        let simulation_config = futures::executor::block_on(
            <SimulationConfig as geng::LoadAsset>::load(geng, &config_path),
        )
        .unwrap();
        info!("Starting simulation");

        let all_units: Vec<UnitTemplate> =
            assets.units.iter().map(|entry| entry.1).cloned().collect();
        let all_clans: Vec<Clan> = assets.clans.map.iter().map(|(k, v)| k.clone()).collect();
        let mut progress = ProgressTracker::new();
        progress.simulations_remains.1 = simulation_config.simulations.len();
        let simulation_results: Vec<SimulationResult> = simulation_config
            .simulations
            .into_iter()
            .map(|simulation_type| {
                let simulation = Simulation::new(
                    &mut progress,
                    config.clone(),
                    assets.clans.clone(),
                    assets.statuses.clone(),
                    assets.units.clone(),
                    simulation_type,
                    assets.rounds.clone(),
                    all_units.clone(),
                    all_clans.clone(),
                );
                simulation.run()
            })
            .collect();

        info!("Simulation ended: {:?}", start.elapsed());
        let result_path = PathBuf::new().join("simulation_result");
        let date_path = result_path.join(format!("{:?}", chrono::offset::Utc::now()));

        // Create directories
        match std::fs::create_dir_all(&result_path) {
            Ok(()) => {}
            Err(error) => match error.kind() {
                std::io::ErrorKind::AlreadyExists => {}
                _ => panic!("Failed to create a simulation_result directory: {error}"),
            },
        }
        match std::fs::create_dir_all(&date_path) {
            Ok(()) => {}
            Err(error) => match error.kind() {
                std::io::ErrorKind::AlreadyExists => {}
                _ => panic!("Failed to create a simulation_result directory: {error}"),
            },
        }
        let koef = simulation_results
            .clone()
            .into_iter()
            .map(|value| value.koef)
            .sum::<f64>()
            / simulation_results.len() as f64;

        // Write results
        write_to(date_path.join("result.json"), &(koef, &simulation_results))
            .expect("Failed to write results");

        info!("Results saved: {:?}", start.elapsed());
    }
}

fn write_to<T: Serialize>(path: impl AsRef<std::path::Path>, item: &T) -> std::io::Result<()> {
    let path = path.as_ref();
    let file = std::fs::File::create(path).expect(&format!("Failed to create {path:?}"));
    let data = serde_json::to_string_pretty(item).expect("Failed to serialize item");
    std::fs::write(path, data)?;
    Ok(())
}
