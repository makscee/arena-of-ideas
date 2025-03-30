use bevy_egui::egui::{emath::GuiRounding, UiBuilder};

use super::*;

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Match), Self::on_enter);
    }
}

#[derive(Resource)]
struct MatchData {
    g: i32,
    shop_case: Vec<ShopCaseUnit>,
    team_world: World,
    core_world: World,
    editing_entity: Option<Entity>,
}

impl MatchPlugin {
    fn on_enter(world: &mut World) {}
    pub fn pane_shop(ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        Ok(())
    }
    pub fn pane_roster(ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        Ok(())
    }
    pub fn pane_team(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        Ok(())
    }

    fn show_shop_slot(
        sc: &ShopCaseUnit,
        i: usize,
        ui: &mut Ui,
        world: &World,
    ) -> Result<(), ExpressionError> {
        ui.with_layout(
            Layout::bottom_up(Align::Center).with_cross_justify(true),
            |ui| {
                if format!("[b buy [yellow {}g]]", sc.price)
                    .as_button()
                    .enabled(!sc.sold)
                    .ui(ui)
                    .clicked()
                {
                    cn().reducers.match_buy(sc.id()).unwrap();
                }
                let entity = world.get_id_link(sc.unit).unwrap();
                let context = &Context::new_world(&world).set_owner(entity).take();
                let name = context.get_string(VarName::name, NodeKind::Unit).unwrap();
                let color = context.get_color_any(VarName::color).unwrap();
                TagWidget::new_name(
                    name,
                    if sc.sold {
                        tokens_global().low_contrast_text()
                    } else {
                        color
                    },
                )
                .ui(ui);
                let size = ui.available_size();
                let size = size.x.at_most(size.y);
                let rect = ui
                    .allocate_new_ui(
                        UiBuilder::new().max_rect(
                            Rect::from_center_size(
                                ui.available_rect_before_wrap().center(),
                                egui::vec2(size, size),
                            )
                            .round_ui(),
                        ),
                        |ui| show_slot(i, 1, false, ui).rect,
                    )
                    .inner
                    .shrink(10.0);
                if !sc.sold {
                    RepresentationPlugin::paint_rect(rect, context, &unit_rep().material, ui).log();
                    if let Some(rep) = world.get::<Representation>(entity) {
                        RepresentationPlugin::paint_rect(rect, context, &rep.material, ui).log();
                    }
                }
                Ok(())
            },
        )
        .inner
    }
}
