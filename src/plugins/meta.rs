use chrono::Utc;

use super::*;

pub struct MetaPlugin;

impl Plugin for MetaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MetaResource>();
    }
}

#[derive(Resource, Default)]
struct MetaResource {
    state: SubState,
}

#[derive(PartialEq, Copy, Clone, EnumIter, Display, Default)]
enum SubState {
    #[default]
    Shop,
    HeroShards,
    Heroes,
    Lootboxes,
}

impl MetaPlugin {
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Top, |ui, world| {
            let mut r = world.resource_mut::<MetaResource>();
            let state = SubsectionMenu::new(r.state).show(ui);
            if r.state != state {
                r.state = state;
                TableState::reset_cache(ui.ctx());
            }
        })
        .sticky()
        .push(world);

        Tile::new(Side::Left, |ui, world| {
            let state = world.resource::<MetaResource>().state;
            match state {
                SubState::Shop => {
                    text_dots_text(
                        "wallet".cstr(),
                        format!("{} ¤", TWallet::current().amount).cstr_cs(YELLOW, CstrStyle::Bold),
                        ui,
                    );
                    br(ui);
                    let now = Utc::now().timestamp();
                    let last_refresh =
                        Duration::from_micros(GlobalData::current().last_shop_refresh).as_secs()
                            as i64;
                    let period = GlobalSettings::current().meta.shop_refresh_period_secs as i64;
                    let til_refresh = period - now + last_refresh;
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
                }
                SubState::HeroShards => {
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
                }
                SubState::Heroes => {
                    TUnitItem::filter_by_owner(user_id())
                        .map(|u| u.unit)
                        .collect_vec()
                        .show_table("Units", ui, world);
                }
                SubState::Lootboxes => {
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
                }
            }
        })
        .sticky()
        .push(world);
    }
}
