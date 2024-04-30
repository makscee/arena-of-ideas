use std::time::UNIX_EPOCH;

use bevy::utils::hashbrown::HashMap;
use bevy_egui::egui::ScrollArea;
use chrono::DateTime;
use egui_extras::{Column, TableBuilder};
use ron::ser::{to_string_pretty, PrettyConfig};
use spacetimedb_sdk::on_subscription_applied;

use self::module_bindings::{ArenaRun, User};

use super::*;

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::load);
    }
}

impl LeaderboardPlugin {
    fn load() {
        info!("Leaderboard startup");

        ArenaRun::on_update(|_, _, _| OperationsPlugin::add(LeaderboardData::load));
        on_subscription_applied(|| OperationsPlugin::add(LeaderboardData::load));
    }

    pub fn ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };

        let columns = Columns::iter().collect_vec();
        let data = if let Some(data) = world.remove_resource::<LeaderboardData>() {
            data
        } else {
            return;
        };
        let mut new_round: Option<Option<usize>> = None;
        window("LEADERBOARD")
            .set_width(400.0)
            .order(egui::Order::Foreground)
            .anchor(Align2::RIGHT_TOP, [-15.0, 15.0])
            .show(ctx, |ui| {
                ScrollArea::new([false, true]).show(ui, |ui| {
                    frame(ui, |ui| {
                        TableBuilder::new(ui)
                            .columns(
                                Column::auto_with_initial_suggestion(50.0),
                                columns.len() + data.round.is_none() as usize,
                            )
                            .striped(true)
                            .header(20.0, |mut row| {
                                for col in columns.iter() {
                                    row.col(|ui| {
                                        col.header(ui);
                                    });
                                }
                            })
                            .body(|mut body| {
                                if let Some(round) = data.round {
                                    for run in &data.data[&round] {
                                        body.row(20.0, |mut row| {
                                            for col in columns.iter() {
                                                row.col(|ui| {
                                                    col.row(run, ui, world);
                                                });
                                            }
                                        });
                                    }
                                } else {
                                    for (_, run) in
                                        data.data.iter().sorted_by_key(|(k, _)| -(**k as i32))
                                    {
                                        body.row(20.0, |mut row| {
                                            for col in columns.iter() {
                                                row.col(|ui| {
                                                    col.row(&run[0], ui, world);
                                                });
                                            }
                                            row.col(|ui| {
                                                let cnt = run.len() - 1;
                                                ui.add_enabled_ui(cnt > 0, |ui| {
                                                    if egui::Button::new(format!(
                                                        "+{}",
                                                        run.len() - 1
                                                    ))
                                                    .wrap(false)
                                                    .ui(ui)
                                                    .clicked()
                                                    {
                                                        new_round =
                                                            Some(Some(run[0].round as usize));
                                                    }
                                                });
                                            });
                                        });
                                    }
                                }
                            });

                        if data.round.is_some() {
                            if ui.button("<-").clicked() {
                                new_round = Some(None);
                            }
                        }
                    });
                });
            });
        world.insert_resource(data);
        if let Some(round) = new_round {
            world.resource_mut::<LeaderboardData>().round = round;
        }
    }
}

#[derive(Resource, Default, Debug)]
struct LeaderboardData {
    data: HashMap<usize, Vec<ArenaRun>>,
    round: Option<usize>,
}

impl LeaderboardData {
    fn load(world: &mut World) {
        info!("Load Leaderboard");
        let mut data: HashMap<usize, Vec<ArenaRun>> = default();
        for run in ArenaRun::iter() {
            if run.active || run.round == 0 {
                continue;
            }
            let round = run.round as usize;
            data.entry(round).or_default().push(run);
        }
        for (_, list) in data.iter_mut() {
            list.sort_by_key(|r| r.last_updated)
        }
        world.insert_resource(LeaderboardData { data, round: None })
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Display)]
enum Columns {
    Round,
    Wins,
    Loses,
    Name,
    Team,
    Time,
}

impl Columns {
    fn header(&self, ui: &mut Ui) -> Response {
        self.to_string()
            .add_color(white())
            .as_label(ui)
            .selectable(false)
            .ui(ui)
    }
    fn row(&self, run: &ArenaRun, ui: &mut Ui, world: &mut World) -> Response {
        match self {
            Columns::Name => User::filter_by_id(run.user_id)
                .unwrap()
                .name
                .add_color(white())
                .label(ui),
            Columns::Team => {
                let mut str = ColoredString::default();
                for unit in run.state.team.iter().rev() {
                    let name = format!("{} ", unit.unit.name.split_at(3).0);
                    str.push_colored(
                        name.add_color(
                            Pools::get_color_by_name(&unit.unit.name, world)
                                .map(|c| c.c32())
                                .unwrap_or(white()),
                        ),
                    );
                }
                let resp = str.button(ui).on_hover_ui(|ui| {
                    for unit in run.state.team.iter().rev() {
                        let unit = &unit.unit;
                        ui.horizontal(|ui| {
                            unit.name
                                .add_color(
                                    Pools::get_color_by_name(&unit.name, world)
                                        .map(|c| c.c32())
                                        .unwrap_or(white()),
                                )
                                .label(ui);
                            format!("{}/{} lvl:{}", unit.pwr, unit.hp, unit.level)
                                .to_colored()
                                .label(ui);
                            for house in unit.houses.split("+") {
                                house
                                    .add_color(
                                        Pools::get_house_color(house, world)
                                            .map(|c| c.c32())
                                            .unwrap_or(light_gray()),
                                    )
                                    .label(ui);
                            }
                        });
                    }
                });
                if resp.clicked() {
                    save_to_clipboard(
                        &to_string_pretty(
                            &PackedTeam::from_table_units(
                                run.state
                                    .team
                                    .clone()
                                    .into_iter()
                                    .map(|u| u.unit)
                                    .collect_vec(),
                            ),
                            PrettyConfig::new(),
                        )
                        .unwrap(),
                        world,
                    )
                }
                resp
            }
            Columns::Round => run.round.to_string().add_color(white()).label(ui),
            Columns::Wins => run.wins().to_string().add_color(white()).label(ui),
            Columns::Loses => run.loses().to_string().add_color(red()).label(ui),
            Columns::Time => DateTime::<chrono::Local>::from(
                UNIX_EPOCH + Duration::from_micros(run.last_updated),
            )
            .format("%d/%m/%Y %H:%M")
            .to_string()
            .to_colored()
            .label(ui),
        }
    }
}
