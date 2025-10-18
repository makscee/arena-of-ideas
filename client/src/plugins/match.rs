use bevy::input::common_conditions::input_just_pressed;
use bevy_egui::egui::Grid;

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
    pub fn check_battles(world: &mut World) -> NodeResult<()> {
        world.with_context(|ctx| {
            let m = player(ctx)?.active_match_ref(ctx)?;
            if let Some(last) = m.battles_ref(ctx)?.last() {
                if last.result.is_none() {
                    GameState::Battle.set_next(ctx.world_mut()?);
                    // MatchPlugin::load_battle(ctx)?;
                    todo!();
                    return Ok(());
                }
            }
            Ok(())
        })
    }
    pub fn check_active(world: &mut World) -> NodeResult<()> {
        world.with_context(|ctx| {
            let m = player(ctx)?.active_match_ref(ctx)?;
            if !m.active {
                GameState::MatchOver.set_next(ctx.world_mut()?);
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
        if let Err(e) = Self::check_battles(world) {
            e.cstr().notify_error(world);
            return;
        }
    }
    fn add_g() {
        cn().reducers.admin_add_gold().notify_op();
    }
    fn on_match_update(mut events: MessageReader<StdbNodeEvent>) {
        for event in events.read() {
            if event.node.kind == "NMatch" && event.node.owner == player_id() {
                op(|world| {
                    world
                        .with_context(|ctx| {
                            let world = ctx.world_mut()?;
                            Self::check_active(world).notify(world);
                            Self::check_battles(world).notify(world);
                            Ok(())
                        })
                        .log();
                });
            }
        }
    }
    pub fn pane_shop(ui: &mut Ui, world: &World) -> NodeResult<()> {
        world.with_context(|ctx| {
            let mut m = player(ctx)?.active_match_ref(ctx)?.clone();
            // Check for unresolved battles
            if let Some(last_battle) = m.battles_load(ctx)?.last() {
                if last_battle.result.is_none() {
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
            }

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
                                if !slot.sold {
                                    ctx.with_owner(slot.node_id, |ctx| {
                                        ui.push_id(i, |ui| -> NodeResult<()> {
                                            let resp = match slot.card_kind {
                                                CardKind::Unit => {
                                                    let unit = ctx.load::<NUnit>(slot.node_id)?;
                                                    unit.as_card().compose(ctx, ui);
                                                    ui.response()
                                                }
                                                CardKind::House => {
                                                    let house = ctx.load::<NHouse>(slot.node_id)?;
                                                    house.as_card().compose(ctx, ui);
                                                    ui.response()
                                                }
                                            };
                                            resp.dnd_set_drag_payload((i, slot.clone()));
                                            Ok(())
                                        })
                                        .inner
                                    })
                                    .ui(ui);
                                }
                            },
                        );
                    }
                });
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
    pub fn pane_info(ui: &mut Ui, world: &World) -> NodeResult<()> {
        world.with_context(|ctx| {
            let m = player(ctx)?.active_match_ref(ctx)?;
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
            });
            Ok(())
        })
    }
    pub fn pane_roster(ui: &mut Ui, world: &World) -> NodeResult<()> {
        world.with_context(|ctx| {
            let m = player(ctx)?.active_match_ref(ctx)?;
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
    pub fn pane_team(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.with_context(|ctx| {
            let m = player(ctx)?.active_match_ref(ctx)?;
            let team = m.team_ref(ctx)?;
            let rect = ui.available_rect_before_wrap();

            let team_editor = TeamEditor::new().filled_slot_action(
                "Sell Unit".to_string(),
                Box::new(|_team, _fusion_id, unit_id, _slot_index| {
                    cn().reducers.match_sell_unit(unit_id).notify_error_op();
                }),
            );

            let (changed_team, actions) = team_editor.edit(team, ui);

            // If team changed, we'd need to apply it back to the server
            // but for now we'll just handle the actions
            for action in actions {
                match action {
                    TeamAction::MoveUnit { unit_id, target } => {
                        match target {
                            crate::plugins::team_editor::UnitTarget::Slot {
                                fusion_id,
                                slot_index,
                            } => {
                                // Convert to old format for server compatibility
                                // This is a temporary solution
                                cn().reducers
                                    .match_move_unit(unit_id, fusion_id)
                                    .notify_error_op();
                            }
                            crate::plugins::team_editor::UnitTarget::Bench => {
                                cn().reducers.match_bench_unit(unit_id).notify_error_op();
                            }
                        }
                    }
                    TeamAction::AddSlot { fusion_id } => {
                        cn().reducers
                            .match_buy_fusion_slot(fusion_id)
                            .notify_error_op();
                    }
                    TeamAction::BenchUnit { unit_id } => {
                        cn().reducers.match_bench_unit(unit_id).notify_error_op();
                    }
                    _ => {}
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
    pub fn pane_match_over(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.with_context(|ctx| {
            let m = player(ctx)?.active_match_ref(ctx)?;
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
            let world = ctx.world_mut()?;
            ui.vertical_centered(|ui| {
                if "Done".cstr().button(ui).clicked() {
                    cn().reducers.match_complete().notify(world);
                    GameState::Title.set_next(world);
                }
            });
            Ok(())
        })
    }
    pub fn pane_leaderboard(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.with_context(|ctx| {
            let world = ctx.world_mut()?;

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
                    |ctx, ui, _, value| {
                        let id = value.get_u64()?;
                        ctx.load::<NPlayer>(id)?.player_name.cstr().label(ui);
                        Ok(())
                    },
                    |context, t| {
                        let team = t.team_ref(context)?;
                        Ok(team.owner.into())
                    },
                )
                .ui(ctx, ui);
            Ok(())
        })
    }

    pub fn pane_fusion(ui: &mut Ui, world: &World) -> NodeResult<()> {
        world.with_context(|context| {
            let m = player(context)?.active_match_ref(context)?;
            let team = m.team_ref(context)?;
            let fusions = team.fusions_ref(context)?;

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
        context: &ClientContext,
        fusions: &Vec<&NFusion>,
    ) -> NodeResult<()> {
        ui.columns(fusions.len(), |columns| {
            for (fusion_idx, fusion) in fusions.iter().enumerate() {
                let ui = &mut columns[fusion_idx];
                Self::render_fusion_header(ui, context, fusion);
            }
        });
        Ok(())
    }

    fn render_fusion_header(ui: &mut Ui, context: &ClientContext, fusion: &NFusion) {
        let mut mat_rect = MatRect::new(egui::Vec2::new(80.0, 80.0));

        if let Ok(units) = context.collect_children::<NUnit>(fusion.id) {
            for unit in units {
                if let Ok(rep) = unit
                    .description_ref(context)
                    .and_then(|d| d.representation_ref(context))
                {
                    mat_rect = mat_rect.add_mat(&rep.material, unit.id);
                }
            }
        }

        mat_rect
            .unit_rep_with_default(fusion.id)
            .ui(ui, context)
            .on_hover_ui(|ui| {
                fusion.as_card().compose(context, ui);
            });
        ui.label(format!(
            "{}/{}",
            fusion.get_action_count(context).unwrap_or(0),
            fusion.actions_limit
        ));

        if let Ok(trigger) = NFusion::get_trigger(context, fusion.trigger_unit) {
            ui.horizontal(|ui| {
                Icon::Lightning.show(ui);
                trigger.title(context).label(ui);
            });
        }
    }

    fn render_fusion_units(
        ui: &mut Ui,
        context: &ClientContext,
        fusions: &Vec<&NFusion>,
    ) -> NodeResult<()> {
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
        context: &ClientContext,
        fusion: &NFusion,
        fusion_idx: usize,
    ) -> NodeResult<()> {
        ui.vertical(|ui| -> NodeResult<()> {
            let units = fusion.units(context).unwrap_or_default();
            let max_slots = fusion.slots_ref(context)?.len();

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

    fn render_unit_icon(ui: &mut Ui, context: &ClientContext, unit: &NUnit) {
        let _resp = if let Ok(desc) = unit.description_ref(context) {
            if let Ok(rep) = desc.representation_ref(context) {
                MatRect::new(egui::Vec2::new(60.0, 60.0))
                    .add_mat(&rep.material, unit.id)
                    .unit_rep_with_default(unit.id)
                    .ui(ui, context)
            } else {
                MatRect::new(egui::Vec2::new(60.0, 60.0)).ui(ui, context)
            }
        } else {
            MatRect::new(egui::Vec2::new(60.0, 60.0)).ui(ui, context)
        };

        if let Ok(state) = unit.state_ref(context) {
            ui.label(format!("Stacks {}", state.stacks));
        }
    }

    fn render_empty_slot(
        ui: &mut Ui,
        context: &ClientContext,
        _fusion: &NFusion,
        _fusion_idx: usize,
        _slot_idx: usize,
    ) {
        let _resp = MatRect::new(egui::Vec2::new(60.0, 60.0)).ui(ui, context);
    }
}
