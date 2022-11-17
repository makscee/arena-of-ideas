use crate::assets::Sounds;

use super::*;
#[derive(clap::Args)]
pub struct Simulate {
    config_path: PathBuf,
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
