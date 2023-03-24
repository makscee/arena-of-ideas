use super::*;

pub struct SimulationSystem {}

impl SimulationSystem {
    pub fn run_battle(
        light: &Vec<&PackedUnit>,
        dark: &Vec<&PackedUnit>,
        world: &mut legion::World,
        resources: &mut Resources,
        assert: Option<&Condition>,
    ) -> bool {
        light.iter().enumerate().for_each(|(slot, unit)| {
            unit.unpack(world, resources, slot + 1, Faction::Light, None);
        });
        dark.iter().enumerate().for_each(|(slot, unit)| {
            unit.unpack(world, resources, slot + 1, Faction::Dark, None);
        });
        ActionSystem::run_ticks(world, resources, &mut None);
        BattleSystem::run_battle(world, resources, &mut None);
        let result = match assert {
            Some(condition) => {
                let result = condition
                    .calculate(&WorldSystem::get_context(world), world, resources)
                    .unwrap();
                if !result {
                    let light = Team::pack_entries(&Faction::Light, world, resources);
                    let dark = Team::pack_entries(&Faction::Dark, world, resources);
                    dbg!((light, dark));
                }
                result
            }
            None => BattleSystem::battle_won(world),
        };
        BattleSystem::clear_world(world, resources);
        result
    }

    pub fn run_team_battle(
        light: &Team,
        dark: &Team,
        world: &mut legion::World,
        resources: &mut Resources,
        assert: Option<&Condition>,
    ) -> bool {
        let light: Vec<&PackedUnit> = light.units.iter().map(|x| x).collect_vec();
        let dark: Vec<&PackedUnit> = dark.units.iter().map(|x| x).collect_vec();
        Self::run_battle(&light, &dark, world, resources, assert)
    }
}

#[cfg(test)]
mod tests {
    use geng::prelude::file::load_json;

    use super::*;

    #[derive(Deserialize)]
    struct TestScenario {
        light: Team,
        dark: Team,
        assert: Condition,
    }

    fn setup() -> (legion::World, Resources) {
        let mut world = legion::World::default();
        let mut resources = Resources::new(Options::load());
        let watcher = &mut FileWatcherSystem::new();
        resources.load(watcher);
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
            houses: default(),
            trigger: default(),
            active_statuses: default(),
            shader: default(),
        };
        let light = vec![&unit];
        let dark = vec![&unit];
        assert!(SimulationSystem::run_battle(
            &light,
            &dark,
            &mut world,
            &mut resources,
            None
        ))
    }

    #[test]
    fn test_scenarios() {
        let (mut world, mut resources) = setup();
        let paths: Vec<PathBuf> =
            futures::executor::block_on(load_json(static_path().join("test/scenarios/_list.json")))
                .unwrap();

        let scenarios: Vec<(PathBuf, TestScenario)> = paths
            .into_iter()
            .map(|x| {
                (
                    x.clone(),
                    futures::executor::block_on(load_json(static_path().join(x))).unwrap(),
                )
            })
            .collect_vec();
        assert!(!scenarios.is_empty());
        for (path, scenario) in scenarios {
            println!("Run scenario: {:?}...", path.file_name().unwrap());
            assert!(
                SimulationSystem::run_team_battle(
                    &scenario.light,
                    &scenario.dark,
                    &mut world,
                    &mut resources,
                    Some(&scenario.assert),
                ),
                "Scenario {:?} failed assert: {:?}",
                path,
                scenario.assert
            )
        }
    }
}
