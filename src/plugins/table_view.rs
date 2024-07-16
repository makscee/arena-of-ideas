use super::*;

pub struct TableViewPlugin;

impl Plugin for TableViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HistoryTables>()
            .init_resource::<HistoryData>()
            .add_systems(
                OnEnter(GameState::TableView(QUERY_BATTLE_HISTORY)),
                Self::on_enter_history,
            );
    }
}

#[derive(Resource, Default)]
struct HistoryTables {
    battles: Vec<TBattle>,
}

#[derive(Resource, Default)]
struct HistoryData {
    team: Option<GID>,
    user: Option<GID>,
}

impl TableViewPlugin {
    fn on_enter_history(mut data: ResMut<HistoryTables>) {
        data.battles = TBattle::iter().collect_vec();
    }
    pub fn ui(query: &str, ctx: &egui::Context, world: &mut World) {
        match query {
            QUERY_LEADERBOARD => Self::draw_leaderboard(ctx, world),
            QUERY_BATTLE_HISTORY => Self::draw_history(ctx, world),
            _ => panic!("Query not supported {query}"),
        }
    }
    fn draw_history(ctx: &egui::Context, world: &mut World) {
        let mut hd = world.remove_resource::<HistoryData>().unwrap();
        Tile::left("User")
            .close_btn()
            .show_data(&mut hd.user, ctx, |gid, ui| gid.get_user().show(ui, world));
        Tile::left("Team")
            .close_btn()
            .show_data(&mut hd.team, ctx, |gid, ui| {
                TTeam::filter_by_id(*gid).unwrap().show(ui, world);
            });
        world.insert_resource(hd);
        let ht = world.remove_resource::<HistoryTables>().unwrap();
        let show_team = |_: &TBattle, gid: VarValue, ui: &mut Ui, w: &mut World| {
            let team = gid.get_gid().unwrap().get_team();
            let r = team.cstr().button(ui);
            if r.clicked() {
                w.resource_mut::<HistoryData>().team = Some(team.id);
            }
            r
        };
        Tile::left("Battle History").show(ctx, |ui| {
            Table::new("Battle History")
                .title()
                .column_gid("id", |d: &TBattle| d.id)
                .column_user_click(
                    "owner",
                    |d| d.owner,
                    |gid, w| w.resource_mut::<HistoryData>().user = Some(gid),
                )
                .column("left", |d| d.team_left.into(), show_team)
                .column("right", |d| d.team_right.into(), show_team)
                .column_cstr("result", |d| match d.result {
                    TBattleResult::Tbd => "tbd".cstr(),
                    TBattleResult::Left | TBattleResult::Even => "win".cstr_c(GREEN),
                    TBattleResult::Right => "lose".cstr_c(RED),
                })
                .ui(&ht.battles, ui, world);
        });
        world.insert_resource(ht);
    }
    fn draw_leaderboard(ctx: &egui::Context, world: &mut World) {
        // Tile::left("Leaderboard")
        //     .open()
        //     .non_resizable()
        //     .show(ctx, |ui| Table::new("Leaderboard"));
    }
}
