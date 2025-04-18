use super::*;

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, _app: &mut App) {}
}

impl MatchPlugin {
    fn show_unit(unit: &NUnit, rect: Rect, context: &Context, ui: &mut Ui) -> Option<()> {
        let d = unit.description_load(context)?;
        if let Some(r) = d.representation_load(context) {
            r.paint(rect, context, ui).ui(ui);
        }
        unit_rep().paint(rect, context, ui).ui(ui);
        Some(())
    }
    pub fn pane_shop(ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        let context = &world.into();
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
                    let slot = slots[i];
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
                                if let Some(unit) = NUnit::get_by_id(slot.unit, context) {
                                    let context = &context.clone().set_owner(unit.entity()).take();
                                    slot_rect_button(ui, |rect, ui| {
                                        if Self::show_unit(unit, rect, context, ui).is_none() {
                                            "Failed to show unit".cstr_c(RED).label(ui);
                                        }
                                    })
                                    .on_hover_ui(|ui| {
                                        unit.show_card(context, ui).ui(ui);
                                    });
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
    }
    pub fn pane_roster(ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        let context = &world.into();
        let m = player(context)?.active_match_err(context)?;
        let team = m.team_err(context)?;
        for house in team.houses_load(context) {
            house
                .tag_card(TagCardContext::new().expanded(true), context, ui)
                .ui(ui);
        }
        Ok(())
    }
    pub fn pane_team(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let context = &world.into();
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
        )
        .ui(ui);
        Ok(())
    }
    pub fn load_battle(world: &mut World) -> Result<(), ExpressionError> {
        GameState::Battle.set_next(world);
        let context = &Context::new(world);
        let battles = player(context)?
            .active_match_err(context)?
            .battles_load(context);
        if battles.is_empty() {
            return Err("No battles in current match".into());
        }
        let battle = *battles.last().unwrap();
        let left = NTeam::load_recursive(battle.team_left)
            .to_e_fn(|| format!("Failed to load Team#{}", battle.team_left))?;
        let right = NTeam::load_recursive(battle.team_right)
            .to_e_fn(|| format!("Failed to load Team#{}", battle.team_right))?;
        BattlePlugin::load_teams(left, right, world);
        Ok(())
    }
}
