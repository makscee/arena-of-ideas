use colored::{ColoredString, Colorize};

use super::*;

pub struct TestScenariosPlugin;

#[derive(Debug, AssetCollection, Resource)]
pub struct TestScenarios {
    #[asset(key = "test_scenarios", collection(typed, mapped))]
    pub handles: HashMap<String, Handle<TestScenario>>,
}

#[derive(Asset, Deserialize, TypePath, Debug, Clone)]
pub struct TestScenario {
    pub left: PackedTeam,
    pub right: PackedTeam,
    #[serde(default = "even")]
    pub result: BattleResult,
}

fn even() -> BattleResult {
    BattleResult::Even
}

impl Plugin for TestScenariosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::TestScenariosRun), Self::run);
    }
}

impl TestScenariosPlugin {
    fn run(world: &mut World) {
        let mut scenarios = Self::get_all_scenarios(world);
        scenarios.sort_by_key(|(s, _)| s.clone());
        if let Some(path) = &ARGS.get().unwrap().extra {
            scenarios = scenarios
                .into_iter()
                .find(|(p, _)| path.eq(p))
                .into_iter()
                .collect_vec();
        }
        let mut failure: Vec<ColoredString> = default();
        let mut success: Vec<ColoredString> = default();
        let path_color = colored::Color::TrueColor {
            r: 50,
            g: 50,
            b: 50,
        };
        for (path, scenario) in scenarios {
            match scenario.run(world) {
                Ok(_) => info!("Test run {}", {
                    let str = format!("{} {}", "Success".bold(), path.color(path_color)).green();
                    success.push(str.clone());
                    str
                }),
                Err(err) => {
                    let str =
                        format!("{} {}\n{}", "Error".bold(), path.color(path_color), err).red();
                    failure.push(str.clone());
                    info!("Test fail: {}", str.clone())
                }
            }
        }
        info!(
            "Tests run complete with {} successes and {} failures.\n{}\n{}",
            success.len(),
            failure.len(),
            success.into_iter().join("\n"),
            failure.into_iter().join("\n"),
        );
        app_exit(world);
    }
    fn get_all_scenarios(world: &World) -> Vec<(String, TestScenario)> {
        world
            .get_resource::<TestScenarios>()
            .unwrap()
            .handles
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

impl TestScenario {
    fn run(self, world: &mut World) -> Result<()> {
        gt().reset();
        BattlePlugin::load_teams(self.left, self.right, world);
        let r = match BattlePlugin::run(world) {
            Ok(result) => {
                if !result.eq(&self.result) {
                    Err(anyhow!("Expected {} got {result}", self.result))
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(anyhow!("{e}")),
        };
        BattlePlugin::clear(world);
        r
    }
}
