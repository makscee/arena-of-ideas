use chrono::Utc;

use super::*;

pub struct MetaPlugin;

#[derive(Resource)]
struct AuctionResource {
    item_id: u64,
    count: u32,
    max_count: u32,
    price: i64,
}

impl AuctionResource {
    fn post(world: &mut World) {
        if let Some(ar) = world.remove_resource::<AuctionResource>() {
            auction_create(ar.item_id, ar.count, ar.price);
            once_on_auction_create(|_, _, status, _, _, _| {
                status.on_success(|w| "Auction created".notify(w));
            });
        }
    }
}

impl MetaPlugin {
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Top, |ui, world| {
            if SubstateMenu::show(
                &[
                    GameState::MetaShop,
                    GameState::MetaAuction,
                    GameState::MetaHeroes,
                    GameState::MetaHeroShards,
                    GameState::MetaLootboxes,
                    GameState::MetaGallery,
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
                    format!("{} {CREDITS_SYM}", TWallet::current().amount)
                        .cstr_cs(YELLOW, CstrStyle::Bold),
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
            GameState::MetaAuction => {
                Tile::new(Side::Left, |ui, world| {
                    text_dots_text(
                        "wallet".cstr(),
                        format!("{} {CREDITS_SYM}", TWallet::current().amount)
                            .cstr_cs(YELLOW, CstrStyle::Bold),
                        ui,
                    );
                    br(ui);
                    TAuction::iter()
                        .collect_vec()
                        .show_modified_table("Auction", ui, world, |t| {
                            t.column(
                                "buy",
                                |_, _| default(),
                                |d, _, ui, _| {
                                    let own = user_id() == d.owner;
                                    if Button::click(if own { "cancel" } else { "buy" }.into())
                                        .enabled(own || can_afford(d.price))
                                        .ui(ui)
                                        .clicked()
                                    {
                                        auction_buy(d.item_id);
                                        once_on_auction_buy(|_, _, status, id| match status {
                                            StdbStatus::Committed => {
                                                format!("Auction#{id} bought").notify_op()
                                            }
                                            StdbStatus::Failed(e) => e.notify_error_op(),
                                            _ => panic!(),
                                        });
                                    }
                                },
                                false,
                            )
                            .filter("Units", "type", "unit".into())
                            .filter("Unit Shards", "type", "unit shard".into())
                            .filter(
                                "Lootboxes",
                                "type",
                                "lootbox".into(),
                            )
                        });
                })
                .push(world);
            }
            GameState::MetaHeroes => Tile::new(Side::Left, |ui, world| {
                TUnitItem::filter_by_owner(user_id())
                    .map(|u| u.unit)
                    .collect_vec()
                    .show_modified_table("Units", ui, world, |t| {
                        t.column_cstr_click(
                            "sell",
                            |_, _| "sell".cstr_c(VISIBLE_LIGHT),
                            |unit, world| {
                                let item = TUnitItem::filter_by_owner(user_id())
                                    .find(|i| i.unit.id == unit.id)
                                    .unwrap();
                                let item_id = item.id;
                                world.insert_resource(AuctionResource {
                                    item_id,
                                    count: 1,
                                    max_count: 1,
                                    price: 1,
                                });
                                Confirmation::new("Create Auction".cstr(), |world| {
                                    AuctionResource::post(world);
                                })
                                .content(|ui, world| {
                                    let mut ar = world.resource_mut::<AuctionResource>();
                                    Slider::new("price").ui(&mut ar.price, 1..=1000, ui);
                                })
                                .push(&egui_context(world).unwrap());
                            },
                        )
                    });
            })
            .pinned()
            .push(world),
            GameState::MetaHeroShards => Tile::new(Side::Left, |ui, world| {
                let d = TUnitShardItem::filter_by_owner(user_id())
                    .sorted_by_key(|d| -(d.count as i32))
                    .collect_vec();
                d.show_modified_table("Hero Shards", ui, world, |t| {
                    t.column_cstr_click(
                        "sell",
                        |_, _| "sell".cstr_c(VISIBLE_LIGHT),
                        |unit, world| {
                            let item = TUnitShardItem::filter_by_owner(user_id())
                                .find(|i| i.id == unit.id)
                                .unwrap();
                            let item_id = item.id;
                            world.insert_resource(AuctionResource {
                                item_id,
                                count: 1,
                                max_count: item.count,
                                price: 1,
                            });
                            Confirmation::new("Create Auction".cstr(), |world| {
                                AuctionResource::post(world);
                            })
                            .content(|ui, world| {
                                let mut ar = world.resource_mut::<AuctionResource>();
                                let max = ar.max_count;
                                if max > 1 {
                                    Slider::new("count").ui(&mut ar.count, 1..=max, ui);
                                }
                                Slider::new("price").ui(&mut ar.price, 1..=1000, ui);
                            })
                            .push(&egui_context(world).unwrap());
                        },
                    )
                    .column(
                        "craft",
                        |_, _| default(),
                        |d, _, ui, world| {
                            let craft_cost =
                                GameAssets::get(world).global_settings.craft_shards_cost;
                            if Button::click("craft".into())
                                .enabled(d.count >= craft_cost)
                                .ui(ui)
                                .clicked()
                            {
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
                        },
                        false,
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
                    t.column_cstr_click(
                        "sell",
                        |_, _| "sell".cstr_c(VISIBLE_LIGHT),
                        |unit, world| {
                            let item = TLootboxItem::filter_by_owner(user_id())
                                .find(|i| i.id == unit.id)
                                .unwrap();
                            let item_id = item.id;
                            world.insert_resource(AuctionResource {
                                item_id,
                                count: 1,
                                max_count: item.count,
                                price: 1,
                            });
                            Confirmation::new("Create Auction".cstr(), |world| {
                                AuctionResource::post(world);
                            })
                            .content(|ui, world| {
                                let mut ar = world.resource_mut::<AuctionResource>();
                                let max = ar.max_count;
                                if max > 1 {
                                    Slider::new("count").ui(&mut ar.count, 1..=max, ui);
                                }
                                Slider::new("price").ui(&mut ar.price, 1..=1000, ui);
                            })
                            .push(&egui_context(world).unwrap());
                        },
                    )
                    .column_btn("open", |d, _, _| {
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
            GameState::MetaGallery => Tile::new(Side::Left, |ui, world| {
                TBaseUnit::iter()
                    .collect_vec()
                    .show_table("Base Units", ui, world);
            })
            .pinned()
            .push(world),
            _ => panic!(),
        }
    }
}
