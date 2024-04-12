use egui_extras::{Column, TableBuilder};

use super::*;

pub struct LeaderboardPlugin;

impl LeaderboardPlugin {
    pub fn ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };

        let columns = Columns::iter().collect_vec();
        window("LEADERBOARD")
            .set_width(400.0)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                TableBuilder::new(ui)
                    .columns(Column::auto(), 5)
                    .striped(true)
                    .header(20.0, |mut row| {
                        for col in columns {
                            row.col(|ui| {
                                col.to_string().add_color(white()).label(ui);
                            });
                        }
                    });
            });
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Display)]
enum Columns {
    Name,
    Team,
    Wins,
    Loses,
    Time,
}

impl Columns {}
