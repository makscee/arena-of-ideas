use chrono::Utc;
use rand::seq::SliceRandom;

use super::*;

pub struct MetaPlugin;

impl Plugin for MetaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MetaBalancing), Self::on_enter_balancing)
            .add_systems(OnExit(GameState::MetaBalancing), Self::on_exit_balancing);
    }
}

#[derive(Resource)]
struct AuctionResource {
    item_id: u64,
    count: u32,
    max_count: u32,
    price: i64,
}
#[derive(Resource)]
struct BalancingResource {
    units: Vec<String>,
    current: String,
    vote: Option<i32>,
}
fn brm(world: &mut World) -> Mut<BalancingResource> {
    world.resource_mut::<BalancingResource>()
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
    fn get_next_for_balancing(world: &mut World) {
        world.game_clear();
        TableState::reset_cache(&egui_context(world).unwrap());
        let mut br = world.resource_mut::<BalancingResource>();
        if let Some(unit) = br.units.pop() {
            br.current = unit.clone();
            PackedUnit::from(TBaseUnit::find_by_name(unit).unwrap()).unpack(
                TeamPlugin::entity(Faction::Team, world),
                None,
                None,
                world,
            );
        }
    }
    fn skip_balancing(world: &mut World) {
        let mut br = world.resource_mut::<BalancingResource>();
        let current = br.current.clone();
        br.units.insert(0, current);
        Self::get_next_for_balancing(world);
    }
    fn on_enter_balancing(world: &mut World) {
        let voted: HashSet<String> =
            HashSet::from_iter(TUnitBalance::filter_by_owner(user_id()).map(|u| u.unit));
        let mut units = TBaseUnit::iter()
            .filter_map(|u| match u.rarity >= 0 && !voted.contains(&u.name) {
                true => Some(u.name),
                false => None,
            })
            .collect_vec();
        units.shuffle(&mut thread_rng());
        world.insert_resource(BalancingResource {
            units,
            current: default(),
            vote: default(),
        });
        Self::get_next_for_balancing(world);
    }
    fn on_exit_balancing(world: &mut World) {
        world.game_clear();
    }
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
                    GameState::MetaBalancing,
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
            GameState::MetaBalancing => {
                Tile::new(Side::Left, |ui, world| {
                    let votes: HashMap<String, i32> = HashMap::from_iter(
                        TUnitBalance::filter_by_owner(user_id()).map(|u| (u.unit, u.vote)),
                    );
                    TBaseUnit::iter()
                        .filter(|u| votes.contains_key(&u.name))
                        .collect_vec()
                        .show_modified_table("Base Units", ui, world, |t| {
                            t.column_int("vote", |u| {
                                TUnitBalance::filter_by_unit(u.name.clone())
                                    .map(|u| u.vote)
                                    .sum::<i32>()
                            })
                            .column_cstr_click(
                                "action",
                                |_, _| "vote".cstr_c(VISIBLE_LIGHT),
                                |d, world| {
                                    brm(world).units.push(d.name.clone());
                                    Self::get_next_for_balancing(world);
                                },
                            )
                        });
                })
                .pinned()
                .push(world);
                Tile::new(Side::Top, |ui, world| {
                    TeamContainer::new(Faction::Team)
                        .slots(1)
                        .slot_content(|_, entity, ui, world| {
                            let Some(entity) = entity else {
                                return;
                            };
                            if let Ok(card) = UnitCard::new(&Context::new(entity), world) {
                                card.ui(ui);
                            }
                        })
                        .ui(ui, world);
                })
                .pinned()
                .transparent()
                .push(world);
                Tile::new(Side::Bottom, |ui, world| {
                    brm(world).vote = None;
                    ui.horizontal_centered(|ui| {
                        Middle3::default().width(200.0).ui_mut(
                            ui,
                            world,
                            |ui, world| {
                                if Button::click("OK".into()).ui(ui).clicked() {
                                    brm(world).vote = Some(0);
                                }
                                if Button::click("Skip".into()).gray(ui).ui(ui).clicked() {
                                    Self::skip_balancing(world);
                                }
                            },
                            |ui, world| {
                                if Button::click("Too Weak".into())
                                    .color(CYAN, ui)
                                    .ui(ui)
                                    .clicked()
                                {
                                    brm(world).vote = Some(-1);
                                }
                            },
                            |ui, world| {
                                if Button::click("Too Strong".into())
                                    .color(YELLOW, ui)
                                    .ui(ui)
                                    .clicked()
                                {
                                    brm(world).vote = Some(1);
                                }
                            },
                        );
                    });

                    if let Some(vote) = brm(world).vote {
                        unit_balance_vote(
                            world.resource::<BalancingResource>().current.clone(),
                            vote,
                        );
                        once_on_unit_balance_vote(|_, _, status, unit, vote| {
                            let unit = unit.clone();
                            let vote = if *vote >= 0 {
                                format!("+{vote}")
                            } else {
                                vote.to_string()
                            };
                            status.on_success(move |w| {
                                format!("Vote accepted: {unit} {vote}").notify(w);
                                Self::get_next_for_balancing(w);
                            });
                        });
                    }
                })
                .pinned()
                .transparent()
                .push(world);
            }
            _ => panic!(),
        }
    }
}
