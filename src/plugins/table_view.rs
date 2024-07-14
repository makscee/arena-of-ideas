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
        });
    }
}
