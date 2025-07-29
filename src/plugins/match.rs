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

            // Handle fusion unit selling with DndArea
            if let Some(payload) = DndArea::<(u64, usize, u64)>::new(available_rect)
                .text_fn(ui, |(_, _, unit_id)| {
                    if let Ok(unit) = context.get_by_id::<NUnit>(*unit_id) {
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
                let (fusion_id, _slot_idx, unit_id) = payload.as_ref();
                cn().reducers
                    .match_sell_fusion_unit(*fusion_id, *unit_id)
                    .notify_op();
            }

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
                            .clone()
                            .see(context)
                            .tag_card_expanded(true, ui)
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

            Self::render_fusion_headers(ui, context, &fusions)?;
            ui.add_space(5.0);
            Self::render_fusion_units(ui, context, &fusions)?;
            Self::handle_house_drops(ui, rect);

            Ok(())
        })
    }

    fn render_fusion_headers(
        ui: &mut Ui,
        context: &Context,
        fusions: &[&NFusion],
    ) -> Result<(), ExpressionError> {
        ui.columns(fusions.len(), |columns| {
            for (fusion_idx, fusion) in fusions.iter().enumerate() {
                let ui = &mut columns[fusion_idx];
                Self::render_fusion_header(ui, context, fusion);
            }
        });
        Ok(())
    }

    fn render_fusion_header(ui: &mut Ui, context: &Context, fusion: &NFusion) {
        let mut mat_rect = MatRect::new(egui::Vec2::new(80.0, 80.0));

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

        mat_rect
            .unit_rep_with_default(fusion.id)
            .ui(ui, context)
            .on_hover_ui(|ui| {
                fusion.show_card(context, ui).ui(ui);
            });
        ui.label(format!(
            "{}/{}",
            fusion.get_action_count(),
            fusion.action_limit
        ));

        if let Ok(trigger) = NFusion::get_trigger(context, &fusion.trigger) {
            ui.horizontal(|ui| {
                Icon::Lightning.show(ui);
                let vctx = ViewContext::new(ui).non_interactible(true);
                trigger.view_title(vctx, context, ui);
            });
        }
    }

    fn render_fusion_units(
        ui: &mut Ui,
        context: &Context,
        fusions: &[&NFusion],
    ) -> Result<(), ExpressionError> {
        ui.columns(fusions.len(), |columns| {
            for (fusion_idx, fusion) in fusions.iter().enumerate() {
                let ui = &mut columns[fusion_idx];
                let result = Self::render_fusion_column(ui, context, fusion, fusion_idx);
                if let Err(e) = result {
                    e.ui(ui);
                }
            }
        });
        Ok(())
    }

    fn render_fusion_column(
        ui: &mut Ui,
        context: &Context,
        fusion: &NFusion,
        fusion_idx: usize,
    ) -> Result<(), ExpressionError> {
        ui.vertical(|ui| -> Result<(), ExpressionError> {
            let units = fusion.units(context).unwrap_or_default();
            let max_slots = fusion.lvl as usize;

            for slot_idx in 0..max_slots {
                if let Some(unit) = units.get(slot_idx) {
                    Self::render_unit_slot(ui, context, fusion, unit, fusion_idx, slot_idx)?;
                } else {
                    Self::render_empty_slot(ui, context, fusion, fusion_idx, slot_idx);
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
        .inner
    }

    fn render_unit_slot(
        ui: &mut Ui,
        context: &Context,
        fusion: &NFusion,
        unit: &NUnit,
        fusion_idx: usize,
        slot_idx: usize,
    ) -> Result<(), ExpressionError> {
        ui.horizontal(|ui| -> Result<(), ExpressionError> {
            let resp = Self::render_unit_icon(ui, context, unit);
            Self::render_unit_actions(ui, context, fusion, unit, slot_idx)?;
            Self::handle_unit_drag_drop(ui, context, fusion, unit, fusion_idx, slot_idx, resp);
            Ok(())
        })
        .inner?;
        ui.add_space(5.0);
        Ok(())
    }

    fn render_unit_icon(ui: &mut Ui, context: &Context, unit: &NUnit) -> Response {
        if let Ok(rep) = context.first_parent_recursive::<NUnitRepresentation>(unit.id) {
            MatRect::new(egui::Vec2::new(60.0, 60.0))
                .add_mat(&rep.material, unit.id)
                .unit_rep_with_default(unit.id)
                .ui(ui, context)
        } else {
            MatRect::new(egui::Vec2::new(60.0, 60.0)).ui(ui, context)
        }
    }

    fn render_unit_actions(
        ui: &mut Ui,
        context: &Context,
        fusion: &NFusion,
        unit: &NUnit,
        slot_idx: usize,
    ) -> Result<(), ExpressionError> {
        ui.vertical(|ui| -> Result<(), ExpressionError> {
            if let Ok(behavior) = context.first_parent_recursive::<NUnitBehavior>(unit.id) {
                let action_range = Self::get_action_range(fusion, slot_idx);
                let max_actions = Self::get_max_actions(&behavior, &fusion.trigger);

                if max_actions > 0 {
                    let (current_start, current_len) = action_range;

                    let range_selector = RangeSelector::new(max_actions)
                        .range(current_start, current_len)
                        .border_thickness(3.0)
                        .drag_threshold(12.0)
                        .show_drag_hints(false)
                        .show_debug_info(false)
                        .id(egui::Id::new(format!("range_selector_{}", unit.id)));

                    let (_, range_changed) =
                        range_selector.ui(ui, context, |item_ui, ctx, action_idx, is_in_range| {
                            if let Some(reaction) =
                                behavior.reactions.get(fusion.trigger.trigger as usize)
                            {
                                if let Some(action) = reaction.actions.get(action_idx) {
                                    let vctx = ViewContext::new(item_ui).non_interactible(true);
                                    if is_in_range {
                                        Self::render_action_normal(
                                            item_ui, ctx, unit, action, vctx,
                                        );
                                    } else {
                                        Self::render_action_greyed(
                                            item_ui, ctx, unit, action, vctx,
                                        );
                                    }
                                }
                            }
                            Ok(())
                        });

                    if let Some((new_start, new_length)) = range_changed {
                        cn().reducers
                            .match_set_fusion_unit_action_range(unit.id, new_start, new_length)
                            .notify_error_op();
                    }
                }
            }
            Ok(())
        })
        .inner
    }

    fn get_action_range(fusion: &NFusion, slot_idx: usize) -> (u8, u8) {
        let current_action_ref = fusion.behavior.get(slot_idx);
        let current_start = current_action_ref.map(|ar| ar.start).unwrap_or(0);
        let current_len = current_action_ref.map(|ar| ar.length).unwrap_or(0);
        (current_start, current_len)
    }

    fn get_max_actions(behavior: &NUnitBehavior, trigger: &UnitTriggerRef) -> u8 {
        behavior
            .reactions
            .get(trigger.trigger as usize)
            .map(|reaction| reaction.actions.len() as u8)
            .unwrap_or(0)
    }

    fn render_action_normal(
        ui: &mut Ui,
        context: &Context,
        unit: &NUnit,
        action: &Action,
        vctx: ViewContext,
    ) {
        if let Ok(entity) = context.entity(unit.id) {
            context
                .with_owner_ref(entity, |context| {
                    action.title_cstr(vctx, context).label_w(ui);
                    Ok(())
                })
                .ui(ui);
        }
    }

    fn render_action_greyed(
        ui: &mut Ui,
        context: &Context,
        unit: &NUnit,
        action: &Action,
        vctx: ViewContext,
    ) {
        let old_style = ui.visuals().clone();
        ui.visuals_mut().override_text_color = Some(egui::Color32::GRAY);

        if let Ok(entity) = context.entity(unit.id) {
            context
                .with_owner_ref(entity, |context| {
                    action
                        .title_cstr(vctx, context)
                        .as_label_alpha(0.5, ui.style())
                        .wrap()
                        .ui(ui);
                    Ok(())
                })
                .ui(ui);
        }

        *ui.visuals_mut() = old_style;
    }

    fn handle_unit_drag_drop(
        ui: &mut Ui,
        context: &Context,
        fusion: &NFusion,
        unit: &NUnit,
        fusion_idx: usize,
        slot_idx: usize,
        resp: Response,
    ) {
        if resp.dragged() {
            resp.dnd_set_drag_payload((fusion.id, slot_idx, unit.id));
            if let Some(pos) = ui.ctx().pointer_latest_pos() {
                let origin = resp.rect.center();
                let painter = ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Foreground,
                    egui::Id::new("drag_arrow"),
                ));
                painter.arrow(origin, pos - origin, ui.visuals().widgets.hovered.fg_stroke);
            }
        }

        if let Some(payload) = DndArea::<(u64, usize, u64)>::new(resp.rect)
            .id(format!("unit_slot_{}_{}", fusion_idx, slot_idx))
            .text_fn(ui, |(source_fusion_id, _, unit_id)| {
                if let Ok(unit) = context.get_by_id::<NUnit>(*unit_id) {
                    if *source_fusion_id == fusion.id {
                        format!("Swap with {}", unit.unit_name)
                    } else {
                        format!("Move {} here", unit.unit_name)
                    }
                } else {
                    "Move unit here".to_string()
                }
            })
            .ui(ui)
        {
            Self::handle_unit_drop(context, fusion, &payload, slot_idx);
        }
    }

    fn handle_unit_drop(
        context: &Context,
        fusion: &NFusion,
        payload: &(u64, usize, u64),
        slot_idx: usize,
    ) {
        let (source_fusion_id, source_slot_idx, source_unit_id) = payload;

        if *source_fusion_id == fusion.id {
            // Same fusion - swap or reorder
            if *source_slot_idx != slot_idx {
                let current_units = fusion.units(context).unwrap_or_default();
                if *source_slot_idx < current_units.len() && slot_idx < current_units.len() {
                    let mut unit_ids: Vec<u64> = current_units.iter().map(|u| u.id).collect();
                    unit_ids.swap(*source_slot_idx, slot_idx);
                    cn().reducers
                        .match_reorder_fusion_units(fusion.id, unit_ids)
                        .notify_error_op();
                }
            }
        } else {
            // Different fusion - move unit
            cn().reducers
                .match_move_unit_between_fusions(
                    *source_fusion_id,
                    fusion.id,
                    *source_unit_id,
                    slot_idx as u32,
                )
                .notify_error_op();
        }
    }

    fn render_empty_slot(
        ui: &mut Ui,
        context: &Context,
        fusion: &NFusion,
        fusion_idx: usize,
        slot_idx: usize,
    ) {
        let resp = MatRect::new(egui::Vec2::new(60.0, 60.0)).ui(ui, context);
        Self::handle_empty_slot_drops(ui, context, fusion, fusion_idx, slot_idx, resp);
    }

    fn handle_empty_slot_drops(
        ui: &mut Ui,
        context: &Context,
        fusion: &NFusion,
        fusion_idx: usize,
        slot_idx: usize,
        resp: Response,
    ) {
        // Handle unit reordering drops
        if let Some(payload) = DndArea::<(u64, usize, u64)>::new(resp.rect)
            .id(format!("empty_slot_{}_{}", fusion_idx, slot_idx))
            .text_fn(ui, |(source_fusion_id, _, unit_id)| {
                if let Ok(unit) = context.get_by_id::<NUnit>(*unit_id) {
                    if *source_fusion_id == fusion.id {
                        format!("Move {} here", unit.unit_name)
                    } else {
                        format!("Move {} to this fusion", unit.unit_name)
                    }
                } else {
                    "Move unit here".to_string()
                }
            })
            .ui(ui)
        {
            Self::handle_unit_move_to_empty(context, fusion, &payload, slot_idx);
        }

        // Handle shop unit purchases
        if let Some(payload) = egui::DragAndDrop::payload::<(usize, ShopSlot)>(ui.ctx()) {
            if payload.1.card_kind == CardKind::Unit {
                if let Some(shop_item) = DndArea::<(usize, ShopSlot)>::new(resp.rect)
                    .id(format!("unit_buy_empty_slot_{}_{}", fusion_idx, slot_idx))
                    .text_fn(ui, |(_, slot)| {
                        format!("play unit [yellow -{}g]", slot.price)
                    })
                    .ui(ui)
                {
                    cn().reducers
                        .match_play_unit(shop_item.0 as u8, fusion.slot as u8)
                        .notify_error_op();
                }
            }
        }
    }

    fn handle_unit_move_to_empty(
        context: &Context,
        fusion: &NFusion,
        payload: &(u64, usize, u64),
        slot_idx: usize,
    ) {
        let (source_fusion_id, source_slot_idx, source_unit_id) = payload;

        if *source_fusion_id == fusion.id {
            // Same fusion - reorder
            let current_units = fusion.units(context).unwrap_or_default();
            if *source_slot_idx < current_units.len() && slot_idx < fusion.lvl as usize {
                let mut unit_ids: Vec<u64> = current_units.iter().map(|u| u.id).collect();
                let moved_unit = unit_ids.remove(*source_slot_idx);
                if slot_idx >= unit_ids.len() {
                    unit_ids.push(moved_unit);
                } else {
                    unit_ids.insert(slot_idx, moved_unit);
                }
                cn().reducers
                    .match_reorder_fusion_units(fusion.id, unit_ids)
                    .notify_error_op();
            }
        } else {
            // Different fusion - move unit
            cn().reducers
                .match_move_unit_between_fusions(
                    *source_fusion_id,
                    fusion.id,
                    *source_unit_id,
                    slot_idx as u32,
                )
                .notify_error_op();
        }
    }

    fn handle_house_drops(ui: &mut Ui, rect: egui::Rect) {
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
    }
}
