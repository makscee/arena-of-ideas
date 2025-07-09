use bevy::{ecs::event::EventReader, input::common_conditions::input_just_pressed};
use bevy_egui::egui::Grid;
use spacetimedb_sdk::DbContext;

use super::*;

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shop), Self::on_enter)
            .add_systems(
                Update,
                Self::on_match_update.run_if(in_state(GameState::Shop)),
            )
            .add_systems(
                Update,
                Self::add_g.run_if(input_just_pressed(KeyCode::KeyG)),
            );
    }
}

impl MatchPlugin {
    pub fn check_battles(world: &mut World) -> Result<(), ExpressionError> {
        Context::from_world_r(world, |context| {
            let m = player(context)?.active_match_load(context)?;
            if let Some(last) = m.battles_load(context).last() {
                if last.result.is_none() {
                    MatchPlugin::load_battle(context).notify(context.world_mut()?);
                }
            }
            Ok(())
        })
    }
    pub fn check_active(world: &mut World) -> Result<(), ExpressionError> {
        Context::from_world_r(world, |context| {
            let m = player(context)?.active_match_load(context)?;
            if !m.active {
                GameState::MatchOver.set_next(context.world_mut()?);
            }
            Ok(())
        })
    }
    fn on_enter(world: &mut World) {
        if let Err(e) = Self::check_active(world) {
            e.cstr().notify_error(world);
            GameState::Title.set_next(world);
            return;
        }
        Self::check_battles(world).log();
    }
    fn add_g() {
        cn().reducers.admin_add_gold().notify_op();
    }
    fn on_match_update(mut events: EventReader<StdbEvent>) {
        for event in events.read() {
            if event.node.kind == "NMatch" && event.node.owner == player_id() {
                op(|world| {
                    Context::from_world_r(world, |context| {
                        let world = context.world_mut()?;
                        Self::check_active(world).notify(world);
                        Self::check_battles(world).notify(world);
                        Ok(())
                    })
                    .log();
                });
            }
        }
    }
    pub fn pane_shop(ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        Context::from_world_ref_r(world, |context| {
            let m = player(context)?.active_match_load(context)?;
            let slots = &m.shop_offers.last().to_e_not_found()?.case;
            let available_rect = ui.available_rect_before_wrap();
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    format!("g: [yellow [b {}]]", m.g).label(ui);
                    if "reroll".cstr().button(ui).clicked() {
                        cn().reducers.match_reroll().notify_op();
                    }
                    ui.add_space(20.0);
                    if "Start Battle".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                        cn().reducers.match_start_battle().notify_op();
                    }
                    ui.expand_to_include_y(available_rect.max.y);
                });
                if ui.available_width() < 30.0 {
                    return;
                }
                ui.columns(slots.len(), |ui| {
                    for i in 0..slots.len() {
                        let ui = &mut ui[i];
                        let slot = slots[i].clone();
                        ui.scope_builder(
                            UiBuilder::new()
                                .layout(Layout::bottom_up(Align::Center).with_cross_justify(true))
                                .layer_id(egui::LayerId::new(Order::Middle, Id::new("card"))),
                            |ui| {
                                if "buy"
                                    .cstr()
                                    .as_button()
                                    .enabled(!slot.sold)
                                    .ui(ui)
                                    .clicked()
                                {
                                    cn().reducers.match_buy(i as u8).notify_op();
                                }
                                if !slot.sold {
                                    context.with_layer_ref(
                                        ContextLayer::Owner(context.entity(slot.node_id).unwrap()),
                                        |context| {
                                            ui.push_id(i, |ui| -> Result<(), ExpressionError> {
                                                let resp = match slot.card_kind {
                                                    CardKind::Unit => {
                                                        let unit = context
                                                            .get_by_id::<NUnit>(slot.node_id)?;
                                                        unit.view_card(context, ui)?
                                                    }
                                                    CardKind::House => {
                                                        let house = context
                                                            .get_by_id::<NHouse>(slot.node_id)?;
                                                        house.view_card(context, ui)?
                                                    }
                                                };
                                                resp.dnd_set_drag_payload((i, slot.clone()));
                                                Ok(())
                                            })
                                            .inner
                                            .ui(ui);
                                        },
                                    );
                                }
                            },
                        );
                    }
                });
            });
            Ok(())
        })
    }
    pub fn pane_info(ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        Context::from_world_ref_r(world, |context| {
            let m = player(context)?.active_match_load(context)?;
            Grid::new("shop info").show(ui, |ui| {
                "g".cstr().label(ui);
                m.g.cstr_cs(YELLOW, CstrStyle::Bold).label(ui);
                ui.end_row();
                "lives".cstr().label(ui);
                m.lives.cstr_cs(GREEN, CstrStyle::Bold).label(ui);
                ui.end_row();
                "floor".cstr().label(ui);
                m.floor.cstr_s(CstrStyle::Bold).label(ui);
                ui.end_row();
                "round".cstr().label(ui);
                m.round.cstr_s(CstrStyle::Bold).label(ui);
                ui.end_row();
            });
            Ok(())
        })
    }
    pub fn pane_roster(ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        Context::from_world_ref_r(world, |context| {
            let m = player(context)?.active_match_load(context)?;
            let team = m.team_load(context)?;
            let (_, card) = ui.dnd_drop_zone::<(usize, CardKind), Result<(), ExpressionError>>(
                Frame::new(),
                |ui| {
                    ui.expand_to_include_rect(ui.available_rect_before_wrap());
                    for house in team.houses_load(context) {
                        house
                            .tag_card(TagCardContext::new().expanded(true), context, ui)
                            .ui(ui);
                    }
                    Ok(())
                },
            );
            if let Some(card) = card {
                cn().reducers.match_play_house(card.0 as u8).unwrap();
            }
            Ok(())
        })
    }
    pub fn pane_hand(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        Context::from_world_r(world, |context| {
            let rect = ui.available_rect_before_wrap();
            let m = player(context)?.active_match_load(context)?;
            if ui.available_width() < 30.0 {
                return Ok(());
            }
            ui.columns(7, |ui| {
                for (i, (card_kind, id)) in m.hand.iter().enumerate() {
                    let ui = &mut ui[i];
                    let entity = context.entity(*id).unwrap();
                    context
                        .with_owner_ref(entity, |context| {
                            match card_kind {
                                CardKind::Unit => {
                                    let unit = context.get::<NUnit>(entity)?;
                                    unit.view_card(context, ui)?
                                        .dnd_set_drag_payload((i, unit.clone()));
                                }
                                CardKind::House => {
                                    let house = context.get::<NHouse>(entity)?;
                                    house
                                        .view_card(context, ui)?
                                        .dnd_set_drag_payload((i, house.clone()));
                                }
                            };
                            Ok(())
                        })
                        .ui(ui);
                }
            });
            if let Some(offer) = DndArea::<(usize, ShopSlot)>::new(rect)
                .text_fn(ui, |(_, slot)| {
                    format!(
                        "buy {} [yellow -{}g]",
                        match slot.card_kind {
                            CardKind::Unit => format!("unit"),
                            CardKind::House => format!("house"),
                        },
                        slot.price
                    )
                })
                .ui(ui)
            {
                cn().reducers.match_buy(offer.0 as u8).unwrap();
            }
            Ok(())
        })
    }
    pub fn pane_team(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        Context::from_world_r(world, |context| {
            let rect = ui.available_rect_before_wrap();
            let m = player(context)?.active_match_load(context)?;
            let team = m.team_load(context)?;
            NFusion::slots_editor(
                team.entity(),
                context,
                ui,
                |ui, resp, fusion| {
                    if let Some(unit) = DndArea::<(usize, NUnit)>::new(resp.rect)
                        .id(fusion.slot)
                        .text_fn(ui, |unit| {
                            let lvl_increase = match fusion.units(context) {
                                Ok(units) => {
                                    if let Some(unit) =
                                        units.iter().find(|u| u.unit_name == unit.1.unit_name)
                                    {
                                        let Ok(state) = unit.state_load(context) else {
                                            return "\nstate error".to_owned();
                                        };
                                        if state.xp + 1 >= state.lvl {
                                            format!(
                                                "\n[n [tl increase lvl:]\n{} -> {}]",
                                                state.lvl,
                                                state.lvl + 1
                                            )
                                        } else {
                                            format!(
                                                "\n[n [tl increase xp:]\n{} -> {} ({})]",
                                                state.xp,
                                                state.xp + 1,
                                                state.lvl
                                            )
                                        }
                                    } else {
                                        default()
                                    }
                                }
                                Err(e) => e.cstr(),
                            };
                            let cost = if lvl_increase.is_empty()
                                && fusion.units.ids.len() as i32 >= fusion.lvl
                            {
                                format!(
                                    "\n[yellow [b -{}g]]",
                                    (fusion.lvl + 1) * global_settings().match_g.fusion_lvl_mul
                                )
                            } else {
                                default()
                            };
                            format!("play [b {}]{cost}{lvl_increase}", unit.1.unit_name)
                        })
                        .ui(ui)
                    {
                        cn().reducers
                            .match_play_unit(unit.0 as u8, fusion.slot as u8)
                            .notify_error_op();
                    }
                },
                |fusion| {
                    cn().reducers
                        .match_edit_fusion(fusion.to_tnode())
                        .notify_op();
                },
                |fusion, id| cn().reducers.match_add_fusion_unit(fusion.id, id).unwrap(),
                |fusion, id| {
                    cn().reducers
                        .match_remove_fusion_unit(fusion.id, id)
                        .unwrap()
                },
                |fusions| {
                    cn().reducers.match_reorder_fusions(fusions).unwrap();
                },
            )
            .ui(ui);
            if let Some(house) = DndArea::<(usize, NHouse)>::new(rect)
                .text_fn(ui, |house| format!("play [b {}]", house.1.house_name))
                .ui(ui)
            {
                cn().reducers
                    .match_play_house(house.0 as u8)
                    .notify_error_op();
            }

            Ok(())
        })
    }
    pub fn pane_match_over(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        Context::from_world_r(world, |context| {
            let m = player(context)?.active_match_load(context)?;
            ui.vertical_centered_justified(|ui| {
                "Match Over".cstr_s(CstrStyle::Heading).label(ui);
                if m.lives > 0 {
                    format!("You're the [yellow [b champion]] of [b {}] floor", m.floor)
                        .cstr()
                        .label(ui);
                } else {
                    format!("Reached [b {}] floor", m.floor).cstr().label(ui);
                }
            });
            let world = context.world_mut()?;
            ui.vertical_centered(|ui| {
                if "Done".cstr().button(ui).clicked() {
                    cn().reducers.match_complete().notify(world);
                    GameState::Title.set_next(world);
                }
            });
            Ok(())
        })
    }
    pub fn pane_leaderboard(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        Context::from_world_r(world, |context| {
            let world = context.world_mut()?;

            let floors = world
                .query::<&NFloorBoss>()
                .iter(world)
                .sorted_by_key(|b| -b.floor)
                .cloned()
                .collect_vec();
            Table::from_data(&floors)
                .column(
                    "id",
                    |_, ui, _, value| {
                        value.get_u64()?.cstr().label(ui);
                        Ok(())
                    },
                    |_, t| Ok(t.id.into()),
                )
                .column(
                    "floor",
                    |_, ui, _, value| {
                        value.get_i32()?.cstr().label(ui);
                        Ok(())
                    },
                    |_, t| Ok(t.floor.into()),
                )
                .column(
                    "player",
                    |context, ui, _, value| {
                        let id = value.get_u64()?;
                        context
                            .get_by_id::<NPlayer>(id)?
                            .player_name
                            .cstr()
                            .label(ui);
                        Ok(())
                    },
                    |context, t| {
                        let team = t.team_load(context)?;
                        Ok(team.owner.into())
                    },
                )
                .ui(context, ui);
            Ok(())
        })
    }
    pub fn load_battle(context: &mut Context) -> Result<(), ExpressionError> {
        GameState::Battle.set_next(context.world_mut()?);
        let battles = player(context)?
            .active_match_load(context)?
            .battles_load(context);
        if battles.is_empty() {
            return Err("No battles in current match".into());
        }
        let battle = *battles.last().unwrap();
        let left = NTeam::load_recursive(context.world()?, battle.team_left)
            .to_custom_e_fn(|| format!("Failed to load Team#{}", battle.team_left))?;
        let right = NTeam::load_recursive(context.world()?, battle.team_right)
            .to_custom_e_fn(|| format!("Failed to load Team#{}", battle.team_right))?;
        let bid = battle.id;
        let world = context.world_mut()?;
        BattlePlugin::load_teams(bid, left, right, world);
        BattlePlugin::on_done_callback(
            |id, result, hash| {
                cn().reducers()
                    .match_submit_battle_result(id, result, hash)
                    .notify_error_op();
            },
            world,
        );
        Ok(())
    }

    pub fn pane_fusion(ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        Context::from_world_ref_r(world, |context| {
            let rect = ui.available_rect_before_wrap();
            let m = player(context)?.active_match_load(context)?;
            let team = m.team_load(context)?;
            let fusions = team.fusions_load(context);

            if fusions.is_empty() {
                ui.label("No fusion units available");
                return Ok(());
            }
            ui.columns(fusions.len(), |columns| {
                for (i, fusion) in fusions.iter().enumerate() {
                    let ui = &mut columns[i];

                    ui.vertical(|ui| {
                        ui.label(format!(
                            "{}/{}",
                            fusion.get_action_count(),
                            fusion.action_limit
                        ));
                        if let Ok(trigger) = NFusion::get_trigger(context, &fusion.trigger) {
                            let vctx = ViewContext::new(ui).non_interactible(true);
                            ui.horizontal(|ui| {
                                Icon::Lightning.show(ui);
                                trigger.view_title(vctx, context, ui);
                            });
                            let units = fusion.units(context).unwrap_or_default();
                            for (unit_index, action_ref) in fusion.behavior.iter().enumerate() {
                                if let Some(unit) = units.get(unit_index) {
                                    for i in 0..action_ref.length as usize {
                                        if let Ok(action) =
                                            NFusion::get_action(context, unit.id, action_ref, i)
                                        {
                                            if let Ok(entity) = context.entity(unit.id) {
                                                context
                                                    .with_owner_ref(entity, |context| {
                                                        action.view(vctx, context, ui);
                                                        Ok(())
                                                    })
                                                    .ui(ui);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            });

            ui.add_space(5.0);

            // Fusion rendering section
            ui.columns(fusions.len(), |columns| {
                for (fusion_idx, fusion) in fusions.iter().enumerate() {
                    let ui = &mut columns[fusion_idx];

                    // Render fusion using MatRect
                    let mut mat_rect = MatRect::new(egui::Vec2::new(80.0, 80.0));

                    // Add fusion-specific representations
                    if let Ok(units) = context.collect_parents_components::<NUnit>(fusion.id) {
                        for unit in units {
                            if let Ok(rep) = unit
                                .description_load(context)
                                .and_then(|d| d.representation_load(context))
                            {
                                mat_rect = mat_rect.add_mat(&rep.material, unit.id);
                            }
                        }
                    }

                    mat_rect.unit_rep_with_default(fusion.id).ui(ui, context);

                    ui.label(format!("Level {}", fusion.lvl));
                    ui.label(format!("Slot {}", fusion.slot));
                }
            });

            ui.add_space(5.0);

            ui.columns(fusions.len(), |columns| {
                for (fusion_idx, fusion) in fusions.iter().enumerate() {
                    let ui = &mut columns[fusion_idx];
                    let result = ui
                        .vertical(|ui| -> Result<(), ExpressionError> {
                            // Fusion slots
                            let units = fusion.units(context).unwrap_or_default();
                            let max_slots = fusion.lvl as usize;

                            for slot_idx in 0..max_slots {
                                if let Some(unit) = units.get(slot_idx) {
                                    let resp = ui
                                        .horizontal(|ui| {
                                            let resp = if let Ok(rep) = context
                                                .first_parent_recursive::<NUnitRepresentation>(
                                                    unit.id,
                                                ) {
                                                MatRect::new(egui::Vec2::new(60.0, 60.0))
                                                    .add_mat(&rep.material, unit.id)
                                                    .unit_rep_with_default(unit.id)
                                                    .ui(ui, context)
                                            } else {
                                                MatRect::new(egui::Vec2::new(60.0, 60.0))
                                                    .ui(ui, context)
                                            };

                                            // Get current action range for this unit by index
                                            let current_start = fusion
                                                .behavior
                                                .get(slot_idx)
                                                .map(|ar| ar.start)
                                                .unwrap_or(0);
                                            let current_len = fusion
                                                .behavior
                                                .get(slot_idx)
                                                .map(|ar| ar.length)
                                                .unwrap_or(0);

                                            // Get unit's available actions count
                                            let max_actions = if let Ok(behavior) = context
                                                .first_parent_recursive::<NUnitBehavior>(
                                                unit.id,
                                            ) {
                                                behavior
                                                    .reactions
                                                    .iter()
                                                    .map(|r| r.actions.len() as u8)
                                                    .max()
                                                    .unwrap_or(0)
                                            } else {
                                                0
                                            };

                                            // Action range controls
                                            ui.vertical(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label("Start:");

                                                    // Start decrease button
                                                    let can_decrease_start = current_start > 0;
                                                    let start_decrease_btn = ui.add_enabled(
                                                        can_decrease_start,
                                                        egui::Button::new("-"),
                                                    );
                                                    if start_decrease_btn.clicked()
                                                        && can_decrease_start
                                                    {
                                                        let new_start = current_start - 1;
                                                        cn().reducers
                                                            .match_set_fusion_unit_action_range(
                                                                unit.id,
                                                                new_start,
                                                                current_len,
                                                            )
                                                            .notify_error_op();
                                                    }

                                                    ui.label(format!("{}", current_start));

                                                    // Start increase button
                                                    let can_increase_start =
                                                        current_start + current_len < max_actions;
                                                    let start_increase_btn = ui.add_enabled(
                                                        can_increase_start,
                                                        egui::Button::new("+"),
                                                    );
                                                    if start_increase_btn.clicked()
                                                        && can_increase_start
                                                    {
                                                        let new_start = current_start + 1;
                                                        // Adjust length if it goes out of bounds
                                                        let max_len =
                                                            max_actions.saturating_sub(new_start);
                                                        let new_len = current_len.min(max_len);
                                                        cn().reducers
                                                            .match_set_fusion_unit_action_range(
                                                                unit.id, new_start, new_len,
                                                            )
                                                            .notify_error_op();
                                                    }
                                                });
                                                ui.horizontal(|ui| {
                                                    ui.label("Len:");

                                                    // Length decrease button
                                                    let can_decrease_len = current_len > 0;
                                                    let len_decrease_btn = ui.add_enabled(
                                                        can_decrease_len,
                                                        egui::Button::new("-"),
                                                    );
                                                    if len_decrease_btn.clicked()
                                                        && can_decrease_len
                                                    {
                                                        cn().reducers
                                                            .match_set_fusion_unit_action_range(
                                                                unit.id,
                                                                current_start,
                                                                current_len - 1,
                                                            )
                                                            .notify_error_op();
                                                    }

                                                    ui.label(format!("{}", current_len));

                                                    // Length increase button
                                                    let can_increase_len =
                                                        current_start + current_len < max_actions;
                                                    let len_increase_btn = ui.add_enabled(
                                                        can_increase_len,
                                                        egui::Button::new("+"),
                                                    );
                                                    if len_increase_btn.clicked()
                                                        && can_increase_len
                                                    {
                                                        cn().reducers
                                                            .match_set_fusion_unit_action_range(
                                                                unit.id,
                                                                current_start,
                                                                current_len + 1,
                                                            )
                                                            .notify_error_op();
                                                    }
                                                });
                                            });
                                            resp
                                        })
                                        .inner;
                                    if resp.dragged() {
                                        resp.dnd_set_drag_payload((fusion.id, slot_idx, unit.id));
                                        if let Some(pos) = ui.ctx().pointer_latest_pos() {
                                            let origin = resp.rect.center();
                                            let painter =
                                                ui.ctx().layer_painter(egui::LayerId::new(
                                                    egui::Order::Foreground,
                                                    egui::Id::new("drag_arrow"),
                                                ));
                                            painter.arrow(
                                                origin,
                                                pos - origin,
                                                ui.visuals().widgets.hovered.fg_stroke,
                                            );
                                        }
                                    }
                                    if let Some(payload) =
                                        DndArea::<(u64, usize, u64)>::new(resp.rect)
                                            .id(format!("unit_slot_{}_{}", fusion_idx, slot_idx))
                                            .text_fn(ui, |(_, _, unit_id)| {
                                                if let Ok(unit) =
                                                    context.get_by_id::<NUnit>(*unit_id)
                                                {
                                                    format!("Swap with {}", unit.unit_name)
                                                } else {
                                                    "Swap units".to_string()
                                                }
                                            })
                                            .ui(ui)
                                    {
                                        let (source_fusion_id, source_slot_idx, _source_unit_id) =
                                            payload.as_ref();
                                        if *source_fusion_id == fusion.id
                                            && *source_slot_idx != slot_idx
                                        {
                                            // Reorder within same fusion by removing and re-adding units
                                            // This is a workaround until proper reordering is implemented
                                            let current_units =
                                                fusion.units(context).unwrap_or_default();

                                            if *source_slot_idx < current_units.len()
                                                && slot_idx < current_units.len()
                                            {
                                                // Reorder within same fusion by swapping units
                                                let mut unit_ids: Vec<u64> =
                                                    current_units.iter().map(|u| u.id).collect();

                                                // Swap the units in the vector
                                                unit_ids.swap(*source_slot_idx, slot_idx);

                                                // Call the reorder reducer
                                                cn().reducers
                                                    .match_reorder_fusion_units(fusion.id, unit_ids)
                                                    .notify_error_op();
                                            }
                                        }
                                    }
                                } else {
                                    let resp =
                                        MatRect::new(egui::Vec2::new(60.0, 60.0)).ui(ui, context);

                                    // Show drop area feedback for empty slot (reordering existing units)
                                    if let Some(payload) =
                                        DndArea::<(u64, usize, u64)>::new(resp.rect)
                                            .id(format!("empty_slot_{}_{}", fusion_idx, slot_idx))
                                            .text_fn(ui, |(_, _, unit_id)| {
                                                if let Ok(unit) =
                                                    context.get_by_id::<NUnit>(*unit_id)
                                                {
                                                    format!("Move {} here", unit.unit_name)
                                                } else {
                                                    "Move unit here".to_string()
                                                }
                                            })
                                            .ui(ui)
                                    {
                                        let (source_fusion_id, source_slot_idx, _source_unit_id) =
                                            payload.as_ref();
                                        if *source_fusion_id == fusion.id {
                                            // Move unit to empty slot within same fusion
                                            let current_units =
                                                fusion.units(context).unwrap_or_default();

                                            if *source_slot_idx < current_units.len()
                                                && slot_idx < fusion.lvl as usize
                                            {
                                                // Move unit to empty slot within same fusion
                                                let mut unit_ids: Vec<u64> =
                                                    current_units.iter().map(|u| u.id).collect();

                                                // Move the unit to the target slot
                                                let moved_unit = unit_ids.remove(*source_slot_idx);

                                                // Insert at target position, or append if target is at end
                                                if slot_idx >= unit_ids.len() {
                                                    unit_ids.push(moved_unit);
                                                } else {
                                                    unit_ids.insert(slot_idx, moved_unit);
                                                }

                                                // Call the reorder reducer
                                                cn().reducers
                                                    .match_reorder_fusion_units(fusion.id, unit_ids)
                                                    .notify_error_op();
                                            }
                                        }
                                    }
                                    if let Some(payload) =
                                        egui::DragAndDrop::payload::<(usize, ShopSlot)>(ui.ctx())
                                    {
                                        if payload.1.card_kind == CardKind::Unit {
                                            if let Some(shop_item) =
                                                DndArea::<(usize, ShopSlot)>::new(resp.rect)
                                                    .id(format!(
                                                        "unit_buy_empty_slot_{}_{}",
                                                        fusion_idx, slot_idx
                                                    ))
                                                    .text_fn(ui, |(_, slot)| {
                                                        format!(
                                                            "play unit [yellow -{}g]",
                                                            slot.price
                                                        )
                                                    })
                                                    .ui(ui)
                                            {
                                                cn().reducers
                                                    .match_play_unit(
                                                        shop_item.0 as u8,
                                                        fusion.slot as u8,
                                                    )
                                                    .notify_error_op();
                                            }
                                        }
                                    }
                                }
                            }

                            ui.add_space(5.0);

                            if "buy slot".cstr().button(ui).clicked() {
                                cn().reducers
                                    .match_buy_fusion_lvl(fusion.slot as u8)
                                    .notify_error_op();
                            }

                            Ok(())
                        })
                        .inner;

                    if let Err(e) = result {
                        ui.label(format!("Error: {}", e));
                    }
                }
            });

            if let Some(payload) = egui::DragAndDrop::payload::<(usize, ShopSlot)>(ui.ctx()) {
                if payload.1.card_kind == CardKind::House {
                    if let Some(shop_item) = DndArea::<(usize, ShopSlot)>::new(rect)
                        .text_fn(ui, |(_, slot)| {
                            format!("play house [yellow -{}g]", slot.price)
                        })
                        .ui(ui)
                    {
                        cn().reducers
                            .match_play_house(shop_item.0 as u8)
                            .notify_error_op();
                    }
                }
            }

            Ok(())
        })
    }
}
