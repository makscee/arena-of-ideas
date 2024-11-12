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
        m: impl Fn(Table<T>) -> Table<T>,
    ) -> TableState;
}

impl ShowTable<TTeam> for Vec<TTeam> {
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: impl Fn(Table<TTeam>) -> Table<TTeam>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .column_cstr("name", |d: &TTeam, _| d.name.cstr_c(VISIBLE_LIGHT))
            .column_cstr("units", |d, _| {
                d.units
                    .iter()
                    .map(|u| u.cstr_limit(1, false))
                    .collect_vec()
                    .join(" ".cstr())
            });
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
        m: impl Fn(Table<TBaseUnit>) -> Table<TBaseUnit>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .column(
                "name",
                |d: &TBaseUnit, _| d.cstr().into(),
                |d, name, ui, world| {
                    let r = name.get_cstr().unwrap().button(ui);
                    if r.hovered() {
                        cursor_window(ui.ctx(), |ui| match cached_base_card(d, ui, world) {
                            Ok(_) => {}
                            Err(e) => error!("{e}"),
                        });
                    }
                    r.context_menu(|ui| {
                        ui.reset_style();
                        if Button::click("Copy").ui(ui).clicked() {
                            match ron::to_string(&PackedUnit::from(d.clone())) {
                                Ok(v) => {
                                    copy_to_clipboard(&v, world);
                                }
                                Err(e) => format!("Failed to copy: {e}").notify_error(world),
                            }
                            ui.close_menu();
                        }
                    });
                },
                true,
            )
            .column_cstr("house", |d, _| {
                let color = name_color(&d.house);
                d.house.cstr_c(color)
            })
            .column_rarity(|d| (d.rarity as i32).into())
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
        m: impl Fn(Table<FusedUnit>) -> Table<FusedUnit>,
    ) -> TableState {
        fn format_stat(value: i32, mutation: i32) -> Cstr {
            match mutation.signum() {
                -1 => mutation
                    .to_string()
                    .cstr_c(RED)
                    .push(format!(" ({value})").cstr())
                    .take(),
                1 => format!("+{mutation}")
                    .cstr_c(GREEN)
                    .push(format!(" ({value})").cstr())
                    .take(),
                _ => format!("{mutation} ({value})").cstr(),
            }
        }
        let mut t = Table::new(name)
            .title()
            .column(
                "name",
                |d: &FusedUnit, _| d.cstr().into(),
                |d, _, ui, world| {
                    let r = d.cstr_limit(3, true).button(ui);
                    if r.clicked() {
                        TilePlugin::add_fused_unit(d.clone(), world);
                    }
                    if r.hovered() {
                        cursor_window(ui.ctx(), |ui| match cached_fused_card(d, ui, world) {
                            Ok(_) => {}
                            Err(e) => error!("{e}"),
                        });
                    }
                    r.context_menu(|ui| {
                        ui.reset_style();
                        if Button::click("Copy").ui(ui).clicked() {
                            match ron::to_string(&PackedUnit::from(d.clone())) {
                                Ok(v) => {
                                    copy_to_clipboard(&v, world);
                                }
                                Err(e) => format!("Failed to copy: {e}").notify_error(world),
                            }
                            ui.close_menu();
                        }
                    });
                },
                true,
            )
            .column_cstr("house", |d, _| {
                let house = d.base_unit().house;
                let color = name_color(&house);
                house.cstr_c(color)
            })
            .column_rarity(|d| (Rarity::from_base(&d.bases[0]) as i32).into())
            .column_int("lvl", |d| d.lvl as i32)
            .column_cstr_value(
                "pwr",
                |u| u.pwr_mutation.into(),
                |u, _| format_stat(u.pwr, u.pwr_mutation),
            )
            .column_cstr_value(
                "hp",
                |u| u.hp_mutation.into(),
                |u, _| format_stat(u.hp, u.hp_mutation),
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
        m: impl Fn(Table<TArenaLeaderboard>) -> Table<TArenaLeaderboard>,
    ) -> TableState {
        let mut t = Table::new(name)
            .column_int("flr", |d: &TArenaLeaderboard| d.floor as i32)
            .column_user_click("owner", |d| d.owner)
            .column_team("team", |d| d.team)
            .column_ts("time", |d| d.ts)
            .column_cstr("mode", |d, _| d.mode.cstr_expanded());
        t = m(t);
        t.ui(self, ui, world)
    }
}
impl ShowTable<TArenaRunArchive> for Vec<TArenaRunArchive> {
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: impl Fn(Table<TArenaRunArchive>) -> Table<TArenaRunArchive>,
    ) -> TableState {
        let mut t = Table::new(name)
            .column_int("flr", |d: &TArenaRunArchive| d.floor as i32)
            .column_user_click("owner", |d| d.owner)
            .column_team("team", |d| d.team)
            .column_gid("id", |d| d.id)
            .column_ts("time", |d| d.ts)
            .column_cstr("mode", |d, _| d.mode.cstr_expanded());
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
        m: impl Fn(Table<TMetaShop>) -> Table<TMetaShop>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .columns_item_kind(|d: &TMetaShop| (d.item_kind.clone(), d.id))
            .column_cstr("price", |d, _| {
                let mut price = d.price;
                if !cn().db.daily_state().current().meta_shop_discount_spent {
                    price = (price as f32 * global_settings().meta.daily_discount) as i64;
                }
                format!("{} {CREDITS_SYM}", price).cstr_c(YELLOW)
            })
            .column(
                "buy",
                |_, _| default(),
                |d, _, ui, _| {
                    let price = (d.price as f32
                        * if !cn().db.daily_state().current().meta_shop_discount_spent {
                            global_settings().meta.daily_discount
                        } else {
                            1.0
                        }) as i64;
                    if Button::click("buy")
                        .enabled(can_afford(price))
                        .ui(ui)
                        .clicked()
                    {
                        cn().reducers.meta_buy(d.id).unwrap();
                    }
                },
                false,
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
        m: impl Fn(Table<TUnitShardItem>) -> Table<TUnitShardItem>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .column_base_unit_name("unit", |d: &TUnitShardItem| d.unit.clone())
            .column_cstr("house", |d, _| {
                let house = d.unit.base_unit().house;
                let color = name_color(&house);
                house.cstr_c(color)
            })
            .column_rarity(|d| (Rarity::from_base(&d.unit) as i32).into())
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
        m: impl Fn(Table<TLootboxItem>) -> Table<TLootboxItem>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .column_cstr("kind", |d: &TLootboxItem, _| match &d.kind {
                LootboxKind::Regular => "Regular".cstr_c(VISIBLE_LIGHT),
                LootboxKind::House(house) => house.cstr_c(name_color(&house)),
            })
            .column_int("count", |d| d.count as i32);
        t = m(t);
        t.ui(self, ui, world)
    }
}
impl ShowTable<TAuction> for Vec<TAuction> {
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: impl Fn(Table<TAuction>) -> Table<TAuction>,
    ) -> TableState {
        fn count(d: &TAuction) -> i32 {
            match d.item_kind {
                ItemKind::Unit => 1,
                ItemKind::UnitShard => d.item_id.unit_shard_item().count as i32,
                ItemKind::Lootbox => d.item_id.lootbox_item().count as i32,
                ItemKind::RainbowShard => d.item_id.rainbow_shard_item().count as i32,
            }
        }
        let mut t = Table::new(name)
            .title()
            .columns_item_kind(|d: &TAuction| (d.item_kind.clone(), d.item_id))
            .column_int("count", |d| count(d))
            .column_cstr("price", |d, _| {
                format!("{} {CREDITS_SYM}", d.price).cstr_c(YELLOW)
            })
            .column_cstr_value(
                "price",
                |d| (d.price as i32).into(),
                |_, v| format!("{} {CREDITS_SYM}", v.get_int().unwrap()).cstr_c(YELLOW),
            )
            .column_cstr_value(
                "unit price",
                |d| (d.price as f32 / count(d) as f32).into(),
                |_, v| format!("{} {CREDITS_SYM}", v.get_float().unwrap()).cstr_c(YELLOW),
            )
            .column_user_click("seller", |d| d.owner);
        t = m(t);
        t.ui(self, ui, world)
    }
}

