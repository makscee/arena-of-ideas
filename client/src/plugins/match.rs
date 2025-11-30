use bevy::input::common_conditions::input_just_pressed;
use bevy_egui::egui::{Grid, UiKind};

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
            let m = player(ctx)?.active_match.load_node(ctx)?;
            if m.state.is_battle() {
                GameState::Battle.set_next(world);
            }
            Ok(())
        })
    }
    pub fn check_active(world: &mut World) -> NodeResult<bool> {
        with_solid_source(|ctx| {
            let m = player(ctx)?.active_match.load_node(ctx)?;
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
            let m = player.active_match.load_node(ctx)?;

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

            if let Some((_, _, variants)) = &m.fusion {
                Self::show_fusion_window(ctx, ui, variants.clone())?;
                return Ok(());
            }

            let m = m.clone();

            let slots = &m.shop_offers.last().to_e_not_found()?.case;
            let available_rect = ui.available_rect_before_wrap();
            ui.vertical_centered_justified(|ui| {
                ui.set_width(200.0);
                if format!(
                    "Reroll [b [yellow {}g]]",
                    global_settings().match_settings.reroll
                )
                .cstr()
                .button(ui)
                .clicked()
                {
                    cn().reducers.match_shop_reroll().notify_op();
                }
                ui.separator();
            });
            ui.horizontal_wrapped(|ui| {
                for i in 0..slots.len() {
                    let slot = slots[i].clone();
                    let buy_txt = format!("Buy [b [yellow {}g]]", slot.price);
                    ctx.with_owner(slot.node_id, |ctx| {
                        let resp = if !slot.sold {
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
                            resp
                        } else {
                            let (_, resp) = ui.allocate_exact_size(CARD_SIZE, Sense::hover());
                            resp
                        };
                        let btn_rect = Rect::from_min_size(
                            resp.rect.left_bottom(),
                            egui::vec2(resp.rect.width(), LINE_HEIGHT),
                        );
                        ui.expand_to_include_rect(btn_rect);
                        {
                            let ui =
                                &mut ui.new_child(UiBuilder::new().max_rect(btn_rect).layout(
                                    Layout::centered_and_justified(egui::Direction::TopDown),
                                ));
                            if buy_txt
                                .clone()
                                .to_button()
                                .enabled(!slot.sold)
                                .ui(ui)
                                .clicked()
                            {
                                cn().reducers.match_shop_buy(i as u8).notify_error_op();
                            }
                        }
                        Ok(())
                    })
                    .ui(ui);
                }
            });

            // Handle fusion unit selling with DndArea
            if let Some(payload) = DndArea::<DraggedUnit>::new(available_rect)
                .text_fn(ui, |dragged_unit| {
                    if let Ok(unit) = ctx.load::<NUnit>(dragged_unit.unit_id) {
                        Some(format!(
                            "sell {} [b [yellow +{}g]]",
                            unit.unit_name,
                            global_settings().match_settings.unit_sell
                        ))
                    } else {
                        Some(format!("[red unit get error]"))
                    }
                })
                .ui(ui)
            {
                cn().reducers.match_sell_unit(payload.unit_id).notify_op();
            }

            Ok(())
        })
    }
    const HOVER_TEXT: &str = "Fight the floor boss. Winning makes you the new boss and ends your run. Losing also ends your run regardless of lives left.";
    pub fn pane_info(ui: &mut Ui, _world: &World) -> NodeResult<()> {
        with_solid_source(|ctx| {
            let player = player(ctx)?;
            let m = player.active_match.load_node(ctx)?;
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
                    format!("{}/{}", floor, floors)
                        .cstr_s(CstrStyle::Bold)
                        .label(ui);
                    ui.end_row();
                });
                let ui = &mut cui[1];
                ui.vertical(|ui| {
                    let boss_button = "[b [red Boss Battle]]"
                        .cstr()
                        .button(ui)
                        .on_hover_text(Self::HOVER_TEXT);
                    if boss_button.clicked() {
                        cn().reducers.match_boss_battle().notify_op();
                    }
                    ui.add_space(20.0);
                    if "Regular Battle"
                        .cstr()
                        .to_button()
                        .enabled(!is_last_floor)
                        .ui(ui)
                        .clicked()
                    {
                        cn().reducers.match_start_battle().notify_op();
                    }
                });
            });
            Ok(())
        })
    }
    pub fn pane_roster(ui: &mut Ui, _world: &World) -> NodeResult<()> {
        with_solid_source(|ctx| {
            let player = player(ctx)?;
            let m = player.active_match.load_node(ctx)?;
            let (_, card) =
                ui.dnd_drop_zone::<(usize, CardKind), NodeResult<()>>(Frame::new(), |ui| {
                    ui.expand_to_include_rect(ui.available_rect_before_wrap());
                    for house in m.shop_pool.get()?.houses.get()? {
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
            let mut m = player
                .active_match
                .load_node(ctx)?
                .clone()
                .load_all(ctx)?
                .take();
            let rect = ui.available_rect_before_wrap();

            // Display slots
            let slot_count = global_settings().team_slots;
            ui.horizontal(|ui| -> NodeResult<()> {
                for slot_index in 0..slot_count {
                    let slot = m
                        .slots
                        .get_mut()?
                        .iter_mut()
                        .find(|s| s.index == slot_index as i32)
                        .unwrap();

                    if let Ok(slot_unit) = slot.unit.get_mut() {
                        // Filled slot - show unit with MatRect
                        let response = MatRect::new(egui::vec2(100.0, 100.0))
                            .add_mat(&slot_unit.representation.get()?.material, slot_unit.id)
                            .unit_rep_with_default(slot_unit.id)
                            .ui(ui, ctx)
                            .on_hover_ui(|ui| {
                                (&*slot_unit).as_card().compose(ctx, ui);
                            });

                        // Set drag payload
                        response.dnd_set_drag_payload(DraggedUnit {
                            unit_id: slot_unit.id,
                            from_location: UnitLocation::Slot {
                                index: slot_index as i32,
                            },
                        });

                        // Context menu for filled slots
                        response.show_menu(ui, |ui| {
                            if ui.button("Sell Unit").clicked() {
                                cn().reducers
                                    .match_sell_unit(slot_unit.id)
                                    .notify_error_op();
                                ui.close_kind(UiKind::Menu);
                            }
                            if ui.button("Move to Bench").clicked() {
                                cn().reducers
                                    .match_bench_unit(slot_unit.id)
                                    .notify_error_op();
                                ui.close_kind(UiKind::Menu);
                            }
                        });

                        #[derive(AsRefStr)]
                        enum FuseStackSwap {
                            Fuse,
                            Stack,
                            Swap,
                        }
                        fn drag_stack_swap(
                            ctx: &ClientContext,
                            source: &mut NUnit,
                            target: &mut NUnit,
                        ) -> NodeResult<FuseStackSwap> {
                            if source.check_stackable(target, ctx)? {
                                Ok(FuseStackSwap::Stack)
                            } else if source.check_fusible_with(target, ctx)? {
                                Ok(FuseStackSwap::Fuse)
                            } else {
                                Ok(FuseStackSwap::Swap)
                            }
                        }
                        // Handle drop on this slot
                        let slot_rect = response.rect;
                        if let Some(dragged) = DndArea::<DraggedUnit>::new(slot_rect)
                            .text_fn(ui, |du| {
                                if du.unit_id == slot_unit.id {
                                    return None;
                                }
                                Some(
                                    drag_stack_swap(
                                        ctx,
                                        &mut ctx.load::<NUnit>(du.unit_id).unwrap(),
                                        slot_unit,
                                    )
                                    .unwrap()
                                    .as_ref()
                                    .to_string(),
                                )
                            })
                            .ui(ui)
                        {
                            if dragged.unit_id != slot_unit.id {
                                if let Ok(mut source) = ctx.load::<NUnit>(dragged.unit_id) {
                                    match drag_stack_swap(ctx, &mut source, slot_unit).unwrap() {
                                        FuseStackSwap::Fuse => cn()
                                            .reducers
                                            .match_start_fusion(dragged.unit_id, slot_unit.id)
                                            .notify_error_op(),
                                        FuseStackSwap::Stack => cn()
                                            .reducers
                                            .match_stack_unit(dragged.unit_id, slot_unit.id)
                                            .notify_error_op(),
                                        FuseStackSwap::Swap => cn()
                                            .reducers
                                            .match_move_unit(dragged.unit_id, slot.index)
                                            .notify_error_op(),
                                    }
                                }
                            }
                        }
                    } else {
                        let resp = MatRect::new(egui::vec2(100.0, 100.0)).ui(ui, ctx);
                        if let Some(dragged) = DndArea::<DraggedUnit>::new(resp.rect)
                            .text_fn(ui, |_| Some(format!("Move")))
                            .ui(ui)
                        {
                            cn().reducers
                                .match_move_unit(dragged.unit_id, slot_index as i32)
                                .notify_error_op();
                        }
                    }
                }
                Ok(())
            })
            .inner
            .ui(ui);
            ui.separator();
            if let Some(dragged) = DndArea::<DraggedUnit>::new(ui.available_rect_before_wrap())
                .text_fn(ui, |du| {
                    if matches!(du.from_location, UnitLocation::Slot { .. }) {
                        Some(format!("Bench"))
                    } else {
                        None
                    }
                })
                .ui(ui)
            {
                cn().reducers
                    .match_bench_unit(dragged.unit_id)
                    .notify_error_op();
            }
            ui.label("Bench:");
            ui.horizontal(|ui| -> NodeResult<()> {
                for unit in m.bench.get()? {
                    let response = MatRect::new(egui::vec2(100.0, 100.0))
                        .add_mat(&unit.representation.get()?.material, unit.id)
                        .unit_rep_with_default(unit.id)
                        .ui(ui, ctx);

                    if response.hovered() {
                        unit.as_card().compose(ctx, ui);
                    }

                    // Set drag payload
                    response.dnd_set_drag_payload(DraggedUnit {
                        unit_id: unit.id,
                        from_location: UnitLocation::Bench,
                    });

                    // Context menu for bench units
                    response.context_menu(|ui| {
                        if ui.button("Sell Unit").clicked() {
                            cn().reducers.match_sell_unit(unit.id).notify_error_op();
                            ui.close_kind(UiKind::Menu);
                        }
                    });
                }
                Ok(())
            })
            .inner
            .ui(ui);

            // Handle shop card drops
            if let Some(card) = DndArea::<(usize, ShopSlot)>::new(rect)
                .text_fn(ui, |slot| Some(format!("buy [yellow -{}g]", slot.1.price)))
                .ui(ui)
            {
                cn().reducers.match_shop_buy(card.0 as u8).notify_error_op();
            }

            Ok(())
        })
    }

    fn show_fusion_window(
        ctx: &mut ClientContext,
        ui: &mut Ui,
        variants: Vec<PackedNodes>,
    ) -> NodeResult<()> {
        ui.vertical_centered_justified(|ui| -> NodeResult<()> {
            "Unit Fusion".cstr_s(CstrStyle::Heading2).label(ui);
            "Select one of three fusion variants:".cstr().label(ui);
            ui.separator();

            ui.horizontal_wrapped(|ui| -> NodeResult<()> {
                for (idx, packed) in variants.iter().enumerate() {
                    let merged_unit = NUnit::unpack(packed)?;
                    let fusion_names = match idx {
                        0 => ("Front", "Combines source actions first"),
                        1 => ("Back", "Combines target actions first"),
                        2 => ("Split", "Keeps separate triggers"),
                        _ => ("Unknown", ""),
                    };

                    ui.vertical(|ui| {
                        format!("[b {}]", fusion_names.0).cstr().label(ui);
                        fusion_names.1.cstr_s(CstrStyle::Small).label(ui);

                        ctx.with_owner(merged_unit.id, |ctx| {
                            merged_unit.as_card().compose(ctx, ui);
                            Ok(())
                        })
                        .ok();

                        if format!("Select [b {}]", fusion_names.0)
                            .cstr()
                            .button(ui)
                            .clicked()
                        {
                            cn().reducers.match_choose_fusion(idx as i32).notify_op();
                        }
                    });
                }
                Ok(())
            })
            .inner?;

            ui.separator();
            if "Cancel Fusion".cstr().button(ui).clicked() {
                cn().reducers.match_cancel_fusion().notify_op();
            }
            Ok(())
        })
        .inner
    }

    pub fn pane_match_over(ui: &mut Ui, _world: &mut World) -> NodeResult<()> {
        with_solid_source(|ctx| {
            let player = player(ctx)?;
            let m = player.active_match.load_node(ctx)?.clone();
            let won_last = if let Some(last_battle_id) = m.battle_history.last() {
                cn().db
                    .battle()
                    .id()
                    .find(last_battle_id)
                    .map(|b| b.result == Some(true))
                    .unwrap_or(false)
            } else {
                false
            };
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
                            value.cstr().label(ui);
                            Ok(())
                        },
                        |_, t| Ok(t.floor.into()),
                    )
                    .column(
                        "Player",
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
                        |ctx, t| {
                            let team = t.team.load_node(ctx)?;
                            Ok(team.owner.into())
                        },
                    )
                    .column(
                        "Team",
                        |_, ui, _, team_str| {
                            team_str.get_string()?.label(ui);
                            Ok(())
                        },
                        |ctx, n| Ok(n.team.load_node(ctx)?.title(ctx).into()),
                    )
                    .ui(ctx, ui);
            }
            Ok(())
        })
    }

    pub fn pane_battle_history(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        with_solid_source(|ctx| {
            let battles = cn()
                .db
                .battle()
                .iter()
                .sorted_by_key(|b| std::cmp::Reverse(b.id))
                .collect_vec();

            ui.vertical_centered_justified(|ui| {
                "Battle History".cstr_s(CstrStyle::Heading2).label(ui);
            });

            if battles.is_empty() {
                ui.vertical_centered(|ui| {
                    "No battles recorded yet".cstr().label(ui);
                });
            } else {
                Table::from_data(&battles)
                    .column(
                        "Time",
                        |_ctx, ui, _, value| {
                            let ts = value.get_u64()?;
                            let date = chrono::DateTime::from_timestamp_micros(ts as i64)
                                .unwrap_or_default();
                            format!("[s [tw {}]]", date.format("%Y-%m-%d %H:%M"))
                                .cstr()
                                .label(ui);
                            Ok(())
                        },
                        |_, b| Ok(b.ts.into()),
                    )
                    .column(
                        "Left Team",
                        |_, ui, _, value| {
                            value.to_string().label(ui);
                            Ok(())
                        },
                        |_, b| Ok(b.left_team_name.clone().into()),
                    )
                    .column(
                        "Right Team",
                        |_, ui, _, value| {
                            value.to_string().label(ui);
                            Ok(())
                        },
                        |_, b| Ok(b.right_team_name.clone().into()),
                    )
                    .column(
                        "Result",
                        |_ctx, ui, b, _| {
                            match b.result {
                                Some(won) => {
                                    if won {
                                        "Left Won".cstr_cs(GREEN, CstrStyle::Bold).label(ui);
                                    } else {
                                        "Right Won".cstr_cs(RED, CstrStyle::Bold).label(ui);
                                    }
                                }
                                None => {
                                    "Pending".cstr_cs(GRAY, CstrStyle::Normal).label(ui);
                                }
                            }
                            Ok(())
                        },
                        |_, b| Ok(b.result.unwrap_or_default().into()),
                    )
                    .column(
                        "Actions",
                        |_, ui, b, _| {
                            if b.result.is_some() && ui.button("Replay").clicked() {
                                let left_team =
                                    NTeam::unpack(&PackedNodes::from_string(&b.left_team)?)?;
                                let right_team =
                                    NTeam::unpack(&PackedNodes::from_string(&b.right_team)?)?;

                                use crate::plugins::battle::BattlePlugin;
                                BattlePlugin::load_replay(left_team, right_team, world);
                                GameState::Battle.set_next(world);
                            }
                            Ok(())
                        },
                        |_, _| Ok(0.into()),
                    )
                    .ui(ctx, ui);
            }
            Ok(())
        })
    }
}
