use chrono::Utc;

use super::*;

pub struct MetaPlugin;

impl MetaPlugin {
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Top, |ui, world| {
            if SubstateMenu::show(
                &[
                    GameState::MetaShop,
                    GameState::MetaHeroes,
                    GameState::MetaHeroShards,
                    GameState::MetaLootboxes,
                ],
                ui,
                world,
            ) {
                TableState::reset_cache(ui.ctx());
            }
        })
        .pinned()
        .push(world);
        match cur_state(world) {
            GameState::MetaShop => Tile::new(Side::Left, |ui, world| {
                text_dots_text(
                    "wallet".cstr(),
                    format!("{} ¤", TWallet::current().amount).cstr_cs(YELLOW, CstrStyle::Bold),
                    ui,
                );
                br(ui);
                let now = Utc::now().timestamp();
                let til_refresh = (now / 86400 + 1) * 86400 - now;
                "Refresh in "
                    .cstr()
                    .push(
                        format!(
                            "{:02}:{:02}:{:02}",
                            til_refresh / 3600,
                            til_refresh / 60 % 60,
                            til_refresh % 60
                        )
                        .cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold),
                    )
                    .label(ui);
                TMetaShop::iter()
                    .sorted_by_key(|d| d.id)
                    .collect_vec()
                    .show_table("Meta Shop", ui, world);
            })
            .pinned()
            .push(world),
            GameState::MetaHeroes => Tile::new(Side::Left, |ui, world| {
                TUnitItem::filter_by_owner(user_id())
                    .map(|u| u.unit)
                    .collect_vec()
                    .show_table("Units", ui, world);
            })
            .pinned()
            .push(world),
            GameState::MetaHeroShards => Tile::new(Side::Left, |ui, world| {
                let d = TUnitShardItem::filter_by_owner(user_id())
                    .sorted_by_key(|d| -(d.count as i32))
                    .collect_vec();
                d.show_modified_table("Hero Shards", ui, world, |t| {
                    t.column(
                        "action",
                        |_, _| default(),
                        |d, _, ui, world| {
                            let craft_cost =
                                GameAssets::get(world).global_settings.craft_shards_cost;
                            let r = Button::click("craft".into())
                                .enabled(d.count >= craft_cost)
                                .ui(ui);
                            if r.clicked() {
                                craft_hero(d.unit.clone());
                                once_on_craft_hero(|_, _, status, unit| match status {
                                    StdbStatus::Committed => {
                                        Notification::new_string(format!("{unit} crafted"))
                                            .push_op()
                                    }
                                    StdbStatus::Failed(e) => e.notify_error_op(),
                                    _ => panic!(),
                                });
                            }
                            r
                        },
                    )
                });
            })
            .pinned()
            .push(world),
            GameState::MetaLootboxes => Tile::new(Side::Left, |ui, world| {
                let d = TLootboxItem::filter_by_owner(user_id())
                    .sorted_by_key(|d| -(d.count as i32))
                    .collect_vec();
                d.show_modified_table("Lootboxes", ui, world, |t| {
                    t.column_btn("open", |d, _, _| {
                        open_lootbox(d.id);
                        TTrade::on_insert(|trade, e| {
                            let id = trade.id;
                            if e.is_some_and(|e| matches!(e, ReducerEvent::OpenLootbox(..))) {
                                OperationsPlugin::add(move |world| {
                                    Trade::open(id, &egui_context(world).unwrap());
                                });
                            }
                        });
                        once_on_open_lootbox(|_, _, status, _| {
                            status.on_success(|world| {
                                "Lootbox opened".notify(world);
                            })
                        });
                    })
                });
            })
            .pinned()
            .push(world),
            _ => panic!(),
        }
    }
}
