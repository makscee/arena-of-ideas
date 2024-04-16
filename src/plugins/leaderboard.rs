use std::time::UNIX_EPOCH;

use bevy::utils::hashbrown::HashMap;
use chrono::DateTime;
use egui_extras::{Column, TableBuilder};

use self::module_bindings::{ArenaRun, User};

use super::*;

pub struct LeaderboardPlugin;

impl LeaderboardPlugin {
    pub fn load(world: &mut World) {
        LeaderboardData::load(world);
    }

    pub fn ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };

        let columns = Columns::iter().collect_vec();
        let (data, round) = if let Some(data) = world.get_resource::<LeaderboardData>() {
            (&data.data, data.round)
        } else {
            return;
        };
        let mut new_round: Option<Option<usize>> = None;
        window("LEADERBOARD")
            .set_width(400.0)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                frame(ui, |ui| {
                    TableBuilder::new(ui)
                        .columns(
                            Column::auto_with_initial_suggestion(50.0),
                            columns.len() + round.is_none() as usize,
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
                            if let Some(round) = round {
                                for run in &data[&round] {
                                    body.row(20.0, |mut row| {
                                        for col in columns.iter() {
                                            row.col(|ui| {
                                                col.row(run, ui, world);
                                            });
                                        }
                                    });
                                }
                            } else {
                                for (_, run) in data.iter().sorted_by_key(|(k, _)| -(**k as i32)) {
                                    body.row(20.0, |mut row| {
                                        for col in columns.iter() {
                                            row.col(|ui| {
                                                col.row(&run[0], ui, world);
                                            });
                                        }
                                        row.col(|ui| {
                                            let cnt = run.len() - 1;
                                            ui.add_enabled_ui(cnt > 0, |ui| {
                                                if ui
                                                    .button(format!("+{}", run.len() - 1))
                                                    .clicked()
                                                {
                                                    new_round = Some(Some(run[0].wins as usize));
                                                }
                                            });
                                        });
                                    });
                                }
                            }
                        });

                    if round.is_some() {
                        if ui.button("<-").clicked() {
                            new_round = Some(None);
                        }
                    }
                });
            });
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
            let round = run.wins as usize;
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
    fn row(&self, run: &ArenaRun, ui: &mut Ui, world: &World) -> Response {
        match self {
            Columns::Name => User::filter_by_id(run.user_id)
                .unwrap()
                .name
                .add_color(white())
                .as_label(ui),
            Columns::Team => {
                let mut str = ColoredString::default();
                for unit in run.state.team.iter().rev() {
                    let name = format!("{} ", unit.unit.name);
                    str.push_colored(
                        name.add_color(
                            Pools::get_color_by_name(&unit.unit.name, world)
                                .map(|c| c.c32())
                                .unwrap_or(light_gray()),
                        ),
                    );
                }
                str.as_label(ui)
            }
            Columns::Wins => run.wins.to_string().add_color(white()).as_label(ui),
            Columns::Loses => run.loses.to_string().add_color(red()).as_label(ui),
            Columns::Time => DateTime::<chrono::Local>::from(
                UNIX_EPOCH + Duration::from_micros(run.last_updated),
            )
            .format("%d/%m/%Y %H:%M")
            .to_string()
            .to_colored()
            .as_label(ui),
        }
        .wrap(false)
        .ui(ui)
    }
}
