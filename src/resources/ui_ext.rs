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
            .column_cstr("name", |d: &TTeam, _| d.name.cstr_c(visible_light()))
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
        m: fn(Table<TBaseUnit>) -> Table<TBaseUnit>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .column(
                "name",
                |d: &TBaseUnit, _| d.cstr().into(),
                |d, name, ui, world| {
                    if name.get_cstr().unwrap().button(ui).hovered() {
                        cursor_window(ui.ctx(), |ui| match cached_base_card(d, ui, world) {
                            Ok(_) => {}
                            Err(e) => error!("{e}"),
                        });
                    }
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
        m: fn(Table<FusedUnit>) -> Table<FusedUnit>,
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
                },
                true,
            )
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
        m: fn(Table<TArenaLeaderboard>) -> Table<TArenaLeaderboard>,
    ) -> TableState {
        let mut t = Table::new(name)
            .column_int("floor", |d: &TArenaLeaderboard| d.floor as i32)
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
impl ShowTable<TArenaRunArchive> for Vec<TArenaRunArchive> {
    fn show_modified_table(
        &self,
        name: &'static str,
        ui: &mut Ui,
        world: &mut World,
        m: fn(Table<TArenaRunArchive>) -> Table<TArenaRunArchive>,
    ) -> TableState {
        let mut t = Table::new(name)
            .column_gid("id", |d: &TArenaRunArchive| d.id)
            .column_int("floor", |d| d.floor as i32)
            .column_team("team", |d| d.team)
            .column_user_click(
                "owner",
                |d| d.owner,
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
            .columns_item_kind(|d: &TMetaShop| (d.item_kind.clone(), d.id))
            .column_cstr("price", |d, _| {
                format!("{} {CREDITS_SYM}", d.price).cstr_c(YELLOW)
            })
            .column(
                "buy",
                |_, _| default(),
                |d, _, ui, _| {
                    if Button::click("buy")
                        .enabled(can_afford(d.price))
                        .ui(ui)
                        .clicked()
                    {
                        meta_buy(d.id);
                        once_on_meta_buy(|_, _, status, _| match status {
                            StdbStatus::Committed => {}
                            StdbStatus::Failed(e) => e.notify_error_op(),
                            _ => panic!(),
                        });
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
        m: fn(Table<TUnitShardItem>) -> Table<TUnitShardItem>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .column_base_unit("unit", |d: &TUnitShardItem| d.unit.clone())
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
        m: fn(Table<TLootboxItem>) -> Table<TLootboxItem>,
    ) -> TableState {
        let mut t = Table::new(name)
            .title()
            .column_cstr("kind", |d: &TLootboxItem, _| match d.kind {
                LootboxKind::Regular => "Regular".cstr_c(visible_light()),
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
        m: fn(Table<TAuction>) -> Table<TAuction>,
    ) -> TableState {
        fn count(d: &TAuction) -> i32 {
            match d.item_kind {
                ItemKind::Unit => 1,
                ItemKind::UnitShard => d.item_id.unit_shard_item().count as i32,
                ItemKind::Lootbox => d.item_id.lootbox_item().count as i32,
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
            .column_user_click(
                "seller",
                |d| d.owner,
                |gid, _, world| TilePlugin::add_user(gid, world),
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
            "name".cstr(),
            self.name.cstr_cs(visible_light(), CstrStyle::Bold),
            ui,
        );
        text_dots_text("id".cstr(), self.id.to_string().cstr_c(visible_light()), ui);
    }
}
impl Show for FusedUnit {
    fn show(&self, ui: &mut Ui, world: &mut World) {
        title("Fused Unit", ui);
        text_dots_text(
            "gid".cstr(),
            self.id.to_string().cstr_c(visible_light()),
            ui,
        );
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
            text_dots_text("name".cstr(), self.name.cstr_c(visible_light()), ui);
        }
        text_dots_text("owner".cstr(), self.owner.get_user().cstr(), ui);
        text_dots_text(
            "gid".cstr(),
            self.id.to_string().cstr_c(visible_light()),
            ui,
        );
        ui.push_id(self.id, |ui| {
            self.units.show_table("Units", ui, world);
        });
    }
}
