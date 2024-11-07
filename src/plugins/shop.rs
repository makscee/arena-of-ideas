use bevy::input::common_conditions::input_just_pressed;
use spacetimedb_sdk::table::TableWithPrimaryKey;

use super::*;

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shop), Self::enter)
            .add_systems(OnExit(GameState::Shop), Self::exit)
            .init_resource::<ShopResource>();
        if cfg!(debug_assertions) {
            app.add_systems(
                Update,
                Self::give_g.run_if(input_just_pressed(KeyCode::KeyG)),
            )
            .add_systems(Update, Self::test.run_if(input_just_pressed(KeyCode::KeyT)));
        }
    }
}

#[derive(Resource, Default)]
pub struct ShopResource {
    pub case_height: f32,
    callback: Option<ArenaRunUpdateCallbackId>,
    stack_source: Option<(usize, Faction)>,
    stack_targets: Vec<usize>,
    fuse_source: Option<usize>,
    fuse_targets: Vec<usize>,
    family_slot: Option<usize>,
    fusion_choice: Vec<i8>,
    fusion_cards: Vec<UnitCard>,
    queued_notifications: Vec<Cstr>,
}
fn rm(world: &mut World) -> Mut<ShopResource> {
    world.resource_mut::<ShopResource>()
}

impl ShopPlugin {
    fn give_g() {
        cn().reducers.shop_change_g(10).unwrap();
    }
    fn test(world: &mut World) {
        TeamPlugin::change_ability_var_int("Siphon".into(), VarName::M1, 1, Faction::Team, world);
    }
    fn enter(mut sd: ResMut<ShopResource>) {
        for text in sd.queued_notifications.drain(..) {
            Notification::new(text).push_op();
        }
        if let Some(run) = cn().db.arena_run().get_current() {
            if !run.active {
                GameState::GameOver.set_next_op();
                return;
            }
            OperationsPlugin::add(|world| Self::sync_run(*run, world));
        } else {
            "No active run found".notify_error_op();
            GameState::Title.proceed_to_target_op();
            return;
        }
        AudioPlugin::queue_sound(SoundEffect::StartGame);
        let cb = cn().db.arena_run().on_update(|_, _, run| {
            let run = run.clone();
            OperationsPlugin::add(|w| Self::sync_run(run, w))
        });
        sd.callback = Some(cb);
    }
    fn exit(world: &mut World) {
        debug!("Shop exit");
        world.game_clear();
        if let Some(cb) = rm(world).callback.take() {
            debug!("Unsubscribe shop updater");
            cn().db.arena_run().remove_on_update(cb);
        }
    }
    fn sync_fusion(a: usize, b: usize, run: &TArenaRun, world: &mut World) {
        let team = run.team.get_team();
        let left_card = UnitCard::from_fused(team.units[a].clone(), world).unwrap();
        let right_card = UnitCard::from_fused(team.units[b].clone(), world).unwrap();
        let mut r = rm(world);
        r.fusion_cards = [left_card.clone(), left_card, right_card].into();
        r.fusion_choice = [-1, 1, 0].into();
        fn update_result_card(world: &mut World) {
            let run = cn().db.arena_run().current();
            let fusion = run.fusion.unwrap();
            let mut unit = fusion.unit;
            let r = rm(world);
            unit.triggers = fusion.triggers[(r.fusion_choice[0] + 1) as usize].clone();
            unit.targets = fusion.targets[(r.fusion_choice[1] + 1) as usize].clone();
            unit.effects = fusion.effects[(r.fusion_choice[2] + 1) as usize].clone();
            let card = UnitCard::from_fused(unit, world).unwrap();
            rm(world).fusion_cards[1] = card;
        }
        update_result_card(world);

        Confirmation::new("Fusion".cstr_c(YELLOW))
            .accept(|world| {
                let fc = rm(world).fusion_choice.clone();
                cn().reducers.fuse_choose(fc[0], fc[1], fc[2]).unwrap();
            })
            .cancel(|_| {
                cn().reducers.fuse_cancel().unwrap();
            })
            .content(|ui, world| {
                if Button::click("Swap").ui(ui).clicked() {
                    Confirmation::close_current(world);
                    cn().reducers.fuse_swap().unwrap();
                }
                ui.set_width(ui.ctx().screen_rect().width() * 0.9);
                ui.columns(3, |ui| {
                    let r = rm(world);
                    ui[0].vertical_centered_justified(|ui| {
                        "Left".cstr_cs(VISIBLE_LIGHT, CstrStyle::Heading2).label(ui);
                    });
                    ui[1].vertical_centered_justified(|ui| {
                        "Result"
                            .cstr_cs(VISIBLE_LIGHT, CstrStyle::Heading2)
                            .label(ui);
                    });
                    ui[2].vertical_centered_justified(|ui| {
                        "Right"
                            .cstr_cs(VISIBLE_LIGHT, CstrStyle::Heading2)
                            .label(ui);
                    });
                    for i in 0..3 {
                        r.fusion_cards[i].ui(&mut ui[i]);
                    }
                });
                ui.columns(3, |ui| {
                    let mut r = rm(world);
                    let mut need_update = false;
                    for i in 0..3 {
                        ui[i as usize].vertical_centered_justified(|ui| {
                            if Button::click("Trigger")
                                .active(r.fusion_choice[0] == i - 1)
                                .ui(ui)
                                .clicked()
                            {
                                need_update = true;
                                if r.fusion_choice[1] == i - 1 {
                                    r.fusion_choice[1] = r.fusion_choice[0];
                                } else if r.fusion_choice[2] == i - 1 {
                                    r.fusion_choice[2] = r.fusion_choice[0];
                                }
                                r.fusion_choice[0] = i - 1;
                            }
                            if Button::click("Target")
                                .active(r.fusion_choice[1] == i - 1)
                                .ui(ui)
                                .clicked()
                            {
                                need_update = true;
                                if r.fusion_choice[0] == i - 1 {
                                    r.fusion_choice[0] = r.fusion_choice[1];
                                } else if r.fusion_choice[2] == i - 1 {
                                    r.fusion_choice[2] = r.fusion_choice[1];
                                }
                                r.fusion_choice[1] = i - 1;
                            }
                            if Button::click("Effect")
                                .active(r.fusion_choice[2] == i - 1)
                                .ui(ui)
                                .clicked()
                            {
                                need_update = true;
                                if r.fusion_choice[0] == i - 1 {
                                    r.fusion_choice[0] = r.fusion_choice[2];
                                } else if r.fusion_choice[1] == i - 1 {
                                    r.fusion_choice[1] = r.fusion_choice[2];
                                }
                                r.fusion_choice[2] = i - 1;
                            }
                        });
                    }
                    if need_update {
                        update_result_card(world);
                    }
                });
            })
            .push(world);
    }
    fn sync_run(run: TArenaRun, world: &mut World) {
        debug!("Sync run");
        let mut shop_units: HashMap<u64, Entity> = HashMap::from_iter(
            UnitPlugin::collect_faction(Faction::Shop, world)
                .into_iter()
                .map(|e| (VarState::get(e, world).id(), e)),
        );
        let team = TeamPlugin::entity(Faction::Shop, world);
        if let Some(Fusion {
            unit: _,
            triggers: _,
            targets: _,
            effects: _,
            a,
            b,
        }) = run.fusion
        {
            Self::sync_fusion(a as usize, b as usize, &run, world);
        } else {
            let mut sd = rm(world);
            sd.fuse_source = None;
            sd.fuse_targets.clear();
            sd.family_slot = None;

            for (
                slot,
                ShopSlot {
                    unit,
                    available,
                    id,
                    house_filter,
                    ..
                },
            ) in (0..).zip(run.shop_slots.into_iter())
            {
                if !house_filter.is_empty() {
                    rm(world).family_slot = Some(slot as usize);
                }
                if available {
                    if shop_units.contains_key(&id) {
                        shop_units.remove(&id);
                        continue;
                    }
                    game_assets().heroes.get(&unit).cloned().unwrap().unpack(
                        team,
                        Some(slot),
                        Some(id),
                        world,
                    );
                }
            }
        }
        for entity in shop_units.into_values() {
            UnitPlugin::despawn(entity, world);
        }
        let mut team_units: HashMap<u64, Entity> = HashMap::from_iter(
            UnitPlugin::collect_faction(Faction::Team, world)
                .into_iter()
                .map(|e| (VarState::get(e, world).id(), e)),
        );
        let team = TeamPlugin::entity(Faction::Team, world);
        for (slot, unit) in (0..).zip(run.team.get_team().units.into_iter()) {
            let id = unit.id;
            if let Some(entity) = team_units.get(&id) {
                let mut state = VarState::get_mut(*entity, world);
                state.set_int(VarName::Slot, slot.into());
                state.set_int(VarName::Hp, unit.hp);
                state.set_int(VarName::Pwr, unit.pwr);
                let new_xp = unit.xp as i32;
                let old_xp = state
                    .get_value_last(VarName::Xp)
                    .unwrap()
                    .get_int()
                    .unwrap();
                let new_lvl = unit.lvl as i32;
                let old_lvl = state
                    .get_value_last(VarName::Lvl)
                    .unwrap()
                    .get_int()
                    .unwrap();
                let level_changed = old_lvl != new_lvl;
                let xp_changed = old_xp != new_xp;

                if level_changed {
                    state.set_int(VarName::Lvl, new_lvl);
                }
                if xp_changed {
                    state.set_int(VarName::Xp, new_xp);
                }
                if level_changed {
                    TextColumnPlugin::add(
                        *entity,
                        format!("+{} Lvl", new_lvl - old_lvl).cstr_cs(PURPLE, CstrStyle::Bold),
                        world,
                    );
                } else if xp_changed {
                    TextColumnPlugin::add(
                        *entity,
                        format!("+{} Xp", new_xp - old_xp).cstr_cs(LIGHT_PURPLE, CstrStyle::Bold),
                        world,
                    );
                }
                team_units.remove(&id);
                continue;
            }
            let unit: PackedUnit = unit.into();
            unit.unpack(team, Some(slot), Some(id), world);
        }
        for entity in team_units.into_values() {
            UnitPlugin::despawn(entity, world);
        }
    }
    pub fn show_stats(run: &TArenaRun, ui: &mut Ui) {
        text_dots_text(
            "G".cstr(),
            run.g.to_string().cstr_cs(YELLOW, CstrStyle::Bold),
            ui,
        );
        text_dots_text(
            "lives".cstr(),
            format!("{}/{}", run.lives, run.max_lives)
                .to_string()
                .cstr_cs(GREEN, CstrStyle::Bold),
            ui,
        );
        text_dots_text(
            "floor".cstr(),
            run.floor.to_string().cstr_c(VISIBLE_BRIGHT),
            ui,
        );
        text_dots_text(
            "streak".cstr(),
            run.streak.to_string().cstr_c(VISIBLE_BRIGHT),
            ui,
        );
        text_dots_text("mode".cstr(), run.mode.cstr_expanded(), ui);
        br(ui);
        let total_weight = run.weights.iter().map(|w| w.at_least(0)).sum::<i32>() as f32;
        for rarity in Rarity::iter() {
            let name = rarity.cstr();
            let chance = format!(
                "{:.0}%",
                run.weights[rarity as i8 as usize] as f32 / total_weight * 100.0
            )
            .cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold);
            text_dots_text(name, chance, ui);
        }
    }
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Right, |ui, world| {
            ui.set_max_width(200.0);
            text_dots_text("name".cstr(), user_name().cstr_c(VISIBLE_BRIGHT), ui);
            if let Some(run) = cn().db.arena_run().get_current() {
                Self::show_stats(&run, ui);
            }
            ui.add_space(40.0);
            let run = cn().db.arena_run().current();
            ui.vertical_centered_justified(|ui| {
                if Button::click("Start Battle").ui(ui).clicked() {
                    cn().reducers.shop_finish(false).unwrap();
                }
                if run.floor == run.boss_floor {
                    "Final Battle".cstr_cs(YELLOW, CstrStyle::Bold).label(ui);
                    run.boss_team.get_team().hover_label(ui, world);
                } else {
                    "Next Floor".cstr_c(VISIBLE_DARK).label(ui);
                    if run.replenish_lives > 0 {
                        "Win for +1 life".cstr_cs(GREEN, CstrStyle::Bold).label(ui);
                    }
                    ui.add_space(50.0);
                    if let Some(floor_boss) = run.current_floor_boss {
                        if Button::click("Boss Battle").red(ui).ui(ui).clicked() {
                            cn().reducers.shop_finish(true).unwrap();
                        }
                        "Challenge Floor Boss"
                            .cstr_cs(RED, CstrStyle::Bold)
                            .label(ui);
                        floor_boss.get_team().hover_label(ui, world);
                    }
                }
            });
        })
        .transparent()
        .non_focusable()
        .pinned()
        .push(world);
        Tile::new(Side::Top, |ui, world| {
            Self::show_shop_container(ui, world);
        })
        .pinned()
        .transparent()
        .push(world);
        Tile::new(Side::Bottom, |ui, world| {
            Self::show_team_container(ui, world);
        })
        .pinned()
        .transparent()
        .push(world);
    }
    fn do_stack(target: u8, world: &mut World) {
        let mut sd = rm(world);
        let (source, faction) = sd.stack_source.unwrap();
        match faction {
            Faction::Team => {
                cn().reducers.stack_team(source as u8, target).unwrap();
            }
            Faction::Shop => {
                cn().reducers.stack_shop(source as u8, target).unwrap();
            }
            _ => panic!(),
        }
        sd.stack_source = None;
    }
    fn cancel_stack(world: &mut World) {
        rm(world).stack_source = None;
    }
    fn show_shop_container(ui: &mut Ui, world: &mut World) {
        let Some(run) = cn().db.arena_run().get_current() else {
            return;
        };

        let sd = world.resource::<ShopResource>();
        let family_slot = sd.family_slot;
        let stack_source = sd.stack_source;
        let g = run.g;
        let mut shop_container = TeamContainer::new(Faction::Shop)
            .slots(run.shop_slots.len())
            .name()
            .top_content(move |ui, _| {
                ui.vertical_centered_justified(|ui| {
                    ui.set_max_width(300.0);
                    let run = cn().db.arena_run().current();
                    let text = if run.free_rerolls > 0 {
                        "-0 G"
                            .cstr_c(YELLOW)
                            .push(format!(" ({})", run.free_rerolls).cstr_c(VISIBLE_LIGHT))
                            .take()
                    } else {
                        format!("-{} G", run.price_reroll).cstr_c(YELLOW)
                    };
                    if Button::click("reroll")
                        .cstr(
                            "Reroll "
                                .cstr_c(VISIBLE_LIGHT)
                                .push(text)
                                .style(CstrStyle::Heading)
                                .take(),
                        )
                        .enabled(g >= run.price_reroll || run.free_rerolls > 0)
                        .ui(ui)
                        .clicked()
                    {
                        cn().reducers.shop_reroll().unwrap();
                    }
                    ui.add_space(20.0);
                });
            })
            .slot_content(move |slot, _, ui, world| {
                let ss = &run.shop_slots[slot];
                if ss.available {
                    if let Some((stack_source, faction)) = stack_source {
                        if slot == stack_source && faction.eq(&Faction::Shop) {
                            if Button::click("Cancel").red(ui).ui(ui).clicked() {
                                Self::cancel_stack(world);
                            }
                        }
                    } else {
                        if Button::click(format!("-{} G", ss.buy_price))
                            .title("buy".cstr())
                            .enabled(g >= ss.buy_price)
                            .ui(ui)
                            .clicked()
                        {
                            cn().reducers.shop_buy(slot as u8).unwrap();
                        }
                        if ss.freeze {
                            if Button::click("Unfreeze").set_bg(true, ui).ui(ui).clicked() {
                                cn().reducers.shop_set_freeze(slot as u8, false).unwrap();
                            }
                        } else {
                            if Button::click("Freeze").gray(ui).ui(ui).clicked() {
                                cn().reducers.shop_set_freeze(slot as u8, true).unwrap();
                            }
                        }
                        if !ss.stack_targets.is_empty() {
                            let price = ss.stack_price;
                            if Button::click(format!("-{} G", price))
                                .title("stack".cstr())
                                .enabled(g >= price)
                                .ui(ui)
                                .clicked()
                            {
                                let mut sd = rm(world);
                                sd.stack_source = Some((slot, Faction::Shop));
                                sd.stack_targets =
                                    ss.stack_targets.iter().map(|v| *v as usize).collect_vec();
                                if ss.stack_targets.len() == 1 {
                                    Self::do_stack(ss.stack_targets[0], world);
                                }
                            }
                        }
                    }
                }
            })
            .hover_content(Self::container_on_hover);
        if let Some(slot) = family_slot {
            shop_container = shop_container.slot_name(slot, "Family Slot".into());
        }
        shop_container.ui(ui, world);
    }
    fn show_team_container(ui: &mut Ui, world: &mut World) {
        let Some(run) = cn().db.arena_run().get_current() else {
            return;
        };
        let team = run.team.get_team();
        let slots = game_assets().global_settings.arena.team_slots as usize;
        TeamContainer::new(Faction::Team)
            .slots(slots.max(team.units.len()))
            .max_slots(slots)
            .name()
            .slot_content(move |slot, e, ui, world| {
                let sd = rm(world);
                if e.is_some() && run.fusion.is_none() {
                    if let Some((stack_source, faction)) = sd.stack_source {
                        if slot == stack_source && faction.eq(&Faction::Team) {
                            if Button::click("Cancel").red(ui).ui(ui).clicked() {
                                Self::cancel_stack(world);
                            }
                        } else if sd.stack_targets.contains(&slot) {
                            if Button::click("Stack").ui(ui).clicked() {
                                Self::do_stack(slot as u8, world);
                            }
                        }
                    } else if let Some(fuse_source) = sd.fuse_source {
                        if slot == fuse_source {
                            if Button::click("Cancel").red(ui).ui(ui).clicked() {
                                let mut sd = rm(world);
                                sd.fuse_source = None;
                                sd.fuse_targets.clear();
                            }
                        } else if sd.fuse_targets.contains(&slot) {
                            if Button::click("Choose").ui(ui).clicked() {
                                cn().reducers
                                    .fuse_start(fuse_source as u8, slot as u8)
                                    .unwrap();
                            }
                        }
                    } else {
                        if let Some(ts) = run.team_slots.get(slot) {
                            if Button::click(format!("+{} G", ts.sell_price))
                                .title("Sell".cstr())
                                .ui(ui)
                                .clicked()
                            {
                                cn().reducers.shop_sell(slot as u8).unwrap();
                            }
                            if !ts.stack_targets.is_empty() {
                                if Button::click("Stack").ui(ui).clicked() {
                                    let mut sd = rm(world);
                                    sd.stack_source = Some((slot, Faction::Team));
                                    sd.stack_targets =
                                        ts.stack_targets.iter().map(|v| *v as usize).collect_vec();
                                }
                            }
                            if !ts.fuse_targets.is_empty() {
                                if Button::click("Fuse").ui(ui).clicked() {
                                    let mut sd = rm(world);
                                    sd.fuse_source = Some(slot);
                                    sd.fuse_targets =
                                        ts.fuse_targets.iter().map(|v| *v as usize).collect_vec();
                                }
                            }
                        }
                    }
                }
            })
            .hover_content(Self::container_on_hover)
            .on_swap(|a, b, _| {
                cn().reducers.shop_reorder(a as u8, b as u8).unwrap();
            })
            .ui(ui, world);
    }
    pub fn container_on_hover(_: usize, entity: Option<Entity>, ui: &mut Ui, world: &mut World) {
        let Some(entity) = entity else {
            return;
        };
        match UnitCard::new(&Context::new_play(entity), world) {
            Ok(c) => c.ui(ui),
            Err(e) => error!("Unit card error: {e}"),
        }
    }
    pub fn game_over_ui(ui: &mut Ui) {
        let Some(run) = cn().db.arena_run().get_current() else {
            return;
        };
        center_window("game_over", ui, |ui| {
            ui.set_width(300.0);
            ui.vertical_centered_justified(|ui| {
                if run.lives > 0 {
                    "Victory".cstr_cs(GREEN, CstrStyle::Heading)
                } else {
                    "Defeat".cstr_cs(RED, CstrStyle::Heading)
                }
                .label(ui);
                "Run Over".cstr_cs(YELLOW, CstrStyle::Bold).label(ui);
                run.mode.cstr_expanded().label(ui);
                text_dots_text(
                    format!("Final floor").cstr(),
                    run.floor.to_string().cstr_cs(YELLOW, CstrStyle::Bold),
                    ui,
                );
                br(ui);
                "Rewards".cstr_cs(YELLOW, CstrStyle::Heading2).label(ui);
                let mut total = 0;
                for Reward { source, amount } in run.rewards {
                    text_dots_text(
                        source.cstr_c(VISIBLE_DARK),
                        amount.to_string().cstr_c(YELLOW),
                        ui,
                    );
                    total += amount;
                }
                text_dots_text(
                    "Total".cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold),
                    total.to_string().cstr_cs(YELLOW, CstrStyle::Bold),
                    ui,
                );
                if Button::click("Finish").ui(ui).clicked() {
                    cn().reducers.run_finish().unwrap();
                }
            });
        });
    }
    pub fn maybe_queue_notification(text: Cstr, world: &mut World) {
        match cur_state(world) {
            GameState::Battle => rm(world).queued_notifications.push(text),
            _ => Notification::new(text).push(world),
        }
    }
}