pub trait Show {
    fn show(&self, ui: &mut Ui, world: &mut World);
}

impl Show for TPlayer {
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
            .map(|u| u.base_unit())
            .collect_vec()
            .show_table("Bases", ui, world);
    }
}
impl Show for TTeam {
    fn show(&self, ui: &mut Ui, world: &mut World) {
        title("Team", ui);
        if !self.name.is_empty() {
            text_dots_text("name".cstr(), self.name.cstr_c(VISIBLE_LIGHT), ui);
        }
        text_dots_text("owner".cstr(), self.owner.get_player().cstr(), ui);
        text_dots_text("gid".cstr(), self.id.to_string().cstr_c(VISIBLE_LIGHT), ui);
        ui.push_id(self.id, |ui| {
            self.units.show_table("Units", ui, world);
        });
    }
}
impl Show for ItemBundle {
    fn show(&self, ui: &mut Ui, world: &mut World) {
        let units = self
            .units
            .iter()
            .map(|id| id.unit_item().unit)
            .collect_vec();
        if !units.is_empty() {
            units.show_table("Units", ui, world);
        }
        let unit_shards = self
            .unit_shards
            .iter()
            .map(|id| id.unit_shard_item())
            .collect_vec();
        if !unit_shards.is_empty() {
            unit_shards.show_table("Unit Shards", ui, world);
        }
        let lootboxes = self
            .lootboxes
            .iter()
            .map(|id| id.lootbox_item())
            .collect_vec();
        if !lootboxes.is_empty() {
            lootboxes.show_table("Lootboxes", ui, world);
        }
        if self.credits != 0 {
            text_dots_text(
                "credits".cstr(),
                format!("{}{}", self.credits, CREDITS_SYM).cstr_cs(YELLOW, CstrStyle::Bold),
                ui,
            );
        }
    }
}
