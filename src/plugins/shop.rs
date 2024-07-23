use bevy::input::common_conditions::input_just_pressed;
use spacetimedb_sdk::table::{TableWithPrimaryKey, UpdateCallbackId};

use super::*;

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shop), Self::enter)
            .add_systems(OnExit(GameState::Shop), Self::exit)
            .init_resource::<ShopData>();
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
pub struct ShopData {
    pub case_height: f32,
    callback: Option<UpdateCallbackId<TArenaRun>>,
    stack_source: Option<(usize, Faction)>,
    stack_targets: Vec<usize>,
    fuse_source: Option<usize>,
    fuse_targets: Vec<usize>,
    family_slot: Option<usize>,
}

impl ShopPlugin {
    fn give_g() {
        shop_change_g(10);
    }
    fn test(world: &mut World) {
        TeamPlugin::change_ability_var_int("Siphon".into(), VarName::M1, 1, Faction::Team, world);
    }
    fn enter(mut sd: ResMut<ShopData>) {
        if let Some(run) = TArenaRun::get_current() {
            if !run.active {
                GameState::GameOver.set_next_op();
                return;
            }
            OperationsPlugin::add(|world| Self::sync_run(*run, world));
        } else {
            run_start();
            once_on_run_start(|_, _, status| match status {
                spacetimedb_sdk::reducer::Status::Committed => OperationsPlugin::add(|world| {
                    Self::sync_run(TArenaRun::current(), world);
                }),
                spacetimedb_sdk::reducer::Status::Failed(e) => {
                    Notification::new(format!("Arena run start error: {e}"))
                        .error()
                        .push_op()
                }
                _ => panic!(),
            });
        }
        let cb = TArenaRun::on_update(|_, run, _| {
            let run = run.clone();
            OperationsPlugin::add(|w| Self::sync_run(run, w))
        });
        sd.callback = Some(cb);
    }
    fn exit(world: &mut World) {
        world.game_clear();
        if let Some(cb) = world.resource_mut::<ShopData>().callback.take() {
            TArenaRun::remove_on_update(cb);
        }
    }
    fn sync_run(run: TArenaRun, world: &mut World) {
        debug!("Sync run");
        let mut shop_units: HashMap<GID, Entity> = HashMap::from_iter(
            UnitPlugin::collect_faction(Faction::Shop, world)
                .into_iter()
                .map(|e| (VarState::get(e, world).id(), e)),
        );
        let team = TeamPlugin::entity(Faction::Shop, world);
        if let Some(Fusion {
            options,
            source: _,
            target: _,
        }) = run.fusion
        {
            for (slot, unit) in (0..).zip(options.into_iter()) {
                let id = unit.id;
                if shop_units.contains_key(&id) {
                    shop_units.remove(&id);
                    continue;
                }
                let unit: PackedUnit = unit.into();
                unit.unpack(team, Some(slot), Some(id), world);
            }
        } else {
            let mut sd = world.resource_mut::<ShopData>();
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
                    world.resource_mut::<ShopData>().family_slot = Some(slot as usize);
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
        let mut team_units: HashMap<GID, Entity> = HashMap::from_iter(
            UnitPlugin::collect_faction(Faction::Team, world)
                .into_iter()
                .map(|e| (VarState::get(e, world).id(), e)),
        );
        let team = TeamPlugin::entity(Faction::Team, world);
        for (slot, unit) in (0..).zip(TTeam::filter_by_id(run.team).unwrap().units.into_iter()) {
            let id = unit.id;
            if let Some(entity) = team_units.get(&id) {
                let mut state = VarState::get_mut(*entity, world);
                state.set_int(VarName::Slot, slot.into());
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
    pub fn overlay_widgets(ctx: &egui::Context, _: &mut World) {
        Tile::left("Stats")
            .open()
            .transparent()
            .non_resizable()
            .show(ctx, |ui| {
                text_dots_text(&"name".cstr(), &user_name().cstr_c(VISIBLE_BRIGHT), ui);
                if let Some(run) = TArenaRun::get_current() {
                    text_dots_text(
                        &"G".cstr(),
                        &run.g.to_string().cstr_cs(YELLOW, CstrStyle::Bold),
                        ui,
                    );
                    text_dots_text(
                        &"lives".cstr(),
                        &run.lives.to_string().cstr_cs(GREEN, CstrStyle::Bold),
                        ui,
                    );
                    text_dots_text(
                        &"round".cstr(),
                        &run.round.to_string().cstr_c(VISIBLE_BRIGHT),
                        ui,
                    );
                    text_dots_text(
                        &"score".cstr(),
                        &run.score.to_string().cstr_c(VISIBLE_BRIGHT),
                        ui,
                    );
                }
            });
        Tile::right("To battle")
            .open()
            .transparent()
            .non_resizable()
            .show(ctx, |ui| {
                if Button::click("Start Battle".into()).ui(ui).clicked() {
                    shop_finish();
                    once_on_shop_finish(|_, _, status| {
                        status.on_success(|w| GameState::ShopBattle.proceed_to_target(w))
                    });
                }
            });
    }
    fn do_stack(target: u8, world: &mut World) {
        let mut sd = world.resource_mut::<ShopData>();
        let (source, faction) = sd.stack_source.unwrap();
        match faction {
            Faction::Team => {
                stack_team(source as u8, target);
                once_on_stack_team(|_, _, status, _, _| match status {
                    StdbStatus::Committed => {}
                    StdbStatus::Failed(e) => Notification::new(format!("Stack failed: {e}"))
                        .error()
                        .push_op(),
                    _ => panic!(),
                });
            }
            Faction::Shop => {
                stack_shop(source as u8, target);
                once_on_stack_shop(|_, _, status, _, _| match status {
                    StdbStatus::Committed => {}
                    StdbStatus::Failed(e) => Notification::new(format!("Stack failed: {e}"))
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
        world.resource_mut::<ShopData>().stack_source = None;
    }
    pub fn show_containers(wd: &mut WidgetData, ui: &mut Ui, world: &mut World) {
        let Some(run) = TArenaRun::get_current() else {
            return;
        };

        let sd = world.resource::<ShopData>().clone();

        let team = TTeam::filter_by_id(run.team).unwrap();
        let g = run.g;
        let mut shop_container = UnitContainer::new(Faction::Shop)
            .pivot(Align2::CENTER_TOP)
            .position(egui::vec2(0.5, 0.0))
            .slots(run.shop_slots.len())
            .top_content(move |ui, _| {
                let run = TArenaRun::current();
                if run.fusion.is_some() {
                    if Button::click("Cancel".into()).ui(ui).clicked() {
                        fuse_cancel();
                    }
                } else {
                    let text = if run.free_rerolls > 0 {
                        format!("-0 G ({})", run.free_rerolls)
                    } else {
                        format!("-{} G", run.price_reroll)
                    };
                    if Button::click(text)
                        .title("Reroll".into())
                        .enabled(g >= 1)
                        .ui(ui)
                        .clicked()
                    {
                        shop_reroll();
                    }
                }
            })
            .slot_content(move |slot, e, ui, world| {
                if run.fusion.is_some() {
                    if e.is_some() && Button::click("Choose".into()).ui(ui).clicked() {
                        fuse_choose(slot as u8);
                        once_on_fuse_choose(|_, _, status, _| match status {
                            StdbStatus::Committed => {}
                            StdbStatus::Failed(e) => e.notify_error(),
                            _ => panic!(),
                        });
                    }
                } else {
                    let ss = &run.shop_slots[slot];
                    if ss.available {
                        if let Some((stack_source, faction)) = sd.stack_source {
                            if slot == stack_source && faction.eq(&Faction::Shop) {
                                if Button::click("Cancel".into()).ui(ui).clicked() {
                                    Self::cancel_stack(world);
                                }
                            }
                        } else {
                            if Button::click(format!("-{} G", ss.buy_price))
                                .title("buy".into())
                                .enabled(g >= ss.buy_price)
                                .ui(ui)
                                .clicked()
                            {
                                shop_buy(slot as u8);
                            }
                            if !ss.stack_targets.is_empty() {
                                let price = ss.stack_price;
                                if Button::click(format!("-{} G", price))
                                    .title("stack".into())
                                    .enabled(g >= price)
                                    .ui(ui)
                                    .clicked()
                                {
                                    let mut sd = world.resource_mut::<ShopData>();
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
                }
            })
            .hover_content(Self::container_on_hover);
        if let Some(slot) = sd.family_slot {
            shop_container = shop_container.slot_name(slot, "Family Slot".into());
        }
        shop_container.ui(wd, ui, world);
        let slots = GameAssets::get(world).global_settings.arena.team_slots as usize;
        let run = TArenaRun::current();
        UnitContainer::new(Faction::Team)
            .pivot(Align2::CENTER_TOP)
            .position(egui::vec2(0.5, 1.0))
            .slots(slots.max(team.units.len()))
            .max_slots(slots)
            .slot_content(move |slot, e, ui, world| {
                if e.is_some() && run.fusion.is_none() {
                    if let Some((stack_source, faction)) = sd.stack_source {
                        if slot == stack_source && faction.eq(&Faction::Team) {
                            if Button::click("Cancel".into()).ui(ui).clicked() {
                                Self::cancel_stack(world);
                            }
                        } else if sd.stack_targets.contains(&slot) {
                            if Button::click("Stack".into()).ui(ui).clicked() {
                                Self::do_stack(slot as u8, world);
                            }
                        }
                    } else if let Some(fuse_source) = sd.fuse_source {
                        if slot == fuse_source {
                            if Button::click("Cancel".into()).ui(ui).clicked() {
                                let mut sd = world.resource_mut::<ShopData>();
                                sd.fuse_source = None;
                                sd.fuse_targets.clear();
                            }
                        } else if sd.fuse_targets.contains(&slot) {
                            if Button::click("Choose".into()).ui(ui).clicked() {
                                fuse_start(slot as u8, fuse_source as u8);
                                once_on_fuse_start(|_, _, status, _, _| match status {
                                    StdbStatus::Committed => {}
                                    StdbStatus::Failed(e) => e.notify_error(),
                                    _ => panic!(),
                                });
                            }
                        }
                    } else {
                        if let Some(ts) = run.team_slots.get(slot) {
                            if Button::click(format!("+{} G", ts.sell_price))
                                .title("Sell".into())
                                .ui(ui)
                                .clicked()
                            {
                                shop_sell(slot as u8);
                            }
                            if !ts.stack_targets.is_empty() {
                                if Button::click("Stack".into()).ui(ui).clicked() {
                                    let mut sd = world.resource_mut::<ShopData>();
                                    sd.stack_source = Some((slot, Faction::Team));
                                    sd.stack_targets =
                                        ts.stack_targets.iter().map(|v| *v as usize).collect_vec();
                                }
                            }
                            if !ts.fuse_targets.is_empty() {
                                if Button::click("Fuse".into()).ui(ui).clicked() {
                                    let mut sd = world.resource_mut::<ShopData>();
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
            .ui(wd, ui, world);
    }
    pub fn container_on_hover(_: usize, entity: Option<Entity>, ui: &mut Ui, world: &mut World) {
        let Some(entity) = entity else {
            return;
        };
        match unit_card(&Context::new_play(entity), ui, world) {
            Ok(_) => {}
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
                text_dots_text(
                    &format!("Final round").cstr(),
                    &run.round.to_string().cstr_cs(YELLOW, CstrStyle::Bold),
                    ui,
                );
                text_dots_text(
                    &format!("Score").cstr(),
                    &run.round.to_string().cstr_cs(YELLOW, CstrStyle::Bold),
                    ui,
                );
                if Button::click("Finish".into()).ui(ui).clicked() {
                    run_finish();
                    once_on_run_finish(|_, _, status| match status {
                        StdbStatus::Committed => OperationsPlugin::add(|w| {
                            WidgetsPlugin::reset_state(w);
                            GameState::Title.proceed_to_target(w);
                        }),
                        StdbStatus::Failed(e) => error!("Failed to finish run: {e}"),
                        _ => panic!(),
                    });
                }
            });
        });
    }
}
