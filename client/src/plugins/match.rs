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
                        cn().reducers.match_shop_reroll().notify_op();
                    }
                    ui.add_space(20.0);
                    if "Start Battle".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                        op(|world| {
                            GameState::Battle.set_next(world);
                        });
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
                let (_fusion_id, _slot_idx, unit_id) = payload.as_ref();
                cn().reducers.match_sell_unit(*unit_id).notify_op();
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
                cn().reducers.match_shop_buy(card.0 as u8).notify_op();
            }
            Ok(())
        })
    }
    pub fn pane_team(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        Context::from_world_r(world, |context| {
            let m = player(context)?.active_match_load(context)?;
            let team = m.team_load(context)?;
            let rect = ui.available_rect_before_wrap();

            let mut team_editor = TeamEditor::new(team.entity());
            team_editor = team_editor.filled_slot_action("Sell Unit");

            if let Ok(actions) = team_editor.ui(ui, context) {
                for action in actions {
                    match action {
                        TeamAction::MoveUnit { unit_id, target } => {
                            cn().reducers
                                .match_move_unit(unit_id, target)
                                .notify_error_op();
                        }
                        TeamAction::AddSlot { fusion_id } => {
                            cn().reducers
                                .match_buy_fusion_slot(fusion_id)
                                .notify_error_op();
                        }
                        TeamAction::ContextMenuAction {
                            unit_id: Some(unit_id),
                            action_name,
                            ..
                        } => {
                            if action_name == "Sell Unit" {
                                cn().reducers.match_sell_unit(unit_id).notify_error_op();
                            }
                        }
                        TeamAction::BenchUnit { unit_id } => {
                            cn().reducers.match_bench_unit(unit_id).notify_error_op();
                        }
                        _ => {}
                    }
                }
            }

            if let Some(card) = DndArea::<(usize, ShopSlot)>::new(rect)
                .text_fn(ui, |slot| format!("buy [yellow -{}g]", slot.1.price))
                .ui(ui)
            {
                cn().reducers.match_shop_buy(card.0 as u8).notify_error_op();
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
            fusion.get_action_count(context).unwrap_or(0),
            fusion.actions_limit
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
            let max_slots = fusion.slots_load(context).len();

            for slot_idx in 0..max_slots {
                if let Some(unit) = units.get(slot_idx) {
                    Self::render_unit_icon(ui, context, unit);
                } else {
                    Self::render_empty_slot(ui, context, fusion, fusion_idx, slot_idx);
                }
            }

            Ok(())
        })
        .inner
    }

    fn render_unit_icon(ui: &mut Ui, context: &Context, unit: &NUnit) {
        let _resp = if let Ok(rep) = context.first_parent_recursive::<NUnitRepresentation>(unit.id)
        {
            MatRect::new(egui::Vec2::new(60.0, 60.0))
                .add_mat(&rep.material, unit.id)
                .unit_rep_with_default(unit.id)
                .ui(ui, context)
        } else {
            MatRect::new(egui::Vec2::new(60.0, 60.0)).ui(ui, context)
        };

        if let Ok(state) = unit.state_load(context) {
            ui.label(format!("Stacks {}", state.stacks));
        }
    }

    fn render_empty_slot(
        ui: &mut Ui,
        context: &Context,
        _fusion: &NFusion,
        _fusion_idx: usize,
        _slot_idx: usize,
    ) {
        let _resp = MatRect::new(egui::Vec2::new(60.0, 60.0)).ui(ui, context);
    }
}
