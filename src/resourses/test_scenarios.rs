use colored::ColoredString;

use super::*;

#[derive(Debug, AssetCollection, Resource)]
pub struct TestScenarios {
    #[asset(key = "test.scenarios", collection(typed, mapped))]
    pub tests_handles: HashMap<String, Handle<TestScenario>>,
}

#[derive(Deserialize, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "d112970f-9d3f-412d-b7a3-25db4f52c6b8"]
pub struct TestScenario {
    pub left: PackedTeam,
    pub right: PackedTeam,
    pub condition: Expression,
}

impl TestScenario {
    pub fn run(self, world: &mut World) -> Result<bool> {
        SimulationPlugin::run(self.left, self.right, world);
        self.condition.get_bool(&Context::default(), world)
    }
}

pub struct TestPlugin;

impl Plugin for TestPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::BattleTest), Self::run_tests);
    }
}

impl TestPlugin {
    pub fn run_tests(world: &mut World) {
        let scenarios = Self::get_all_scenarios(world);
        let mut failure: Vec<ColoredString> = default();
        let mut success: Vec<ColoredString> = default();
        for (name, scenario) in scenarios {
            match scenario.run(world) {
                Ok(value) => debug!(
                    "Test run {}",
                    match value {
                        true => {
                            let str = format!("{} {}", "Success".bold(), name.dimmed()).green();
                            success.push(str.clone());
                            str
                        }
                        false => {
                            let str = format!("{} {}", "Failure".bold(), name).red();
                            failure.push(str.clone());
                            str
                        }
                    }
                ),
                Err(err) => {
                    let str = format!("Error {err}").red().bold();
                    failure.push(str.clone());
                    debug!("Test fail: {}", str.clone())
                }
            }
        }
        debug!(
            "Tests run complete with {} successes and {} failures.\n{}\n{}",
            success.len(),
            failure.len(),
            success.into_iter().join("\n"),
            failure.into_iter().join("\n"),
        );
    }

    pub fn get_all_scenarios(world: &World) -> Vec<(String, TestScenario)> {
        world
            .get_resource::<TestScenarios>()
            .unwrap()
            .tests_handles
            .clone()
            .into_iter()
            .map(|(name, handle)| {
                (
                    name,
                    world
                        .get_resource::<Assets<TestScenario>>()
                        .unwrap()
                        .get(&handle)
                        .unwrap()
                        .clone(),
                )
            })
            .collect_vec()
    }
}
