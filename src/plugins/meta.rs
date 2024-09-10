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
    Inventory,
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
                TeamPlugin::despawn(Faction::Team, world);
            }
        })
        .min_size(0.0)
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
                SubState::Inventory => {
                    TItem::iter()
                        .collect_vec()
                        .show_table("Inventory", ui, world);
                }
                SubState::HeroShards => {
                    let d = TItem::iter()
                        .filter(|d| matches!(d.stack.item, Item::HeroShard(..)))
                        .sorted_by_key(|d| -(d.stack.count as i32))
                        .collect_vec();
                    Table::new("Hero Shards")
                        .title()
                        .column_cstr("name", |d: &TItem, _| d.stack.item.name_cstr())
                        .column_cstr("type", |d, world| d.stack.item.type_cstr(world))
                        .column_int("count", |d| d.stack.count as i32)
                        .column(
                            "action",
                            |_, _| default(),
                            |d, _, ui, world| {
                                let craft_cost =
                                    GameAssets::get(world).global_settings.craft_shards_cost;
                                match &d.stack.item {
                                    Item::HeroShard(base) => {
                                        let r = Button::click("craft".into())
                                            .enabled(d.stack.count >= craft_cost)
                                            .ui(ui);
                                        if r.clicked() {
                                            craft_hero(base.clone());
                                            once_on_craft_hero(|_, _, status, hero| match status {
                                                StdbStatus::Committed => Notification::new_string(
                                                    format!("{hero} crafted"),
                                                )
                                                .push_op(),
                                                StdbStatus::Failed(e) => e.notify_error(),
                                                _ => panic!(),
                                            });
                                        }
                                        r
                                    }
                                    _ => panic!(),
                                }
                            },
                        )
                        .ui(&d, ui, world);
                }
                SubState::Heroes => {
                    let d = TItem::iter()
                        .filter_map(|d| match d.stack.item {
                            Item::Hero(u) => Some(u),
                            _ => None,
                        })
                        .collect_vec();
                    d.show_modified_table("Heroes", ui, world, |t| {
                        t.column(
                            "select",
                            |_, _| default(),
                            |d: &FusedUnit, _, ui, _| set_starting_hero_button(d.id, ui),
                        )
                        .column_btn("spawn", |u, _, world| {
                            let unit: PackedUnit = u.clone().into();
                            TeamPlugin::despawn(Faction::Team, world);
                            unit.unpack(
                                TeamPlugin::entity(Faction::Team, world),
                                None,
                                None,
                                world,
                            );
                        })
                    });
                }
                SubState::Lootboxes => {
                    TItem::iter()
                        .filter(|d| matches!(d.stack.item, Item::Lootbox))
                        .collect_vec()
                        .show_table("Lootboxes", ui, world);
                }
            }
        })
        .sticky()
        .push(world);
    }
}
fn set_starting_hero_button(id: u64, ui: &mut Ui) -> Response {
    let active = TStartingHero::get_current()
        .map(|d| d.item_id)
        .unwrap_or_default()
        == id;
    let r = Button::click("select".into()).active(active).ui(ui);
    if r.clicked() {
        let id = if active { None } else { Some(id) };
        set_starting_hero(id);
        once_on_set_starting_hero(move |_, _, status, id| match status {
            StdbStatus::Committed => {
                Notification::new_string(format!("{id:?} set as starting hero")).push_op()
            }
            StdbStatus::Failed(e) => e.notify_error(),
            _ => panic!(),
        });
    }
    r
}

pub trait ItemExt {
    fn name_cstr(&self) -> Cstr;
    fn type_cstr(&self, world: &World) -> Cstr;
}

impl ItemExt for Item {
    fn name_cstr(&self) -> Cstr {
        match &self {
            Item::HeroShard(name) => name.cstr_c(name_color(&name)),
            Item::Hero(unit) => unit.cstr(),
            Item::Lootbox => "normal".cstr(),
        }
    }
    fn type_cstr(&self, world: &World) -> Cstr {
        match self {
            Item::HeroShard(n) => "shard "
                .cstr()
                .push(Rarity::from_base(n, world).cstr())
                .take(),
            Item::Hero(h) => "hero "
                .cstr_c(YELLOW)
                .push(Rarity::from_base(&h.bases[0], world).cstr())
                .take(),
            Item::Lootbox => "lootbox".cstr_c(CYAN),
        }
    }
}

pub trait TItemExt {
    fn action(&self, ui: &mut Ui, world: &mut World) -> Response;
}

impl TItemExt for TItem {
    fn action(&self, ui: &mut Ui, world: &mut World) -> Response {
        match &self.stack.item {
            Item::HeroShard(base) => {
                let craft_cost = GameAssets::get(world).global_settings.craft_shards_cost;
                let r = Button::click("craft".into())
                    .enabled(self.stack.count >= craft_cost)
                    .ui(ui);
                if r.clicked() {
                    craft_hero(base.clone());
                    once_on_craft_hero(|_, _, status, hero| match status {
                        StdbStatus::Committed => {
                            Notification::new_string(format!("{hero} crafted")).push_op()
                        }
                        StdbStatus::Failed(e) => e.notify_error(),
                        _ => panic!(),
                    });
                }
                r
            }
            Item::Hero(_) => set_starting_hero_button(self.id, ui),
            Item::Lootbox => {
                let r = Button::click("open".into()).ui(ui);
                if r.clicked() {
                    open_lootbox(self.id);
                    once_on_open_lootbox(move |_, _, status, id| match status {
                        StdbStatus::Committed => {
                            let id = *id;
                            OperationsPlugin::add(move |world| {
                                Notification::new_string("Lootbox opened".into()).push(world);
                                Trade::open(id, &egui_context(world).unwrap());
                            });
                        }
                        StdbStatus::Failed(e) => e.notify_error(),
                        _ => panic!(),
                    });
                }
                r
            }
        }
    }
}
