use bevy::utils::hashbrown::HashMap;
use bevy_egui::egui::ScrollArea;
use egui_extras::{Column, TableBuilder};
use ron::ser::{to_string_pretty, PrettyConfig};

use crate::module_bindings::{ArenaArchive, User};

use super::*;

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::load)
            .add_systems(OnEnter(GameState::MainMenu), Self::open)
            .init_resource::<LeaderboardData>();
    }
}

impl LeaderboardPlugin {
    fn load() {
        info!("Leaderboard startup");
        ArenaArchive::on_insert(|run, _| {
            let run = run.clone();
            OperationsPlugin::add(|world| LeaderboardData::load(run, world));
        });
    }

    fn open(world: &mut World) {
        TopButton::Leaderboard.click(world);
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
        if data.data.is_empty() {
            world.insert_resource(data);
            return;
        }
        let mut new_round: Option<Option<usize>> = None;
        window("LEADERBOARD")
            .set_width(400.0)
            .anchor(Align2::RIGHT_TOP, [-15.0, 15.0])
            .show(ctx, |ui| {
                let mut iter = data.data.iter().sorted_by_key(|(k, _)| -(**k as i32));
                if data.round.is_none() {
                    if let Some((round, data)) = iter.next() {
                        frame(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                let run = &data[0];
                                "Current leader".to_colored().label(ui);
                                show_name(run, ColoredStringStyle::Heading, ui, world);
                                format!("Round #{}", round).add_color(orange()).label(ui);
                                show_team(run, false, ui, world);
                                draw_plus(data, &mut new_round, ui);
                            })
                        });
                    }
                }
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
                                    for (_, run) in iter {
                                        body.row(20.0, |mut row| {
                                            for col in columns.iter() {
                                                row.col(|ui| {
                                                    col.row(&run[0], ui, world);
                                                });
                                            }
                                            row.col(|ui| {
                                                draw_plus(run, &mut new_round, ui);
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

fn draw_plus(runs: &Vec<ArenaArchive>, new_round: &mut Option<Option<usize>>, ui: &mut Ui) {
    let cnt = runs.len() - 1;
    ui.add_enabled_ui(cnt > 0, |ui| {
        if egui::Button::new(format!("+{}", runs.len() - 1))
            .wrap(false)
            .ui(ui)
            .clicked()
        {
            *new_round = Some(Some(runs[0].round as usize));
        }
    });
}

#[derive(Resource, Default, Debug)]
struct LeaderboardData {
    data: HashMap<usize, Vec<ArenaArchive>>,
    round: Option<usize>,
}

impl LeaderboardData {
    fn load(run: ArenaArchive, world: &mut World) {
        info!("Load Leaderboard");
        let mut ld = world.resource_mut::<LeaderboardData>();
        let round = run.round as usize;
        let entry = ld.data.entry(round).or_default();
        entry.push(run);
        entry.sort_by_key(|r| r.timestamp);
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

fn show_name(
    run: &ArenaArchive,
    style: ColoredStringStyle,
    ui: &mut Ui,
    world: &mut World,
) -> Response {
    let resp = User::filter_by_id(run.user_id)
        .unwrap()
        .name
        .to_colored()
        .set_style(style)
        .as_button_uncolored(ui)
        .frame(false)
        .ui(ui);
    if resp.clicked() {
        ProfilePlugin::open_player_profile(run.user_id, world);
    }
    resp
}

fn show_team(run: &ArenaArchive, short: bool, ui: &mut Ui, world: &mut World) -> Response {
    let mut str = ColoredString::default();
    for unit in run.team.iter().rev() {
        let name = format!(
            "{} ",
            if short {
                unit.name.split_at(3).0
            } else {
                &unit.name
            }
        );
        str.push_colored(
            name.add_color(
                Pools::get_color_by_name(&unit.name, world)
                    .map(|c| c.c32())
                    .unwrap_or(white()),
            ),
        );
    }
    let resp = str.button(ui).on_hover_ui(|ui| {
        for unit in run.team.iter().rev() {
            ui.horizontal(|ui| {
                unit.name
                    .add_color(
                        Pools::get_color_by_name(&unit.name, world)
                            .map(|c| c.c32())
                            .unwrap_or(white()),
                    )
                    .label(ui);
                format!(
                    "{}/{} lvl:{}",
                    unit.pwr,
                    unit.hp,
                    PackedUnit::level_from_stacks(unit.stacks).0
                )
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
                &PackedTeam::from_table_units(run.team.clone()),
                PrettyConfig::new(),
            )
            .unwrap(),
            world,
        )
    }
    resp
}

impl Columns {
    fn header(&self, ui: &mut Ui) -> Response {
        self.to_string()
            .add_color(white())
            .as_label(ui)
            .selectable(false)
            .ui(ui)
    }
    fn row(&self, run: &ArenaArchive, ui: &mut Ui, world: &mut World) -> Response {
        match self {
            Columns::Name => show_name(run, ColoredStringStyle::Normal, ui, world),
            Columns::Team => show_team(run, true, ui, world),
            Columns::Round => run.round.to_string().add_color(white()).label(ui),
            Columns::Wins => run.wins.to_string().add_color(white()).label(ui),
            Columns::Loses => run.loses.to_string().add_color(red()).label(ui),
            Columns::Time => format_timestamp(run.timestamp).to_colored().label(ui),
        }
    }
}
