use super::*;

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Match), Self::on_enter);
    }
}

impl MatchPlugin {
    fn on_enter(world: &mut World) {}
    fn show_unit(unit: &Unit, rect: Rect, context: &Context, ui: &mut Ui) -> Option<()> {
        let d = unit.description_load(context)?;
        let context = &context.clone().set_owner(unit.entity()).take();
        if let Some(r) = d.representation_load(context) {
            r.paint(rect, context, ui).ui(ui);
        }
        unit_rep().paint(rect, context, ui).ui(ui);
        Some(())
    }
    pub fn pane_shop(ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        let context = &world.into();
        let m = player(context)?
            .active_match_load(context)
            .to_e("Active match not found")?;
        let slots = m.shop_case_load(context);
        if slots.is_empty() {
            return Err("Shop case slots are empty".into());
        }
        let slot_size = (ui.available_width() / (slots.len() as f32))
            .at_most(ui.available_height())
            .v2();
        let available_rect = ui.available_rect_before_wrap();
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                format!("g: [yellow [b {}]]", m.g).label(ui);
                if "reroll".cstr().button(ui).clicked() {
                    cn().reducers.match_reroll().notify_op();
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
                            if "buy".cstr().button(ui).clicked() {
                                cn().reducers.match_buy(slot.id()).notify_op();
                            }
                            slot.id().cstr().label(ui);
                            let _ = RectButton::new_size(ui.available_size()).ui(
                                ui,
                                |color, rect, _, ui| {
                                    let rect = rect.shrink(5.0);
                                    corners_rounded_rect(
                                        rect,
                                        slot_size.x * 0.1,
                                        color.stroke(),
                                        ui,
                                    );
                                    if !slot.sold {
                                        if let Some(unit) = Unit::get_by_id(slot.unit, context) {
                                            if Self::show_unit(unit, rect, context, ui).is_none() {
                                                "Failed to show unit".cstr_c(RED).label(ui);
                                            }
                                        } else {
                                            "Core unit not found".cstr_c(RED).label(ui);
                                        }
                                    }
                                },
                            );
                        },
                    );
                }
            });
        });
        Ok(())
    }
    pub fn pane_roster(ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        Ok(())
    }
    pub fn pane_team(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        Ok(())
    }
}
