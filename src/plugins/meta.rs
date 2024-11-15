use rand::seq::SliceRandom;

use super::*;

pub struct MetaPlugin;

impl Plugin for MetaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MetaResource>()
            .add_systems(OnExit(GameState::Meta), Self::clear);
    }
}

#[derive(AsRefStr, EnumIter, Clone, Copy)]
pub enum MetaMode {
    Shop,
    Auction,
    Inventory,
    Gallery,
    Balancing,
    Teams,
}

#[derive(Resource, Default)]
struct MetaResource {
    load_mode: Option<MetaMode>,
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
#[derive(Resource, Default)]
struct CraftResource {
    use_rainbow: u32,
}
fn brm(world: &mut World) -> Mut<BalancingResource> {
    world.resource_mut::<BalancingResource>()
}

impl AuctionResource {
    fn post(world: &mut World) {
        if let Some(ar) = world.remove_resource::<AuctionResource>() {
            let _ = cn().reducers.auction_create(ar.item_id, ar.count, ar.price);
        }
    }
}

impl MetaPlugin {
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            ui.vertical_centered_justified(|ui| {
                for m in MetaMode::iter() {
                    if Button::click(m.as_ref()).ui(ui).clicked() {
                        Self::load_mode(m, world);
                    }
                }
            });
        })
        .pinned()
        .transparent()
        .keep()
        .stretch_min()
        .push(world);
        if let Some(mode) = world.resource_mut::<MetaResource>().load_mode.take() {
            Self::load_mode(mode, world);
        }
    }
    pub fn set_mode(mode: MetaMode, world: &mut World) {
        world.resource_mut::<MetaResource>().load_mode = Some(mode);
    }
    pub fn load_mode(mode: MetaMode, world: &mut World) {
        Self::clear(world);
        match mode {
            MetaMode::Shop => Self::open_shop(world),
            MetaMode::Auction => Self::open_auction(world),
            MetaMode::Inventory => Self::open_inventory(world),
            MetaMode::Gallery => Self::open_gallery(world),
            MetaMode::Balancing => Self::open_balancing(world),
            MetaMode::Teams => TeamPlugin::teams_tiles(world),
        }
    }
    pub fn can_balance_vote() -> bool {
        cn().db
            .unit_balance()
            .iter()
            .filter(|b| b.owner == player_id())
            .count()
            < game_assets().heroes.len()
    }
    pub fn get_next_for_balancing(world: &mut World) {
        world.game_clear();
        let mut br = world.resource_mut::<BalancingResource>();
        if let Some(unit) = br.units.pop() {
            br.current = unit.clone();
            PackedUnit::from(cn().db.base_unit().name().find(&unit).unwrap()).unpack(
                TeamPlugin::entity(Faction::Team, world),
                None,
                None,
                world,
            );
        } else {
            br.current = default();
        }
    }
    fn skip_balancing(world: &mut World) {
        let mut br = world.resource_mut::<BalancingResource>();
        let current = br.current.clone();
        br.units.insert(0, current);
        Self::get_next_for_balancing(world);
    }
    fn on_enter_balancing(world: &mut World) {
        let voted: HashSet<String> = HashSet::from_iter(
            cn().db
                .unit_balance()
                .iter()
                .filter(|b| b.owner == player_id())
                .map(|u| u.unit),
        );
        let mut units = cn()
            .db
            .base_unit()
            .iter()
            .filter_map(
                |u| match u.pool == UnitPool::Game && !voted.contains(&u.name) {
                    true => Some(u.name),
                    false => None,
                },
            )
            .collect_vec();
        units.shuffle(&mut thread_rng());
        world.insert_resource(BalancingResource {
            units,
            current: default(),
            vote: default(),
        });
        Self::get_next_for_balancing(world);
    }
    pub fn clear(world: &mut World) {
        TilePlugin::clear(world);
        world.game_clear();
        TeamSyncPlugin::unsubscribe_all(world);
    }
    fn open_shop(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            show_daily_refresh_timer(ui);
            if !cn().db.daily_state().current().meta_shop_discount_spent {
                format!(
                    "Daily discount on first buy {:.0}%",
                    global_settings().meta.daily_discount * 100.0
                )
                .cstr_cs(GREEN, CstrStyle::Bold)
                .label(ui);
            }
            cn().db
                .meta_shop()
                .iter()
                .sorted_by_key(|d| d.id)
                .collect_vec()
                .show_table("Meta Shop", ui, world);
        })
        .pinned()
        .push(world);
    }
    fn open_inventory(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            Self::show_lootboxes(ui, world);
        })
        .pinned()
        .push(world);
        Tile::new(Side::Left, |ui, world| {
            Self::show_units(ui, world);
        })
        .pinned()
        .push(world);
        Tile::new(Side::Left, |ui, world| {
            Self::show_shards(ui, world);
        })
        .pinned()
        .push(world);
    }
    fn open_auction(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            cn().db
                .auction()
                .iter()
                .collect_vec()
                .show_modified_table("Auction", ui, world, |t| {
                    t.column(
                        "buy",
                        |_, _| default(),
                        |d: &TAuction, _, ui, _| {
                            let own = player_id() == d.owner;
                            if Button::click(if own { "cancel" } else { "buy" })
                                .enabled(own || can_afford(d.price))
                                .ui(ui)
                                .clicked()
                            {
                                let _ = cn().reducers.auction_buy(d.item_id);
                            }
                        },
                        false,
                    )
                    .filter("Units", "type", "unit".into())
                    .filter("Unit Shards", "type", "unit shard".into())
                    .filter("Rainbow Shards", "type", "rainbow shard".into())
                    .filter("Lootboxes", "type", "lootbox".into())
                });
        })
        .pinned()
        .push(world);
    }
    fn open_gallery(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            cn().db
                .base_unit()
                .iter()
                .collect_vec()
                .show_table("Base Units", ui, world);
        })
        .pinned()
        .push(world);
    }
    fn open_balancing(world: &mut World) {
        Self::on_enter_balancing(world);
        Tile::new(Side::Left, |ui, world| {
            let votes: HashMap<String, (u64, i32)> = HashMap::from_iter(
                cn().db
                    .unit_balance()
                    .iter()
                    .filter(|b| b.owner == player_id())
                    .map(|u| (u.unit, (u.id, u.vote))),
            );
            cn().db
                .base_unit()
                .iter()
                .filter(|u| votes.contains_key(&u.name))
                .sorted_by_key(|u| votes.get(&u.name).unwrap().0)
                .collect_vec()
                .show_modified_table("Base Units", ui, world, |t| {
                    t.column_int("vote", |u| {
                        cn().db
                            .unit_balance()
                            .iter()
                            .filter(|b| b.unit == u.name)
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
            let mut r = brm(world);
            r.vote = None;
            if r.current.is_empty() {
                return;
            }
            ui.horizontal_centered(|ui| {
                Middle3::default().width(200.0).ui_mut(
                    ui,
                    world,
                    |ui, world| {
                        if Button::click("OK").ui(ui).clicked() {
                            brm(world).vote = Some(0);
                        }
                        if Button::click("Skip").gray(ui).ui(ui).clicked() {
                            Self::skip_balancing(world);
                        }
                    },
                    |ui, world| {
                        if Button::click("Too Weak").color(CYAN, ui).ui(ui).clicked() {
                            brm(world).vote = Some(-1);
                        }
                    },
                    |ui, world| {
                        if Button::click("Too Strong")
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
                let _ = cn()
                    .reducers
                    .unit_balance_vote(world.resource::<BalancingResource>().current.clone(), vote);
            }
        })
        .pinned()
        .transparent()
        .push(world);
    }

    fn show_units(ui: &mut Ui, world: &mut World) {
        cn().db
            .unit_item()
            .iter()
            .filter(|u| u.owner == player_id())
            .map(|u| u.unit)
            .collect_vec()
            .show_modified_table("Units", ui, world, |t| {
                t.column_btn("sell", |unit, _, world| {
                    if Confirmation::has_active(world) {
                        return;
                    }
                    let item = cn()
                        .db
                        .unit_item()
                        .iter()
                        .filter(|u| u.owner == player_id())
                        .find(|i| i.unit.id == unit.id)
                        .unwrap();
                    let item_id = item.id;
                    world.insert_resource(AuctionResource {
                        item_id,
                        count: 1,
                        max_count: 1,
                        price: 1,
                    });
                    Confirmation::new("Create Auction".cstr())
                        .accept(|world| {
                            AuctionResource::post(world);
                        })
                        .cancel(|_| {})
                        .content(|ui, world| {
                            let mut ar = world.resource_mut::<AuctionResource>();
                            Slider::new("price").ui(&mut ar.price, 1..=1000, ui);
                        })
                        .push(world);
                })
                .column_btn("dismantle", |unit, _, world| {
                    if Confirmation::has_active(world) {
                        return;
                    }
                    let item = cn()
                        .db
                        .unit_item()
                        .iter()
                        .filter(|u| u.owner == player_id())
                        .find(|i| i.unit.id == unit.id)
                        .unwrap();
                    let item_id = item.id;
                    let base = unit.base_unit();
                    Confirmation::new(
                        format!(
                            "Dismantle {} to get {} Rainbow shards?",
                            base.name,
                            base.rarity + 1
                        )
                        .cstr_c(VISIBLE_LIGHT),
                    )
                    .accept(move |_| {
                        let _ = cn().reducers.dismantle_hero(item_id);
                    })
                    .cancel(|_| {})
                    .push(world);
                })
            });
    }

    fn show_shards(ui: &mut Ui, world: &mut World) {
        let rs = cn()
            .db
            .rainbow_shard_item()
            .iter()
            .filter(|u| u.owner == player_id())
            .exactly_one()
            .ok();
        text_dots_text(
            "Rainbow Shards".cstr_rainbow(),
            rs.clone()
                .map(|d| d.count)
                .unwrap_or_default()
                .to_string()
                .cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold),
            ui,
        );
        if Button::click("Sell").enabled(rs.is_some()).ui(ui).clicked() {
            if !Confirmation::has_active(world) {
                let item = rs.unwrap();
                let item_id = item.id;
                world.insert_resource(AuctionResource {
                    item_id,
                    count: 1,
                    max_count: item.count,
                    price: 1,
                });
                Confirmation::new("Create Auction".cstr())
                    .accept(|world| {
                        AuctionResource::post(world);
                    })
                    .cancel(|_| {})
                    .content(|ui, world| {
                        let mut ar = world.resource_mut::<AuctionResource>();
                        let max = ar.max_count;
                        if max > 1 {
                            Slider::new("count").ui(&mut ar.count, 1..=max, ui);
                        }
                        Slider::new("price").ui(&mut ar.price, 1..=1000, ui);
                    })
                    .push(world);
            }
        }
        let d = cn()
            .db
            .unit_shard_item()
            .iter()
            .filter(|u| u.owner == player_id())
            .sorted_by_key(|d| -(d.count as i32))
            .collect_vec();
        let rs = cn()
            .db
            .rainbow_shard_item()
            .iter()
            .filter(|u| u.owner == player_id())
            .exactly_one()
            .map(|i| i.count)
            .unwrap_or_default();
        d.show_modified_table("Hero Shards", ui, world, move |t| {
            t.column_cstr_click(
                "sell",
                |_, _| "sell".cstr_c(VISIBLE_LIGHT),
                |unit, world| {
                    if Confirmation::has_active(world) {
                        return;
                    }
                    let item = cn()
                        .db
                        .unit_shard_item()
                        .iter()
                        .filter(|u| u.owner == player_id())
                        .find(|i| i.id == unit.id)
                        .unwrap();
                    let item_id = item.id;
                    world.insert_resource(AuctionResource {
                        item_id,
                        count: 1,
                        max_count: item.count,
                        price: 1,
                    });
                    Confirmation::new("Create Auction".cstr())
                        .accept(|world| {
                            AuctionResource::post(world);
                        })
                        .cancel(|_| {})
                        .content(|ui, world| {
                            let mut ar = world.resource_mut::<AuctionResource>();
                            let max = ar.max_count;
                            if max > 1 {
                                Slider::new("count").ui(&mut ar.count, 1..=max, ui);
                            }
                            Slider::new("price").ui(&mut ar.price, 1..=1000, ui);
                        })
                        .push(world);
                },
            )
            .column_dyn(
                "craft",
                Box::new(|_, _| default()),
                Box::new(move |d, _, ui, world| {
                    let craft_cost = game_assets().global_settings.craft_shards_cost;
                    let needed = if craft_cost > d.count {
                        craft_cost - d.count
                    } else {
                        0
                    };
                    if Button::click("craft")
                        .enabled(d.count + rs >= craft_cost)
                        .ui(ui)
                        .clicked()
                    {
                        let unit = d.unit.clone();
                        world.insert_resource(CraftResource::default());
                        Confirmation::new(
                            "Craft ".cstr_c(VISIBLE_LIGHT) + &unit.cstr_c(name_color(&unit)),
                        )
                        .content(move |ui, world| {
                            if rs > 0 {
                                Slider::new("Use Rainbow Shards").ui(
                                    &mut world.resource_mut::<CraftResource>().use_rainbow,
                                    needed..=rs.at_most(craft_cost - 1),
                                    ui,
                                );
                            }
                        })
                        .accept(move |world| {
                            let _ = cn().reducers.craft_hero(
                                unit.clone(),
                                world.resource::<CraftResource>().use_rainbow,
                            );
                        })
                        .cancel(|_| {})
                        .push(world);
                    }
                }),
                false,
            )
        });
    }

    fn show_lootboxes(ui: &mut Ui, world: &mut World) {
        let d = cn()
            .db
            .lootbox_item()
            .iter()
            .filter(|u| u.owner == player_id())
            .sorted_by_key(|d| -(d.count as i32))
            .collect_vec();
        d.show_modified_table("Lootboxes", ui, world, |t| {
            t.column_cstr_click(
                "sell",
                |_, _| "sell".cstr_c(VISIBLE_LIGHT),
                |unit, world| {
                    if Confirmation::has_active(world) {
                        return;
                    }
                    let item = cn()
                        .db
                        .lootbox_item()
                        .iter()
                        .filter(|u| u.owner == player_id())
                        .find(|i| i.id == unit.id)
                        .unwrap();
                    let item_id = item.id;
                    world.insert_resource(AuctionResource {
                        item_id,
                        count: 1,
                        max_count: item.count,
                        price: 1,
                    });
                    Confirmation::new("Create Auction".cstr())
                        .accept(|world| {
                            AuctionResource::post(world);
                        })
                        .cancel(|_| {})
                        .content(|ui, world| {
                            let mut ar = world.resource_mut::<AuctionResource>();
                            let max = ar.max_count;
                            if max > 1 {
                                Slider::new("count").ui(&mut ar.count, 1..=max, ui);
                            }
                            Slider::new("price").ui(&mut ar.price, 1..=1000, ui);
                        })
                        .push(world);
                },
            )
            .column_btn("open", |d, _, _| {
                let _ = cn().reducers.open_lootbox(d.id);
            })
        });
    }
}
