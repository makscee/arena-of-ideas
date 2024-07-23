use super::*;

pub struct TableViewPlugin;

impl Plugin for TableViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TablesData>()
            .add_systems(
                OnEnter(GameState::TableView(QUERY_BATTLE_HISTORY)),
                Self::on_enter_history,
            )
            .add_systems(
                OnEnter(GameState::TableView(QUERY_BASE_UNITS)),
                Self::on_enter_base_units,
            );
    }
}

#[derive(Resource, Default)]
struct TablesData {
    battles: Vec<TBattle>,
    base_units: Vec<TBaseUnit>,
}

impl TableViewPlugin {
    fn on_enter_history(mut data: ResMut<TablesData>) {
        data.battles = TBattle::iter().collect_vec();
    }
    fn on_enter_base_units(mut data: ResMut<TablesData>) {
        data.base_units = TBaseUnit::iter().collect_vec();
    }
    pub fn ui(query: &str, ctx: &egui::Context, world: &mut World) {
        match query {
            QUERY_LEADERBOARD => Self::draw_leaderboard(ctx, world),
            QUERY_BATTLE_HISTORY => Self::draw_history(ctx, world),
            QUERY_BASE_UNITS => Self::draw_base_units(ctx, world),
            _ => panic!("Query not supported {query}"),
        }
    }
    pub fn ui_content(query: &str, wd: &mut WidgetData, ui: &mut Ui, world: &mut World) {
        match query {
            QUERY_BASE_UNITS | QUERY_BATTLE_HISTORY => {
                UnitContainer::new(Faction::Team)
                    .hover_content(ShopPlugin::container_on_hover)
                    .position(egui::vec2(0.5, 0.5))
                    .slots(1)
                    .ui(wd, ui, world);
            }
            _ => panic!("Query not supported {query}"),
        }
    }
    fn draw_history(ctx: &egui::Context, world: &mut World) {
        let td = world.remove_resource::<TablesData>().unwrap();
        let show_team = |_: &TBattle, gid: VarValue, ui: &mut Ui, _: &mut World| {
            let team = gid.get_gid().unwrap().get_team();
            let r = team.cstr().button(ui);
            if r.clicked() {
                Tile::add_team(team.id, ui.ctx());
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
                    |gid, ui, _| Tile::add_user(gid, ui.ctx()),
                )
                .column("left", |d| d.team_left.into(), show_team)
                .column("right", |d| d.team_right.into(), show_team)
                .column_cstr("result", |d| match d.result {
                    TBattleResult::Tbd => "tbd".cstr(),
                    TBattleResult::Left | TBattleResult::Even => "win".cstr_c(GREEN),
                    TBattleResult::Right => "lose".cstr_c(RED),
                })
                .ui(&td.battles, ui, world);
        });
        world.insert_resource(td);
    }
    fn draw_base_units(ctx: &egui::Context, world: &mut World) {
        let td = world.remove_resource::<TablesData>().unwrap();
        Tile::left("Base Units").show(ctx, |ui| {
            td.base_units
                .show_modified_table("Base Units", ui, world, |t| {
                    t.column_btn("spawn", |u, _, world| {
                        let unit: PackedUnit = u.clone().into();
                        TeamPlugin::despawn(Faction::Team, world);
                        unit.unpack(TeamPlugin::entity(Faction::Team, world), None, None, world);
                    })
                });
        });
        world.insert_resource(td);
    }
    fn draw_leaderboard(ctx: &egui::Context, world: &mut World) {
        // Tile::left("Leaderboard")
        //     .open()
        //     .non_resizable()
        //     .show(ctx, |ui| Table::new("Leaderboard"));
    }
}