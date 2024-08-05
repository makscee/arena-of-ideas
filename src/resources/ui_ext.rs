use super::*;

pub trait ShowTable<T> {
    fn show_table(&self, name: &'static str, ui: &mut Ui, world: &mut World) -> TableState {
        self.show_modified_table(name, ui, world, |t| t)
    }
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: fn(Table<T>) -> Table<T>,
    ) -> TableState;
}

impl ShowTable<TTeam> for Vec<TTeam> {
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: fn(Table<TTeam>) -> Table<TTeam>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .selectable()
            .column_cstr("units", |d: &TTeam| d.cstr());
        t = m(t);
        t.ui(self, ui, world)
    }
}
impl ShowTable<TBaseUnit> for Vec<TBaseUnit> {
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: fn(Table<TBaseUnit>) -> Table<TBaseUnit>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .column_cstr("name", |d: &TBaseUnit| d.name.cstr_c(name_color(&d.name)))
            .column_cstr("house", |d| {
                let color = name_color(&d.house);
                d.house.cstr_c(color)
            })
            .column_int("pwr", |d| d.pwr)
            .column_int("hp", |d| d.hp)
            .column(
                "rarity",
                |u| (u.rarity as i32).into(),
                |u, _, ui, _| Rarity::from_int(u.rarity).cstr().label(ui),
            );
        t = m(t);
        t.ui(self, ui, world)
    }
}
impl ShowTable<FusedUnit> for Vec<FusedUnit> {
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: fn(Table<FusedUnit>) -> Table<FusedUnit>,
    ) -> TableState {
        let mut t = Table::new(name).title().column(
            "name",
            |d: &FusedUnit| d.id.into(),
            |d, _, ui, _| {
                let r = d.cstr_limit(0).button(ui);
                if r.clicked() {
                    Tile::add_fused_unit(d.clone(), ui.ctx());
                }
                r
            },
        );
        t = m(t);
        t.ui(self, ui, world)
    }
}
impl ShowTable<TArenaLeaderboard> for Vec<TArenaLeaderboard> {
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: fn(Table<TArenaLeaderboard>) -> Table<TArenaLeaderboard>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .column_int("round", |d: &TArenaLeaderboard| d.round as i32)
            .column_int("score", |d| d.score as i32)
            .column_ts("time", |d| d.ts)
            .column_cstr("team", |d| {
                d.team.get_team().cstr().style(CstrStyle::Small).take()
            })
            .column_user_click(
                "owner",
                |d| d.user,
                |gid, ui, _| Tile::add_user(gid, ui.ctx()),
            )
            .column_cstr("mode", |d| d.mode.cstr());
        t = m(t);
        t.ui(self, ui, world)
    }
}
impl ShowTable<TMetaShop> for Vec<TMetaShop> {
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: fn(Table<TMetaShop>) -> Table<TMetaShop>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .column_cstr("name", |d: &TMetaShop| match &d.item {
                Item::HeroShard(name) => name.cstr_c(name_color(&name)),
                Item::Hero(unit) => unit.cstr(),
                Item::Lootbox => "normal".cstr(),
            })
            .column_cstr("type", |d| match &d.item {
                Item::HeroShard(_) => "shard".cstr(),
                Item::Hero(_) => "hero".cstr(),
                Item::Lootbox => "lootbox".cstr_c(CYAN),
            })
            .column_cstr("price", |d| d.price.to_string().cstr_c(YELLOW))
            .column_btn("buy", |d, _, _| {
                meta_buy(d.id);
                once_on_meta_buy(|_, _, status, _| match status {
                    StdbStatus::Committed => {}
                    StdbStatus::Failed(e) => e.notify_error(),
                    _ => panic!(),
                });
            });
        t = m(t);
        t.ui(self, ui, world)
    }
}
impl ShowTable<TItem> for Vec<TItem> {
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: fn(Table<TItem>) -> Table<TItem>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .column_cstr("name", |d: &TItem| match &d.item {
                Item::HeroShard(name) => name.cstr_c(name_color(&name)),
                Item::Hero(unit) => unit.cstr(),
                Item::Lootbox => "normal".cstr(),
            })
            .column_cstr("type", |d| match &d.item {
                Item::HeroShard(_) => "shard".cstr(),
                Item::Hero(_) => "hero".cstr(),
                Item::Lootbox => "lootbox".cstr_c(CYAN),
            })
            .column_int("count", |d| d.count as i32)
            .column(
                "action",
                |_| default(),
                |d, _, ui, world| {
                    let craft_cost = GameAssets::get(world).global_settings.craft_shards_cost;
                    match &d.item {
                        Item::HeroShard(base) => {
                            let r = Button::click("craft".into())
                                .enabled(d.count >= craft_cost)
                                .ui(ui);
                            if r.clicked() {
                                craft_hero(base.clone());
                                once_on_craft_hero(|_, _, status, hero| match status {
                                    StdbStatus::Committed => {
                                        Notification::new(format!("{hero} crafted")).push_op()
                                    }
                                    StdbStatus::Failed(e) => e.notify_error(),
                                    _ => panic!(),
                                });
                            }
                            r
                        }
                        Item::Hero(_) => {
                            let active = TStartingHero::get_current()
                                .map(|d| d.item_id)
                                .unwrap_or_default()
                                == d.id;
                            let r = Button::click("select".into()).active(active).ui(ui);
                            if r.clicked() {
                                let id = if active { None } else { Some(d.id) };
                                set_starting_hero(id);
                                once_on_set_starting_hero(move |_, _, status, id| match status {
                                    StdbStatus::Committed => {
                                        Notification::new(format!("{id:?} set as starting hero"))
                                            .push_op()
                                    }
                                    StdbStatus::Failed(e) => e.notify_error(),
                                    _ => panic!(),
                                });
                            }
                            r
                        }
                        Item::Lootbox => {
                            let r = Button::click("open".into()).ui(ui);
                            if r.clicked() {
                                open_lootbox(d.id);
                                once_on_open_lootbox(move |_, _, status, _| match status {
                                    StdbStatus::Committed => {
                                        Notification::new("Lootbox opened".into()).push_op()
                                    }
                                    StdbStatus::Failed(e) => e.notify_error(),
                                    _ => panic!(),
                                });
                            }
                            r
                        }
                    }
                },
            );
        t = m(t);
        t.ui(self, ui, world)
    }
}

