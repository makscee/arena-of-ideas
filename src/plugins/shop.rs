use bevy::input::common_conditions::input_just_pressed;
use egui::ScrollArea;
use spacetimedb_sdk::table::{TableWithPrimaryKey, UpdateCallbackId};

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

#[derive(Resource, Clone, Default)]
pub struct ShopResource {
    pub case_height: f32,
    callback: Option<UpdateCallbackId<TArenaRun>>,
    stack_source: Option<(usize, Faction)>,
    stack_targets: Vec<usize>,
    fuse_source: Option<usize>,
    fuse_targets: Vec<usize>,
    family_slot: Option<usize>,
    fusion_choice: Vec<i8>,
    fusion_cards: Vec<UnitCard>,
}
fn rm(world: &mut World) -> Mut<ShopResource> {
    world.resource_mut::<ShopResource>()
}

impl ShopPlugin {
    fn give_g() {
        shop_change_g(10);
    }
    fn test(world: &mut World) {
        TeamPlugin::change_ability_var_int("Siphon".into(), VarName::M1, 1, Faction::Team, world);
    }
    fn enter(mut sd: ResMut<ShopResource>) {
        if let Some(run) = TArenaRun::get_current() {
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
        let cb = TArenaRun::on_update(|_, run, _| {
            let run = run.clone();
            OperationsPlugin::add(|w| Self::sync_run(run, w))
        });
        sd.callback = Some(cb);
    }
    fn exit(world: &mut World) {
        world.game_clear();
        if let Some(cb) = rm(world).callback.take() {
            TArenaRun::remove_on_update(cb);
        }
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
            let team = run.team.get_team();
            let left_card = UnitCard::from_fused(team.units[a as usize].clone(), world).unwrap();
            let right_card = UnitCard::from_fused(team.units[b as usize].clone(), world).unwrap();
            let mut r = rm(world);
            r.fusion_cards = [left_card.clone(), left_card, right_card].into();
            r.fusion_choice = [-1, 1, 0].into();
            fn update_result_card(world: &mut World) {
                let run = TArenaRun::current();
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

            Confirmation::new("Fusion".cstr_c(YELLOW), |world| {
                let fc = rm(world).fusion_choice.clone();
                fuse_choose(fc[0], fc[1], fc[2]);
            })
            .content(|ui, world| {
                if Button::click("Swap".into()).ui(ui).clicked() {
                    Confirmation::pop(&egui_context(world).unwrap());
                    fuse_swap();
                }
                ui.set_width(ui.ctx().screen_rect().width() * 0.9);
                ui.columns(3, |ui| {
                    let r = rm(world);
                    for i in 0..3 {
                        r.fusion_cards[i].ui(&mut ui[i]);
                    }
                });
                ui.columns(3, |ui| {
                    let mut r = rm(world);
                    let mut need_update = false;
                    for i in 0..3 {
                        ui[i as usize].vertical_centered_justified(|ui| {
                            if Button::click("Trigger".into())
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
                            if Button::click("Target".into())
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
                            if Button::click("Effect".into())
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
            .decline(|_| {
                fuse_cancel();
            })
            .push(&egui_context(world).unwrap());
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
                    GameAssets::get(world)
                        .heroes
                        .get(&unit)
                        .cloned()
                        .unwrap()
                        .unpack(team, Some(slot), Some(id), world);
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
        for (slot, unit) in (0..).zip(TTeam::find_by_id(run.team).unwrap().units.into_iter()) {
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
            run.lives.to_string().cstr_cs(GREEN, CstrStyle::Bold),
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
        text_dots_text("mode".cstr(), run.mode.cstr(), ui);
    }
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Left, |ui, _| {
            text_dots_text("name".cstr(), user_name().cstr_c(VISIBLE_BRIGHT), ui);
            if let Some(run) = TArenaRun::get_current() {
                Self::show_stats(&run, ui);
            }
        })
        .transparent()
        .pinned()
        .non_focusable()
        .push(world);
        Tile::new(Side::Right, |ui, world| {
            let run = TArenaRun::current();
            let btn_text = "Start Battle".to_string();
            if Button::click(btn_text).ui(ui).clicked() {
                shop_finish();
                once_on_shop_finish(|_, _, status| {
                    status.on_success(|w| GameState::ShopBattle.proceed_to_target(w))
                });
            }
            if let Some(champion) = run.champion {
                ui.vertical_centered_justified(|ui| {
                    ui.add_space(30.0);
                    "Champion Battle".cstr_cs(YELLOW, CstrStyle::Bold).label(ui);
                    champion.get_team().hover_label(ui, world);
                });
            }
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
                stack_team(source as u8, target);
                once_on_stack_team(|_, _, status, _, _| match status {
                    StdbStatus::Committed => {}
                    StdbStatus::Failed(e) => Notification::new_string(format!("Stack failed: {e}"))
                        .error()
                        .push_op(),
                    _ => panic!(),
                });
            }
            Faction::Shop => {
                stack_shop(source as u8, target);
                once_on_stack_shop(|_, _, status, _, _| match status {
                    StdbStatus::Committed => {}
                    StdbStatus::Failed(e) => Notification::new_string(format!("Stack failed: {e}"))
                        .error()
                        .push_op(),
                    _ => panic!(),
                });
            }
            _ => panic!(),
        }
        sd.stack_source = None;
    }
    fn cancel_stack(world: &mut World) {
        rm(world).stack_source = None;
    }
    fn show_shop_container(ui: &mut Ui, world: &mut World) {
        let Some(run) = TArenaRun::get_current() else {
            return;
        };

        let sd = world.resource::<ShopResource>().clone();
        let g = run.g;
        let mut shop_container = TeamContainer::new(Faction::Shop)
            .slots(run.shop_slots.len())
            .name()
            .right_to_left()
            .top_content(move |ui, _| {
                let run = TArenaRun::current();
                let text = if run.free_rerolls > 0 {
                    format!("-0 G ({})", run.free_rerolls)
                } else {
                    format!("-{} G", run.price_reroll)
                };
                if Button::click(text)
                    .title("Reroll".cstr())
                    .enabled(g >= run.price_reroll || run.free_rerolls > 0)
                    .ui(ui)
                    .clicked()
                {
                    shop_reroll();
                }
            })
            .slot_content(move |slot, _, ui, world| {
                let ss = &run.shop_slots[slot];
                if ss.available {
                    if let Some((stack_source, faction)) = sd.stack_source {
                        if slot == stack_source && faction.eq(&Faction::Shop) {
                            if Button::click("Cancel".into()).red(ui).ui(ui).clicked() {
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
                            shop_buy(slot as u8);
                        }
                        if ss.freeze {
                            if Button::click("Unfreeze".into())
                                .set_bg(true, ui)
                                .ui(ui)
                                .clicked()
                            {
                                shop_set_freeze(slot as u8, false);
                            }
                        } else {
                            if Button::click("Freeze".into()).gray(ui).ui(ui).clicked() {
                                shop_set_freeze(slot as u8, true);
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
        if let Some(slot) = sd.family_slot {
            shop_container = shop_container.slot_name(slot, "Family Slot".into());
        }
        shop_container.ui(ui, world);
    }
    fn show_team_container(ui: &mut Ui, world: &mut World) {
        let Some(run) = TArenaRun::get_current() else {
            return;
        };
        let team = run.team.get_team();
        let slots = GameAssets::get(world).global_settings.arena.team_slots as usize;
        let sd = world.resource::<ShopResource>().clone();
        TeamContainer::new(Faction::Team)
            .slots(slots.max(team.units.len()))
            .max_slots(slots)
            .name()
            .right_to_left()
            .slot_content(move |slot, e, ui, world| {
                if e.is_some() && run.fusion.is_none() {
                    if let Some((stack_source, faction)) = sd.stack_source {
                        if slot == stack_source && faction.eq(&Faction::Team) {
                            if Button::click("Cancel".into()).red(ui).ui(ui).clicked() {
                                Self::cancel_stack(world);
                            }
                        } else if sd.stack_targets.contains(&slot) {
                            if Button::click("Stack".into()).ui(ui).clicked() {
                                Self::do_stack(slot as u8, world);
                            }
                        }
                    } else if let Some(fuse_source) = sd.fuse_source {
                        if slot == fuse_source {
                            if Button::click("Cancel".into()).red(ui).ui(ui).clicked() {
                                let mut sd = rm(world);
                                sd.fuse_source = None;
                                sd.fuse_targets.clear();
                            }
                        } else if sd.fuse_targets.contains(&slot) {
                            if Button::click("Choose".into()).ui(ui).clicked() {
                                fuse_start(fuse_source as u8, slot as u8);
                                once_on_fuse_start(|_, _, status, _, _| match status {
                                    StdbStatus::Committed => {}
                                    StdbStatus::Failed(e) => e.notify_error_op(),
                                    _ => panic!(),
                                });
                            }
                        }
                    } else {
                        if let Some(ts) = run.team_slots.get(slot) {
                            if Button::click(format!("+{} G", ts.sell_price))
                                .title("Sell".cstr())
                                .ui(ui)
                                .clicked()
                            {
                                shop_sell(slot as u8);
                            }
                            if !ts.stack_targets.is_empty() {
                                if Button::click("Stack".into()).ui(ui).clicked() {
                                    let mut sd = rm(world);
                                    sd.stack_source = Some((slot, Faction::Team));
                                    sd.stack_targets =
                                        ts.stack_targets.iter().map(|v| *v as usize).collect_vec();
                                }
                            }
                            if !ts.fuse_targets.is_empty() {
                                if Button::click("Fuse".into()).ui(ui).clicked() {
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
                shop_reorder(a as u8, b as u8);
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
        let Some(run) = TArenaRun::get_current() else {
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
                run.mode.cstr().label(ui);
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
                if Button::click("Finish".into()).ui(ui).clicked() {
                    run_finish();
                    once_on_run_finish(|_, _, status| match status {
                        StdbStatus::Committed => OperationsPlugin::add(|w| {
                            GameState::GameStart.proceed_to_target(w);
                        }),
                        StdbStatus::Failed(e) => error!("Failed to finish run: {e}"),
                        _ => panic!(),
                    });
                }
            });
        });
    }
}
