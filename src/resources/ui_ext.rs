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
            .column_cstr("name", |d: &TTeam, _| d.name.cstr_c(VISIBLE_LIGHT))
            .column_cstr("units", |d, _| d.cstr());
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
            .column(
                "name",
                |d: &TBaseUnit, _| d.cstr().into(),
                |d, name, ui, world| {
                    let r = name.get_cstr().unwrap().button(ui);
                    if r.hovered() {
                        cursor_card_window(ui.ctx(), |ui| match cached_base_card(d, ui, world) {
                            Ok(_) => {}
                            Err(e) => error!("{e}"),
                        });
                    }
                    r
                },
            )
            .column_cstr("house", |d, _| {
                let color = name_color(&d.house);
                d.house.cstr_c(color)
            })
            .column_rarity(|d, _| (d.rarity as i32).into())
            .column_int("pwr", |d| d.pwr)
            .column_int("hp", |d| d.hp);
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
        let mut t = Table::new(name)
            .title()
            .column(
                "name",
                |d: &FusedUnit, _| d.cstr().into(),
                |d, _, ui, world| {
                    let r = d.cstr_limit(0).button(ui);
                    if r.clicked() {
                        TilePlugin::add_fused_unit(d.clone(), world);
                    }
                    if r.hovered() {
                        cursor_card_window(ui.ctx(), |ui| match cached_fused_card(d, ui, world) {
                            Ok(_) => {}
                            Err(e) => error!("{e}"),
                        });
                    }
                    r
                },
            )
            .column_rarity(|d, world| (Rarity::from_base(&d.bases[0], world) as i32).into())
            .column_int("lvl", |d| d.lvl as i32)
            .column_int("pwr", |d| d.pwr)
            .column_int("hp", |d| d.hp);
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
            .column_team("team", |d| d.team)
            .column_user_click(
                "owner",
                |d| d.user,
                |gid, _, world| TilePlugin::add_user(gid, world),
            )
            .column_cstr("mode", |d, _| d.mode.cstr());
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
            .column_cstr("type", |d: &TMetaShop, w| match d.item_kind {
                ItemKind::Unit => panic!("unsupported"),
                ItemKind::UnitShard => "unit shard".cstr_c(rarity_color(
                    TBaseUnit::find_by_name(d.id.get_unit_shard_item().unit)
                        .unwrap()
                        .rarity,
                )),
                ItemKind::Lootbox => "lootbox".cstr_c(CYAN),
            })
            .column_cstr("name", |d, _| match d.item_kind {
                ItemKind::Unit => panic!("unsupported"),
                ItemKind::UnitShard => TUnitShardItem::find_by_id(d.id)
                    .map(|u| u.unit.cstr_c(name_color(&u.unit)))
                    .unwrap_or_else(|| "error".cstr_c(RED)),
                ItemKind::Lootbox => TLootboxItem::find_by_id(d.id)
                    .map(|u| match u.kind {
                        LootboxKind::Regular => "Regular".cstr_c(VISIBLE_LIGHT),
                    })
                    .unwrap_or_else(|| "error".cstr_c(RED)),
            })
            .column_cstr("price", |d, _| {
                format!("{} {CREDITS_SYM}", d.price).cstr_c(YELLOW)
            })
            .column(
                "buy",
                |_, _| default(),
                |d, _, ui, _| {
                    let r = Button::click("buy".into())
                        .enabled(can_afford(d.price))
                        .ui(ui);
                    if r.clicked() {
                        meta_buy(d.id);
                        once_on_meta_buy(|_, _, status, _| match status {
                            StdbStatus::Committed => {}
                            StdbStatus::Failed(e) => e.notify_error_op(),
                            _ => panic!(),
                        });
                    }
                    r
                },
            );
        t = m(t);
        t.ui(self, ui, world)
    }
}
impl ShowTable<TUnitShardItem> for Vec<TUnitShardItem> {
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: fn(Table<TUnitShardItem>) -> Table<TUnitShardItem>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .column(
                "name",
                |d: &TUnitShardItem, _| d.unit.cstr_c(name_color(&d.unit)).into(),
                |d, name, ui, world| {
                    let r = name.get_cstr().unwrap().button(ui);
                    if r.hovered() {
                        cursor_card_window(ui.ctx(), |ui| {
                            match cached_base_card(
                                &TBaseUnit::find_by_name(d.unit.clone()).unwrap(),
                                ui,
                                world,
                            ) {
                                Ok(_) => {}
                                Err(e) => error!("{e}"),
                            }
                        });
                    }
                    r
                },
            )
            .column_rarity(|d, world| (Rarity::from_base(&d.unit, world) as i32).into())
            .column_int("count", |d| d.count as i32);
        t = m(t);
        t.ui(self, ui, world)
    }
}
impl ShowTable<TLootboxItem> for Vec<TLootboxItem> {
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: fn(Table<TLootboxItem>) -> Table<TLootboxItem>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .column_cstr("kind", |d: &TLootboxItem, _| match d.kind {
                LootboxKind::Regular => "Regular".cstr_c(VISIBLE_LIGHT),
            })
            .column_int("count", |d| d.count as i32);
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
            "name".cstr(),
            self.name.cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold),
            ui,
        );
        text_dots_text("id".cstr(), self.id.to_string().cstr_c(VISIBLE_LIGHT), ui);
    }
}
impl Show for FusedUnit {
    fn show(&self, ui: &mut Ui, world: &mut World) {
        title("Fused Unit", ui);
        text_dots_text("gid".cstr(), self.id.to_string().cstr_c(VISIBLE_LIGHT), ui);
        self.bases
            .iter()
            .filter_map(|b| TBaseUnit::find_by_name(b.clone()))
            .collect_vec()
            .show_table("Bases", ui, world);
    }
}
impl Show for TTeam {
    fn show(&self, ui: &mut Ui, world: &mut World) {
        title("Team", ui);
        text_dots_text("owner".cstr(), self.owner.get_user().cstr(), ui);
        text_dots_text("gid".cstr(), self.id.to_string().cstr_c(VISIBLE_LIGHT), ui);
        ui.push_id(self.id, |ui| {
            self.units.show_table("Units", ui, world);
        });
    }
}
