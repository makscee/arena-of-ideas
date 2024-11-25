use super::*;

impl<T: 'static + Clone + Send + Sync> Table<T> {
    pub fn add_team_columns(self, f: fn(&T) -> &TTeam) -> Self {
        self.column_cstr_dyn(
            "name",
            Box::new(move |d: &T, _| f(d).name.cstr_c(VISIBLE_LIGHT)),
        )
        .column_cstr_dyn(
            "units",
            Box::new(move |d, _| f(d).units.iter().map(|u| u.cstr_limit(1, false)).join(" ")),
        )
    }
    pub fn add_base_unit_columns(self, f: fn(&T) -> &TBaseUnit) -> Self {
        self.row_height(60.0)
            .column_base_unit_texture(f)
            .column_dyn(
                "name",
                Box::new(move |d, _| f(d).cstr().into()),
                Box::new(move |d, name, ui, world| {
                    let unit = f(d).clone();
                    let r = name.get_string().unwrap().button(ui);
                    if r.hovered() {
                        cursor_window(ui.ctx(), |ui| match cached_base_card(&unit, ui, world) {
                            Ok(_) => {}
                            Err(e) => error!("{e}"),
                        });
                    }
                    r.context_menu(|ui| {
                        ui.reset_style();
                        if Button::new("Copy").ui(ui).clicked() {
                            match ron::to_string(&PackedUnit::from(unit)) {
                                Ok(v) => {
                                    copy_to_clipboard(&v, world);
                                }
                                Err(e) => format!("Failed to copy: {e}").notify_error(world),
                            }
                            ui.close_menu();
                        }
                    });
                }),
                true,
            )
            .column_cstr_dyn(
                "house",
                Box::new(move |d, _| {
                    let unit = f(d);
                    let color = name_color(&unit.house);
                    unit.house.cstr_c(color)
                }),
            )
            .column_rarity_dyn(Box::new(move |d| (f(d).rarity as i32).into()))
            .column_int_dyn("pwr", Box::new(move |d| f(d).pwr))
            .column_int_dyn("hp", Box::new(move |d| f(d).hp))
    }
    pub fn add_fused_unit_columns(self, f: fn(&T) -> &FusedUnit) -> Self {
        fn format_stat(value: i32, mutation: i32) -> Cstr {
            match mutation.signum() {
                -1 => mutation.to_string().cstr_c(RED) + &format!(" ({value})"),
                1 => format!("+{mutation}").cstr_c(GREEN) + &format!(" ({value})"),
                _ => format!("{mutation} ({value})"),
            }
        }
        self.column_dyn(
            "name",
            Box::new(move |d, _| f(d).cstr().into()),
            Box::new(move |d, _, ui, world| {
                let d = f(d);
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
                    if Button::new("Copy").ui(ui).clicked() {
                        match ron::to_string(&PackedUnit::from(d.clone())) {
                            Ok(v) => {
                                copy_to_clipboard(&v, world);
                            }
                            Err(e) => format!("Failed to copy: {e}").notify_error(world),
                        }
                        ui.close_menu();
                    }
                });
            }),
            true,
        )
        .column_cstr_dyn(
            "house",
            Box::new(move |d, _| {
                let house = f(d).base_unit().house;
                let color = name_color(&house);
                house.cstr_c(color)
            }),
        )
        .column_rarity_dyn(Box::new(move |d| {
            (Rarity::from_base(&f(d).bases[0]) as i32).into()
        }))
        .column_int_dyn("lvl", Box::new(move |d| f(d).lvl as i32))
        .column_cstr_value_dyn(
            "pwr",
            Box::new(move |u| f(u).pwr_mutation.into()),
            Box::new(move |u, _| {
                let u = f(u);
                format_stat(u.pwr, u.pwr_mutation)
            }),
        )
        .column_cstr_value_dyn(
            "hp",
            Box::new(move |u| f(u).hp_mutation.into()),
            Box::new(move |u, _| {
                let u = f(u);
                format_stat(u.hp, u.hp_mutation)
            }),
        )
    }
    pub fn add_arena_leaderboard_columns(self, f: fn(&T) -> &TArenaLeaderboard) -> Self {
        self.column_int_dyn("flr", Box::new(move |d| f(d).floor as i32))
            .column_player_click_dyn("owner", Box::new(move |d| f(d).owner))
            .column_team_dyn("team", Box::new(move |d| f(d).team))
            .column_ts_dyn("time", Box::new(move |d| f(d).ts))
            .column_cstr_dyn("mode", Box::new(move |d, _| f(d).mode.cstr_expanded()))
    }
    pub fn add_arena_run_archive_columns(self, f: fn(&T) -> &TArenaRunArchive) -> Self {
        self.column_int_dyn("flr", Box::new(move |d| f(d).floor as i32))
            .column_player_click_dyn("owner", Box::new(move |d| f(d).owner))
            .column_team_dyn("team", Box::new(move |d| f(d).team))
            .column_id_dyn("id", Box::new(move |d| f(d).id))
            .column_ts_dyn("time", Box::new(move |d| f(d).ts))
            .column_cstr_dyn("mode", Box::new(move |d, _| f(d).mode.cstr_expanded()))
    }
    pub fn add_item_kind_columns(self, f: fn(&T) -> (ItemKind, u64)) -> Self {
        self.column_dyn(
            "type",
            Box::new(move |d, _| {
                let (kind, id) = f(d);
                match kind {
                    ItemKind::Unit => {
                        "unit".cstr_c(rarity_color(id.unit_item().unit.base_unit().rarity))
                    }
                    ItemKind::UnitShard => "unit shard"
                        .cstr_c(rarity_color(id.unit_shard_item().unit.base_unit().rarity)),
                    ItemKind::Lootbox => "lootbox".cstr_c(CYAN),
                    ItemKind::RainbowShard => "rainbow shard".cstr_rainbow(),
                }
                .into()
            }),
            Box::new(|_, v, ui, _| {
                v.get_string().unwrap().label(ui);
            }),
            true,
        )
        .column_dyn(
            "name",
            Box::new(move |d, _| {
                let (kind, _) = f(d);
                match kind {
                    ItemKind::Unit => "unit",
                    ItemKind::UnitShard => "shard",
                    ItemKind::Lootbox => "lootbox",
                    ItemKind::RainbowShard => "rainbow shard",
                }
                .into()
            }),
            Box::new(move |d, _, ui, world| {
                let (kind, id) = f(d);
                match kind {
                    ItemKind::Unit => {
                        let unit = id.unit_item().unit;
                        let r = unit.cstr_limit(0, true).button(ui);
                        if r.hovered() {
                            cursor_window(ui.ctx(), |ui| {
                                match cached_fused_card(&unit, ui, world) {
                                    Ok(_) => {}
                                    Err(e) => error!("{e}"),
                                }
                            });
                        }
                        if r.clicked() {
                            TilePlugin::add_fused_unit(unit, world);
                        }
                    }
                    ItemKind::UnitShard => {
                        let item = id.unit_shard_item();
                        let r = item.unit.cstr_c(name_color(&item.unit)).label(ui);
                        if r.hovered() {
                            cursor_window(ui.ctx(), |ui| {
                                match cached_base_card(&item.unit.base_unit(), ui, world) {
                                    Ok(_) => {}
                                    Err(e) => error!("{e}"),
                                }
                            });
                        }
                    }
                    ItemKind::Lootbox => {
                        match &id.lootbox_item().kind {
                            LootboxKind::Regular => "Regular".cstr_c(VISIBLE_LIGHT),
                            LootboxKind::House(house) => house.cstr_c(name_color(house)),
                        }
                        .label(ui);
                    }
                    ItemKind::RainbowShard => default(),
                }
            }),
            true,
        )
    }
    pub fn add_meta_shop_columns(self, f: fn(&T) -> &TMetaShop) -> Self {
        self.column_cstr_dyn(
            "price",
            Box::new(move |d, _| {
                let mut price = f(d).price;
                if !cn().db.daily_state().current().meta_shop_discount_spent {
                    price = (price as f32 * global_settings().meta.daily_discount) as i64;
                }
                format!("{} {CREDITS_SYM}", price).cstr_c(YELLOW)
            }),
        )
        .column_btn_mod_dyn(
            "buy",
            Box::new(move |d, _, _| cn().reducers.meta_buy(f(d).id).unwrap()),
            Box::new(move |d, _, button| {
                let price = (f(d).price as f32
                    * if !cn().db.daily_state().current().meta_shop_discount_spent {
                        global_settings().meta.daily_discount
                    } else {
                        1.0
                    }) as i64;

                button.enabled(can_afford(price))
            }),
        )
    }
    pub fn add_unit_shard_item_columns(self, f: fn(&T) -> &TUnitShardItem) -> Self {
        self.column_base_unit_name_dyn("unit", Box::new(move |d| f(d).unit.clone()))
            .column_cstr_dyn(
                "house",
                Box::new(move |d, _| {
                    let house = f(d).unit.base_unit().house;
                    let color = name_color(&house);
                    house.cstr_c(color)
                }),
            )
            .column_rarity_dyn(Box::new(move |d| {
                (Rarity::from_base(&f(d).unit) as i32).into()
            }))
            .column_int_dyn("count", Box::new(move |d| f(d).count as i32))
    }
    pub fn add_lootbox_item_columns(self, f: fn(&T) -> &TLootboxItem) -> Self {
        self.column_cstr_dyn(
            "kind",
            Box::new(move |d, _| match &f(d).kind {
                LootboxKind::Regular => "Regular".cstr_c(VISIBLE_LIGHT),
                LootboxKind::House(house) => house.cstr_c(name_color(&house)),
            }),
        )
        .column_int_dyn("count", Box::new(move |d| f(d).count as i32))
    }
    pub fn add_auction_columns(self, f: fn(&T) -> &TAuction) -> Self {
        fn count(d: &TAuction) -> i32 {
            match d.item_kind {
                ItemKind::Unit => 1,
                ItemKind::UnitShard => d.item_id.unit_shard_item().count as i32,
                ItemKind::Lootbox => d.item_id.lootbox_item().count as i32,
                ItemKind::RainbowShard => d.item_id.rainbow_shard_item().count as i32,
            }
        }
        self.column_int_dyn("count", Box::new(move |d| count(f(d))))
            .column_cstr_dyn(
                "price",
                Box::new(move |d, _| format!("{} {CREDITS_SYM}", f(d).price).cstr_c(YELLOW)),
            )
            .column_cstr_value_dyn(
                "price",
                Box::new(move |d| (f(d).price as i32).into()),
                Box::new(move |_, v| {
                    format!("{} {CREDITS_SYM}", v.get_int().unwrap()).cstr_c(YELLOW)
                }),
            )
            .column_cstr_value_dyn(
                "unit price",
                Box::new(move |d| (f(d).price as f32 / count(f(d)) as f32).into()),
                Box::new(move |_, v| {
                    format!("{} {CREDITS_SYM}", v.get_float().unwrap()).cstr_c(YELLOW)
                }),
            )
            .column_player_click_dyn("seller", Box::new(move |d| f(d).owner))
    }
    pub fn add_content_piece_columns(self, f: fn(&T) -> TContentPiece) -> Self {
        self.column_id_dyn("id", Box::new(move |d| f(d).id))
            .column_player_click_dyn("owner", Box::new(move |d| f(d).owner))
            .column_dyn(
                "data",
                Box::new(|_, _| default()),
                Box::new(move |d, _, ui, world| {
                    let d = f(d);
                    d.t.to_local().show(&d.data, ui, world);
                }),
                false,
            )
    }
    pub fn add_content_vote_columns(self, f: fn(&T) -> String) -> Self {
        self.column_int_dyn(
            "score",
            Box::new(move |d| {
                let id = f(d);
                cn().db
                    .content_vote_score()
                    .id()
                    .find(&id)
                    .map(|l| l.score)
                    .unwrap_or_default()
            }),
        )
        .column_btn_mod_dyn(
            "+",
            Box::new(move |d, _, _| {
                cn().reducers.incubator_vote(f(d), true).unwrap();
            }),
            Box::new(move |d, _, b| b.active(IncubatorPlugin::get_vote(player_id(), &f(d)) == 1)),
        )
        .column_btn_mod_dyn(
            "-",
            Box::new(move |d, _, _| {
                cn().reducers.incubator_vote(f(d), false).unwrap();
            }),
            Box::new(move |d, ui, b| {
                b.active(IncubatorPlugin::get_vote(player_id(), &f(d)) == -1)
                    .red(ui)
            }),
        )
    }
    pub fn add_content_favorite_columns(self, f: fn(&T) -> (String, String)) -> Self {
        self.column_int_dyn(
            "fav",
            Box::new(move |d| {
                let (type_key, target) = f(d);
                let key = format!("{type_key}_{target}");
                cn().db
                    .content_favorite_score()
                    .type_target()
                    .find(&key)
                    .map(|d| d.score as i32)
                    .unwrap_or_default()
            }),
        )
        .column_btn_mod_dyn(
            "♥",
            Box::new(move |d, _, _| {
                let (t, target) = f(d);
                cn().reducers.incubator_favorite(t, target).unwrap();
            }),
            Box::new(move |d, _, b| {
                let (type_key, target) = f(d);
                let owner_type = format!("{}_{type_key}", player_id());
                if let Some(fav) = cn().db.content_favorite().owner_type().find(&owner_type) {
                    b.active(fav.target == target)
                } else {
                    b
                }
            }),
        )
    }
}
