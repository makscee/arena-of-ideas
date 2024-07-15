use super::*;

pub struct TableViewPlugin;

impl Plugin for TableViewPlugin {
    fn build(&self, app: &mut App) {}
}

impl TableViewPlugin {
    pub fn ui(query: &str, ctx: &egui::Context, world: &mut World) {
        match query {
            QUERY_LEADERBOARD => Self::draw_leaderboard(ctx, world),
            _ => panic!("Query not supported {query}"),
        }
    }
    fn draw_leaderboard(ctx: &egui::Context, world: &mut World) {
        Tile::left("Leaderboard")
            .open()
            .non_resizable()
            .title()
            .content(|ui, _| {
                br(ui);
                Table::new_cached(
                    "Leaderboard",
                    || TArenaLeaderboard::iter().collect_vec(),
                    ui.ctx(),
                )
                .column(
                    "season",
                    TableColumn::new(|v: &TArenaLeaderboard| (v.season as i32).into()),
                )
                .column(
                    "round",
                    TableColumn::new(|v: &TArenaLeaderboard| (v.round as i32).into()),
                )
                .column(
                    "name",
                    TableColumn::new(|v: &TArenaLeaderboard| {
                        TUser::filter_by_id(v.user).unwrap().name.into()
                    })
                    .no_sort()
                    .show(|v: &TArenaLeaderboard, _, ui| {
                        let name = TUser::filter_by_id(v.user).unwrap().name.clone();
                        let resp = Button::click(name).ui(ui);
                        if resp.clicked() {
                            debug!("click");
                        }
                        resp
                    }),
                )
                .column(
                    "score",
                    TableColumn::new(|v: &TArenaLeaderboard| (v.score as i32).into()),
                )
                .ui(ui);
            })
            .show(ctx, world);
    }
}
