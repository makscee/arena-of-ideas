use super::*;

pub struct SimulationSystem {}

impl SimulationSystem {
    pub fn run_ranked_battle(
        light: &PackedTeam,
        dark: &PackedTeam,
        world: &mut legion::World,
        resources: &mut Resources,
        assert: Option<&Condition>,
    ) -> usize {
        let mut dark = dark.clone();
        let mut score = 0;
        for _ in 0..3 {
            let (result, _) = Self::run_battle(light, &dark, world, resources, assert);
            if !result {
                break;
            }
            for unit in dark.units.iter_mut() {
                unit.rank += 1;
            }
            score += 1;
        }
        score
    }

    pub fn run_battle(
        light: &PackedTeam,
        dark: &PackedTeam,
        world: &mut legion::World,
        resources: &mut Resources,
        assert: Option<&Condition>,
    ) -> (bool, bool) {
        light.unpack(&Faction::Light, world, resources);
        dark.unpack(&Faction::Dark, world, resources);
        resources.logger.log(
            || format!("Run simulation: {light} {dark}"),
            &LogContext::Test,
        );
        let mut win = BattleSystem::run_battle(world, resources, &mut None) > 0;
        match assert {
            Some(condition) => {
                let context = &Context::new(
                    ContextLayer::Entity {
                        entity: WorldSystem::entity(world),
                    },
                    world,
                    resources,
                );
                let result = condition.calculate(context, world, resources).unwrap();
                win = result;
                if !result {
                    let light = PackedTeam::pack(Faction::Light, world, resources);
                    let dark = PackedTeam::pack(Faction::Dark, world, resources);
                    println!("Light: {light}\nDark : {dark}");
                }
            }
            None => {}
        };
        let beat = !UnitSystem::collect_faction(world, Faction::Light).is_empty();
        BattleSystem::clear_world(world, resources);
        resources.action_queue.clear();
        (win, beat)
    }
}

#[cfg(test)]
mod tests {
    use colored::Colorize;
    use geng::prelude::file::load_json;

    use super::*;

    #[derive(Deserialize)]
    struct TestScenario {
        light: PackedTeam,
        dark: PackedTeam,
        assert: Condition,
    }

    fn setup() -> (legion::World, Resources) {
        let mut world = legion::World::default();
        let mut resources = Resources::new(Options::do_load());
        let watcher = &mut FileWatcherSystem::new();
        resources.load(watcher);
        resources
            .logger
            .set_enabled(resources.logger.is_context_enabled(&LogContext::Test));
        Game::init_world(&mut resources, &mut world);
        (world, resources)
    }

    #[test]
    fn test_simple() {
        setup();
        let (mut world, mut resources) = setup();
        let unit = PackedUnit {
            name: "test".to_string(),
            description: "test".to_string(),
            health: 1,
            damage: 0,
            attack: 1,
            house: default(),
            trigger: default(),
            statuses: default(),
            shader: default(),
            rank: default(),
        };
        let light = PackedTeam::new(String::from("light"), vec![unit.clone()]);
        let (win, _) =
            SimulationSystem::run_battle(&light, &light, &mut world, &mut resources, None);
        assert!(win)
    }

    #[test]
    fn test_scenarios() {
        println!("Start scenarios");
        let (mut world, mut resources) = setup();
        let paths: Vec<PathBuf> =
            futures::executor::block_on(load_json(static_path().join("test/scenarios/_list.json")))
                .unwrap();

        let scenarios: Vec<(PathBuf, TestScenario)> = paths
            .into_iter()
            .map(|x| {
                println!("Load scenario {:?}", x);
                (
                    x.clone(),
                    futures::executor::block_on(load_json(static_path().join(x))).unwrap(),
                )
            })
            .collect_vec();
        assert!(!scenarios.is_empty());
        for (path, scenario) in scenarios.iter() {
            let text = format!("Run scenario: {:?}...", path.file_name().unwrap()).on_blue();
            println!("\n{text}\n");
            let (win, _) = SimulationSystem::run_battle(
                &scenario.light,
                &scenario.dark,
                &mut world,
                &mut resources,
                Some(&scenario.assert),
            );
            assert!(
                win,
                "Scenario {:?} failed assert: {:?}",
                path, scenario.assert
            );
        }
        println!("Scenarios:");
        scenarios
            .iter()
            .rev()
            .for_each(|(path, _)| println!("{}", path.file_name().unwrap().to_string_lossy()));
    }
}
