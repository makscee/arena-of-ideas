use bevy::input::common_conditions::input_just_pressed;
use bevy_egui::egui::Grid;

use super::*;

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shop), Self::on_enter)
            .add_systems(
                Update,
                Self::add_g.run_if(input_just_pressed(KeyCode::KeyG)),
            );
    }
}

impl MatchPlugin {
    pub fn check_battles(world: &mut World) -> NodeResult<()> {
        with_solid_source(|ctx| {
            let m = player(ctx)?.active_match_ref(ctx)?;
            if m.state.is_battle() {
                GameState::Battle.set_next(world);
            }
            Ok(())
        })
    }
    pub fn check_active(world: &mut World) -> NodeResult<bool> {
        with_solid_source(|ctx| {
            let m = player(ctx)?.active_match_ref(ctx)?;
            dbg!(m);
            if !m.active {
                GameState::MatchOver.set_next(world);
                Ok(false)
            } else {
                Ok(true)
            }
        })
    }
    fn on_enter(world: &mut World) {
        match Self::check_active(world) {
            Ok(active) => {
                if !active {
                    return;
                }
            }
            Err(e) => {
                e.cstr().notify_error(world);
                GameState::Title.set_next(world);
                return;
            }
        }
        if let Err(e) = Self::check_battles(world) {
            e.cstr().notify_error(world);
            return;
        }
    }
    fn add_g() {
        cn().reducers.admin_add_gold().notify_op();
    }
    pub fn pane_shop(ui: &mut Ui, _world: &World) -> NodeResult<()> {
        with_solid_source(|ctx| {
            let player = player(ctx)?;
            let m = player.active_match_ref(ctx)?;

            if m.state.is_battle() {
                ui.vertical_centered_justified(|ui| {
                    "Battle in Progress".cstr_s(CstrStyle::Heading2).label(ui);
                    "Complete the current battle to access the shop."
                        .cstr()
                        .label(ui);
                    if "Go to Battle".cstr().button(ui).clicked() {
                        GameState::Battle.set_next_op();
                    }
                });
                return Ok(());
            }

            let m = m.clone();

            let slots = &m.shop_offers.last().to_e_not_found()?.case;
            let available_rect = ui.available_rect_before_wrap();
            ui.horizontal_wrapped(|ui| {
                for i in 0..slots.len() {
                    let slot = slots[i].clone();
                    if !slot.sold {
                        ctx.with_owner(slot.node_id, |ctx| {
                            let resp = match slot.card_kind {
                                CardKind::Unit => {
                                    let unit = ctx.load::<NUnit>(slot.node_id)?;
                                    unit.as_card().compose(ctx, ui)
                                }
                                CardKind::House => {
                                    let house = ctx.load::<NHouse>(slot.node_id)?;
                                    house.as_card().compose(ctx, ui)
                                }
                            };
                            resp.dnd_set_drag_payload((i, slot.clone()));
                            Ok(())
                        })
                        .ui(ui);
                    }
                }
            });

            // Handle fusion unit selling with DndArea
            if let Some(payload) = DndArea::<(u64, usize, u64)>::new(available_rect)
                .text_fn(ui, |(_, _, unit_id)| {
                    if let Ok(unit) = ctx.load::<NUnit>(*unit_id) {
                        format!(
                            "sell {} [green +{}g]",
                            unit.unit_name,
                            global_settings().match_g.unit_sell
                        )
                    } else {
                        format!("[red unit get error]")
                    }
                })
                .ui(ui)
            {
                let (_fusion_id, _slot_idx, unit_id) = payload.as_ref();
                cn().reducers.match_sell_unit(*unit_id).notify_op();
            }

            Ok(())
        })
    }
    pub fn pane_info(ui: &mut Ui, _world: &World) -> NodeResult<()> {
        with_solid_source(|ctx| {
            let player = player(ctx)?;
            let m = player.active_match_ref(ctx)?;
            let g = m.g;
            let lives = m.lives;
            let floor = m.floor;

            let arena = ctx.load::<NArena>(ID_ARENA)?;
            let floors = arena.last_floor as i32;
            let is_last_floor = floor >= floors;

            ui.columns(2, |cui| {
                let ui = &mut cui[0];
                Grid::new("shop info").show(ui, |ui| {
                    "g".cstr().label(ui);
                    g.cstr_cs(YELLOW, CstrStyle::Bold).label(ui);
                    ui.end_row();
                    "lives".cstr().label(ui);
                    lives.cstr_cs(GREEN, CstrStyle::Bold).label(ui);
                    ui.end_row();
                    "floor".cstr().label(ui);
                    format!("{}/{}", floor, floors).cstr_s(CstrStyle::Bold).label(ui);
                    ui.end_row();
                });
                let ui = &mut cui[1];
                ui.horizontal_wrapped(|ui| {
                    if "Reroll".cstr().button(ui).clicked() {
                        cn().reducers.match_shop_reroll().notify_op();
                    }
                    ui.add_space(20.0);

                    // Regular battle button (disabled on last floor)
                    let regular_button = ui.add_enabled(!is_last_floor, egui::Button::new("Regular Battle"));
                    if regular_button.clicked() {
                        cn().reducers.match_start_battle().notify_op();
                    }

                    ui.add_space(10.0);

                    // Boss battle button with tooltip
                    let boss_button = "Boss Battle".cstr_s(CstrStyle::Bold).button(ui)
                        .on_hover_text("Fight the floor boss. Winning makes you the new boss and ends your run. Losing also ends your run regardless of lives left.");
                    if boss_button.clicked() {
                        cn().reducers.match_boss_battle().notify_op();
                    }
                });
            });
            Ok(())
        })
    }
    pub fn pane_roster(ui: &mut Ui, _world: &World) -> NodeResult<()> {
        with_solid_source(|ctx| {
            let player = player(ctx)?;
            let m = player.active_match_ref(ctx)?;
            let team = m.team_ref(ctx)?;
            let (_, card) =
                ui.dnd_drop_zone::<(usize, CardKind), NodeResult<()>>(Frame::new(), |ui| {
                    ui.expand_to_include_rect(ui.available_rect_before_wrap());
                    for house in team.houses_ref(ctx)? {
                        house.as_card().compose(ctx, ui);
                    }
                    Ok(())
                });
            if let Some(card) = card {
                cn().reducers.match_shop_buy(card.0 as u8).notify_op();
            }
            Ok(())
        })
    }
    pub fn pane_team(ui: &mut Ui, _world: &mut World) -> NodeResult<()> {
        with_solid_source(|ctx| {
            let player = player(ctx)?;
            let m = player.active_match_ref(ctx)?;
            let team = m.team_ref(ctx)?.clone().load_all(ctx)?.take();
            let rect = ui.available_rect_before_wrap();

            let team_editor = TeamEditor::new()
                .filled_slot_action(
                    "Sell Unit".to_string(),
                    Box::new(|_team, _fusion_id, unit_id, _slot_index, _ctx, _ui| {
                        cn().reducers.match_sell_unit(unit_id).notify_error_op();
                    }),
                )
                .with_action_handler(move |ctx, action| {
                    match action {
                        TeamAction::MoveUnit { unit_id, target } => match target {
                            UnitTarget::Slot {
                                fusion_id,
                                slot_index,
                            } => {
                                let slot_id = ctx
                                    .load::<NFusion>(*fusion_id)?
                                    .slots_ref(ctx)?
                                    .into_iter()
                                    .find(|s| s.index == *slot_index)
                                    .to_not_found()?
                                    .id;
                                cn().reducers
                                    .match_move_unit(*unit_id, slot_id)
                                    .notify_error_op();
                            }
                            UnitTarget::Bench => {
                                cn().reducers.match_bench_unit(*unit_id).notify_error_op();
                            }
                        },
                        TeamAction::AddSlot { fusion_id } => {
                            cn().reducers
                                .match_buy_fusion_slot(*fusion_id)
                                .notify_error_op();
                        }
                        TeamAction::BenchUnit { unit_id } => {
                            cn().reducers.match_bench_unit(*unit_id).notify_error_op();
                        }
                        TeamAction::ChangeActionRange {
                            slot_id,
                            start,
                            length,
                        } => {
                            cn().reducers
                                .match_change_action_range(*slot_id, *start as u8, *length as u8)
                                .notify_error_op();
                        }
                        TeamAction::ChangeTrigger { fusion_id, trigger } => {
                            cn().reducers
                                .match_change_trigger(*fusion_id, *trigger)
                                .notify_error_op();
                        }
                        TeamAction::StackUnit {
                            unit_id,
                            target_unit_id,
                        } => {
                            cn().reducers
                                .match_stack_unit(*unit_id, *target_unit_id)
                                .notify_error_op();
                        }
                        _ => {}
                    };
                    Ok(())
                });

            let _changed_team = team_editor.edit(&team, ctx, ui);

            if let Some(card) = DndArea::<(usize, ShopSlot)>::new(rect)
                .text_fn(ui, |slot| format!("buy [yellow -{}g]", slot.1.price))
                .ui(ui)
            {
                cn().reducers.match_shop_buy(card.0 as u8).notify_error_op();
            }

            Ok(())
        })
    }
    pub fn pane_match_over(ui: &mut Ui, _world: &mut World) -> NodeResult<()> {
        with_solid_source(|ctx| {
            let player = player(ctx)?;
            let m = player.active_match_ref(ctx)?;
            let won_last = m
                .battles_ref(ctx)?
                .iter()
                .sorted_by_key(|b| b.id)
                .last()
                .to_not_found()?
                .result
                == Some(true);
            let arena = ctx.load::<NArena>(ID_ARENA)?;
            let last_floor = arena.last_floor;

            ui.vertical_centered_justified(|ui| {
                "Match Over".cstr_s(CstrStyle::Heading).label(ui);

                if won_last && m.floor > last_floor {
                    format!(
                        "You're the [yellow [b ULTIMATE CHAMPION]]! Conquered [b {}] floors!",
                        m.floor
                    )
                    .cstr()
                    .label(ui);
                } else if won_last && m.floor == last_floor {
                    "You're the [yellow [b CHAMPION]]! Conquered the final floor!"
                        .cstr()
                        .label(ui);
                } else if won_last {
                    format!("Victory! You're the boss of floor [b {}]", m.floor)
                        .cstr()
                        .label(ui);
                } else if matches!(m.state, MatchState::BossBattle | MatchState::ChampionBattle) {
                    if m.floor == last_floor {
                        "Defeated by the champion on the final floor!"
                            .cstr()
                            .label(ui);
                    } else {
                        "Defeated by the boss! Your journey ends here."
                            .cstr()
                            .label(ui);
                    }
                } else {
                    "Out of lives! Game over.".cstr().label(ui);
                }

                // Show final stats
                ui.add_space(20.0);
                Grid::new("final_stats").show(ui, |ui| {
                    "Final Floor:".cstr().label(ui);
                    m.floor.cstr_s(CstrStyle::Bold).label(ui);
                    ui.end_row();
                    "Lives Remaining:".cstr().label(ui);
                    m.lives
                        .cstr_cs(if m.lives > 0 { GREEN } else { RED }, CstrStyle::Bold)
                        .label(ui);
                    ui.end_row();
                    if m.floor == last_floor {
                        "Status:".cstr().label(ui);
                        if won_last {
                            "Champion".cstr_cs(YELLOW, CstrStyle::Bold).label(ui);
                        } else {
                            "Challenged Champion"
                                .cstr_cs(RED, CstrStyle::Bold)
                                .label(ui);
                        }
                        ui.end_row();
                    }
                });
            });
            ui.vertical_centered(|ui| {
                if "Done".cstr().button(ui).clicked() {
                    op(|world| {
                        cn().reducers.match_complete().notify(world);
                        GameState::Title.set_next(world);
                    });
                }
            });
            Ok(())
        })
    }
    pub fn pane_leaderboard(ui: &mut Ui, _world: &mut World) -> NodeResult<()> {
        with_solid_source(|ctx| {
            let ctx_world = ctx.world_mut()?;

            let floors = ctx_world
                .query::<&NFloorBoss>()
                .iter(ctx_world)
                .sorted_by_key(|b| -b.floor)
                .cloned()
                .collect_vec();

            ui.vertical_centered_justified(|ui| {
                "Arena Bosses".cstr_s(CstrStyle::Heading2).label(ui);
            });

            if floors.is_empty() {
                ui.vertical_centered(|ui| {
                    "No bosses yet - be the first to conquer the arena!"
                        .cstr()
                        .label(ui);
                });
            } else {
                Table::from_data(&floors)
                    .column(
                        "Floor",
                        |_ctx, ui, _, value| {
                            let floor = value.get_i32()?;
                            // Show all high floors as champions
                            if floor >= 15 {
                                format!("[yellow [b {}] ⭐ CHAMPION]", floor)
                                    .cstr()
                                    .label(ui);
                            } else {
                                format!("[b {}] (Boss)", floor).cstr().label(ui);
                            }
                            Ok(())
                        },
                        |_, t| Ok(t.floor.into()),
                    )
                    .column(
                        "Boss",
                        |ctx, ui, _, value| {
                            let id = value.get_u64()?;
                            if let Ok(player) = ctx.load::<NPlayer>(id) {
                                player
                                    .player_name
                                    .cstr_cs(YELLOW, CstrStyle::Bold)
                                    .label(ui);
                            } else {
                                "Arena".cstr().label(ui);
                            }
                            Ok(())
                        },
                        |context, t| {
                            let team = t.team_ref(context)?;
                            Ok(team.owner.into())
                        },
                    )
                    .ui(ctx, ui);
            }
            Ok(())
        })
    }
}
