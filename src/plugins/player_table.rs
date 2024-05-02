use bevy_egui::egui::ScrollArea;
use convert_case::Casing;
use egui_extras::{Column, TableBuilder};

use crate::module_bindings::User;

use super::*;

pub struct PlayerTablePlugin;

impl Plugin for PlayerTablePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerTableCache>();
    }
}

impl PlayerTablePlugin {
    pub fn load(world: &mut World) {
        PlayerTableCache::load(world);
    }

    pub fn ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };

        let columns = Columns::iter().collect_vec();
        let mut data = if let Some(data) = world.remove_resource::<PlayerTableCache>() {
            data
        } else {
            return;
        };

        window("PLAYERS TABLE").set_width(400.0).show(ctx, |ui| {
            frame(ui, |ui| {
                ScrollArea::new([false, true]).show(ui, |ui| {
                    TableBuilder::new(ui)
                        .columns(Column::auto_with_initial_suggestion(100.0), columns.len())
                        .striped(true)
                        .header(20.0, |mut h| {
                            let mut do_sort = false;
                            for column in columns.iter() {
                                h.col(|ui| {
                                    if ui
                                        .button(
                                            column.to_string().to_case(convert_case::Case::Title),
                                        )
                                        .clicked()
                                    {
                                        do_sort = true;
                                        data.sorting = Some((
                                            *column,
                                            if data.sorting.as_ref().is_some_and(|(c, s)| {
                                                matches!(s, Sorting::Asc) && c.eq(column)
                                            }) {
                                                Sorting::Desc
                                            } else {
                                                Sorting::Asc
                                            },
                                        ));
                                    }
                                });
                            }
                            if do_sort {
                                match data.sorting.as_ref().unwrap().0 {
                                    Columns::Name => {
                                        data.players.sort_by_key(|d| d.name.to_owned())
                                    }
                                    Columns::MaxRound => {
                                        data.players.sort_by_key(|d| d.data.max_round)
                                    }
                                    Columns::TotalRuns => {
                                        data.players.sort_by_key(|d| d.data.total_runs)
                                    }
                                    Columns::WinRate => data.players.sort_by(|a, b| {
                                        a.data.win_rate.total_cmp(&b.data.win_rate)
                                    }),
                                    Columns::Wins => {
                                        data.players.sort_by_cached_key(|d| d.data.total_wins)
                                    }
                                    Columns::Loses => {
                                        data.players.sort_by_cached_key(|d| d.data.total_loses)
                                    }
                                    Columns::Online => data.players.sort_by_cached_key(|d| {
                                        if d.online {
                                            "999".to_owned()
                                        } else {
                                            d.last_login.clone()
                                        }
                                    }),
                                }
                                if data.sorting.as_ref().unwrap().1 == Sorting::Desc {
                                    data.players.reverse();
                                }
                            }
                        })
                        .body(|mut body| {
                            for player in &data.players {
                                body.row(20.0, |mut row| {
                                    for col in columns.iter() {
                                        row.col(|ui| {
                                            col.row(player, ui, world);
                                        });
                                    }
                                })
                            }
                        })
                })
            })
        });
        world.insert_resource(data);
    }
}

#[derive(Resource, Default)]
struct PlayerTableCache {
    players: Vec<PlayerTableData>,
    sorting: Option<(Columns, Sorting)>,
}

struct PlayerTableData {
    name: String,
    id: u64,
    online: bool,
    last_login: String,
    data: PlayerProfileData,
}

impl PlayerTableCache {
    fn load(world: &mut World) {
        let mut utc = world.resource_mut::<PlayerTableCache>();
        utc.players.clear();
        for user in User::iter().sorted_by(|a, b| a.id.cmp(&b.id)) {
            utc.players.push(PlayerTableData {
                name: user.name,
                id: user.id,
                online: user.online,
                last_login: format_timestamp(user.last_login),
                data: PlayerProfileData::from_id(user.id),
            });
        }
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Display)]
enum Columns {
    Name,
    MaxRound,
    TotalRuns,
    WinRate,
    Wins,
    Loses,
    Online,
}

#[derive(Serialize, Deserialize, Clone, Debug, EnumIter, PartialEq)]
enum Sorting {
    Asc,
    Desc,
}

impl Columns {
    fn header(&self, ui: &mut Ui) -> Response {
        self.to_string()
            .add_color(white())
            .as_label(ui)
            .selectable(false)
            .ui(ui)
    }
    fn row(&self, player: &PlayerTableData, ui: &mut Ui, world: &mut World) {
        match self {
            Columns::Name => {
                if ui.button(&player.name).clicked() {
                    ProfilePlugin::open_player_profile(player.id, world);
                }
            }
            Columns::MaxRound => {
                player
                    .data
                    .max_round
                    .to_string()
                    .add_color(white())
                    .label(ui);
            }
            Columns::TotalRuns => {
                player
                    .data
                    .max_round
                    .to_string()
                    .add_color(white())
                    .label(ui);
            }
            Columns::WinRate => {
                player
                    .data
                    .win_rate
                    .to_string()
                    .add_color(white())
                    .label(ui);
            }
            Columns::Wins => {
                player
                    .data
                    .total_wins
                    .to_string()
                    .add_color(white())
                    .label(ui);
            }
            Columns::Loses => {
                player
                    .data
                    .total_loses
                    .to_string()
                    .add_color(red())
                    .label(ui);
            }
            Columns::Online => {
                if player.online {
                    "online".add_color(white()).label(ui);
                } else {
                    player.last_login.to_colored().label(ui);
                }
            }
        }
    }
}