pub trait Show {
    fn show(&self, ui: &mut Ui, world: &mut World);
}

impl Show for TUser {
    fn show(&self, ui: &mut Ui, _: &mut World) {
        text_dots_text(
            &"name".cstr(),
            &self.name.cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold),
            ui,
        );
        text_dots_text(&"id".cstr(), &self.id.to_string().cstr_c(VISIBLE_LIGHT), ui);
    }
}
impl Show for FusedUnit {
    fn show(&self, ui: &mut Ui, world: &mut World) {
        title("Fused Unit", ui);
        text_dots_text(
            &"gid".cstr(),
            &self.id.to_string().cstr_c(VISIBLE_LIGHT),
            ui,
        );
        self.bases
            .iter()
            .filter_map(|b| TBaseUnit::filter_by_name(b.clone()))
            .collect_vec()
            .show_table("Bases", ui, world);
    }
}
impl Show for TTeam {
    fn show(&self, ui: &mut Ui, world: &mut World) {
        title("Team", ui);
        text_dots_text(&"owner".cstr(), &self.owner.get_user().cstr(), ui);
        text_dots_text(
            &"gid".cstr(),
            &self.id.to_string().cstr_c(VISIBLE_LIGHT),
            ui,
        );
        ui.push_id(self.id, |ui| {
            self.units.show_table("Units", ui, world);
        });
    }
}
