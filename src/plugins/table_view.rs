use super::*;

pub struct TableViewPlugin;

impl Plugin for TableViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HistoryData>();
    }
}

#[derive(Resource, Default)]
struct HistoryData {
    team: Option<(GID, bool)>,
}

impl TableViewPlugin {
    pub fn ui(query: &str, ctx: &egui::Context, world: &mut World) {
        match query {
            QUERY_LEADERBOARD => Self::draw_leaderboard(ctx, world),
            QUERY_BATTLE_HISTORY => Self::draw_history(ctx, world),
            _ => panic!("Query not supported {query}"),
        }
    }
    fn draw_history(ctx: &egui::Context, world: &mut World) {
        Tile::left("Battle History")
            .open()
            .title()
            // .child(|ctx, world| {
            //     Tile::left("Team")
            //         .title()
            //         .close_btn()
            //         .content(|ui, world| {
            //             let (team_id, refresh) = world.resource::<HistoryData>().team.unwrap();
            //             if refresh {
            //                 world.resource_mut::<HistoryData>().team = Some((team_id, false));
            //             }
            //             let team = TTeam::filter_by_id(team_id).unwrap();
            //             let owner = TUser::filter_by_id(team.owner).unwrap();
            //             text_dots_text(&"owner".cstr(), &owner.name.cstr_c(VISIBLE_BRIGHT), ui);
            //             br(ui);
            //             Table::new_cached_refreshed(
            //                 "Team",
            //                 refresh,
            //                 move || TTeam::filter_by_id(team_id).unwrap().units,
            //                 ui.ctx(),
            //             )
            //             .column("name", column_value(|u| u.bases.join(" ").into()))
            //             .ui(ui, world);
            //         })
            //         .show(ctx, world);
            // })
            .show(ctx, |ui| {
                br(ui);
                Table::new_cached("Battle History", || TBattle::iter().collect_vec(), ui.ctx())
                    .column("GID", column_value(|v| (v.id as i32).into()))
                    .column(
                        "owner",
                        column_value(|v: &TBattle| {
                            TUser::filter_by_id(v.owner).unwrap().name.into()
                        })
                        .show_fn(|v: &TBattle, _, ui, _| {
                            let user = TUser::filter_by_id(v.owner).unwrap();
                            let resp = Button::click(user.name.clone()).cstr(user.cstr()).ui(ui);
                            if resp.clicked() {
                                debug!("user {}", user.name);
                            }
                            resp
                        }),
                    )
                    .column(
                        "left",
                        column_show(|v: &TBattle, _, ui, _| {
                            let resp = v.team_left.get_team().cstr().button(ui);
                            if resp.clicked() {}
                            resp
                        }),
                    )
                    .column(
                        "right",
                        column_show(|v: &TBattle, _, ui, _| {
                            let resp = v.team_right.get_team().cstr().button(ui);
                            if resp.clicked() {}
                            resp
                        }),
                    )
                    .ui(ui, world);
            });
    }
    fn draw_leaderboard(ctx: &egui::Context, world: &mut World) {
        Tile::left("Leaderboard")
            .open()
            .non_resizable()
            .title()
            .show(ctx, |ui| {
                br(ui);
                Table::new_cached(
                    "Leaderboard",
                    || TArenaLeaderboard::iter().collect_vec(),
                    ui.ctx(),
                )
                .column(
                    "season",
                    column_value(|v: &TArenaLeaderboard| (v.season as i32).into()),
                )
                .column(
                    "round",
                    column_value(|v: &TArenaLeaderboard| (v.round as i32).into()),
                )
                .column(
                    "user",
                    column_value(|v: &TArenaLeaderboard| {
                        TUser::filter_by_id(v.user).unwrap().name.into()
                    })
                    .no_sort()
                    .show_fn(|v: &TArenaLeaderboard, _, ui, _| {
                        let user = TUser::filter_by_id(v.user).unwrap();
                        let resp = Button::click(user.name.clone()).cstr(user.cstr()).ui(ui);
                        if resp.clicked() {
                            debug!("click");
                        }
                        resp
                    }),
                )
                .column(
                    "score",
                    column_value(|v: &TArenaLeaderboard| (v.score as i32).into()),
                )
                .ui(ui, world);
            });
    }
}
