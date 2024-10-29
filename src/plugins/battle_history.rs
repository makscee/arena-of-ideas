use super::*;

pub struct BattleHistoryPlugin;

#[derive(Resource)]
struct BattleHistoryResource {
    battles: Vec<TBattle>,
}

impl BattleHistoryPlugin {
    pub fn add_tiles(world: &mut World) {
        world.insert_resource(BattleHistoryResource {
            battles: TBattle::iter()
                .sorted_by(|a, b| b.id.cmp(&a.id))
                .collect_vec(),
        });
        Tile::new(Side::Left, |ui, world| {
            let bh = world.remove_resource::<BattleHistoryResource>().unwrap();
            Table::new("Battle History")
                .title()
                .column_gid("id", |d: &TBattle| d.id)
                .column_cstr("mode", |d, _| d.mode.cstr())
                .column_user_click(
                    "player",
                    |d| d.owner,
                    |gid, _, world| TilePlugin::add_user(gid, world),
                )
                .column_team("player team >", |d| d.team_left)
                .column_team("< enemy team", |d| d.team_right)
                .column_user_click(
                    "enemy",
                    |d| d.team_right.get_team().owner,
                    |gid, _, world| TilePlugin::add_user(gid, world),
                )
                .column_cstr("result", |d, _| match d.result {
                    TBattleResult::Tbd => "-".cstr(),
                    TBattleResult::Left => "W".cstr_c(GREEN),
                    TBattleResult::Right | TBattleResult::Even => "L".cstr_c(RED),
                })
                .column_ts("time", |d| d.ts)
                .column_btn("copy", |d, _, world| {
                    copy_to_clipboard(
                        &ron::to_string(&BattleResource::from(d.clone())).unwrap(),
                        world,
                    );
                })
                .column_btn("editor", |d, _, world| {
                    EditorPlugin::load_battle(
                        PackedTeam::from_id(d.team_left),
                        PackedTeam::from_id(d.team_right),
                    );
                    GameState::Editor.set_next(world);
                })
                .column_btn("run", |d, _, world| {
                    world.insert_resource(BattleResource::from(d.clone()));
                    BattlePlugin::set_next_state(cur_state(world), world);
                    GameState::Battle.set_next(world);
                })
                .filter("My", "player", user_id().into())
                .filter("Win", "result", "W".into())
                .filter("Lose", "result", "L".into())
                .filter("TBD", "result", "-".into())
                .ui(&bh.battles, ui, world);
            world.insert_resource(bh);
        })
        .pinned()
        .push(world);
    }
}
