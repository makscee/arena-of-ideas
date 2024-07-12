use egui_extras::{Column, TableBuilder};

use super::*;

pub struct TableViewPlugin;

impl Plugin for TableViewPlugin {
    fn build(&self, app: &mut App) {}
}

impl TableViewPlugin {
    pub fn ui(query: &str, ui: &mut Ui, world: &mut World) {
        match query {
            QUERY_LEADERBOARD => Self::draw_leaderboard(ui),
            _ => panic!("Query not supported {query}"),
        }
    }
    fn draw_leaderboard(ui: &mut Ui) {
        center_window("Leaderboard", ui, |ui| {
            TableBuilder::new(ui)
                .columns(Column::auto(), 3)
                .header(20.0, |mut h| {
                    h.col(|ui| {
                        Button::click("round".into()).gray(ui).ui(ui);
                    });
                    h.col(|ui| {
                        Button::click("score".into()).gray(ui).ui(ui);
                    });
                })
                .body(|mut body| {
                    for TArenaLeaderboard {
                        season,
                        round,
                        score,
                        user,
                        team,
                        run,
                    } in TArenaLeaderboard::iter()
                    {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label(round.to_string());
                            });
                            row.col(|ui| {
                                ui.label(score.to_string());
                            });
                        });
                    }
                });
        });
    }
}
