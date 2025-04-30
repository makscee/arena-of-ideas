use bevy_egui::egui::Grid;
use spacetimedb_sdk::DbContext;

use super::*;

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shop), Self::on_enter);
    }
}

impl MatchPlugin {
    pub fn check_battles(world: &mut World) -> Result<(), ExpressionError> {
        Context::from_world_r(world, |context| {
            let m = player(context)?.active_match_err(context)?;
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
            let m = player(context)?.active_match_err(context)?;
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
    fn show_unit(unit: &NUnit, rect: Rect, context: &Context, ui: &mut Ui) -> Option<()> {
        let d = unit.description_load(context)?;
        if let Some(r) = d.representation_load(context) {
            r.paint(rect, context, ui).ui(ui);
        }
        unit_rep().paint(rect, context, ui).ui(ui);
        Some(())
    }
    pub fn pane_shop(ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        Context::from_world_ref_r(world, |context| {
            let m = player(context)?.active_match_err(context)?;
            let slots = m.shop_case_load(context);
            if slots.is_empty() {
                return Err("Shop case slots are empty".into());
            }
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
                ui.columns(slots.len(), |ui| {
                    for i in 0..slots.len() {
                        let ui = &mut ui[i];
                        let slot = &slots[i];
                        ui.with_layout(
                            Layout::bottom_up(Align::Center).with_cross_justify(true),
                            |ui| {
                                if "buy"
                                    .cstr()
                                    .as_button()
                                    .enabled(!slot.sold)
                                    .ui(ui)
                                    .clicked()
                                {
                                    cn().reducers.match_buy(slot.id()).notify_op();
                                }
                                if !slot.sold {
                                    if let Ok(unit) = NUnit::get_by_id(slot.unit, context) {
                                        context.with_layer_ref(
                                            ContextLayer::Owner(unit.entity()),
                                            |context| {
                                                slot_rect_button(ui, |rect, ui| {
                                                    if Self::show_unit(&unit, rect, context, ui)
                                                        .is_none()
                                                    {
                                                        "Failed to show unit".cstr_c(RED).label(ui);
                                                    }
                                                })
                                                .on_hover_ui(|ui| {
                                                    unit.show_card(context, ui).ui(ui);
                                                });
                                            },
                                        );
                                    } else {
                                        "Core unit not found".cstr_c(RED).label(ui);
                                    }
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
            let m = player(context)?.active_match_err(context)?;
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
            let m = player(context)?.active_match_err(context)?;
            let team = m.team_err(context)?;
            for house in team.houses_load(context) {
                house
                    .tag_card(TagCardContext::new().expanded(true), context, ui)
                    .ui(ui);
            }
            Ok(())
        })
    }
    pub fn pane_team(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        Context::from_world_r(world, |context| {
            let m = player(context)?.active_match_err(context)?;
            let team = m.team_err(context)?.entity();
            NFusion::slots_editor(
                team,
                context,
                ui,
                |ui| {
                    ui.vertical_centered_justified(|ui| {
                        if "buy fusion".cstr().button(ui).clicked() {
                            cn().reducers.match_buy_fusion().notify_op();
                        }
                    });
                },
                |fusion| {
                    cn().reducers
                        .match_edit_fusion(fusion.to_tnode())
                        .notify_op();
                },
                |_, _| todo!(),
                |_, _| todo!(),
            )
            .ui(ui);
            Ok(())
        })
    }
    pub fn pane_match_over(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        Context::from_world_r(world, |context| {
            let m = player(context)?.active_match_err(context)?;
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
    pub fn load_battle(context: &mut Context) -> Result<(), ExpressionError> {
        GameState::Battle.set_next(context.world_mut()?);
        let battles = player(context)?
            .active_match_err(context)?
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
}
