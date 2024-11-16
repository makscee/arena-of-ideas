use super::*;

pub struct SubstateMenu;

impl SubstateMenu {
    pub fn show(options: &[GameState], ui: &mut Ui, world: &mut World) -> bool {
        let state = cur_state(world);
        let mut clicked = false;
        ui.horizontal(|ui| {
            ui.visuals_mut().widgets.hovered.bg_fill = VISIBLE_LIGHT;
            for i in options {
                let active = i.eq(&state);
                let name = i.get_name().to_owned();
                if Button::new(name).active(active).ui(ui).clicked() {
                    i.proceed_to_target(world);
                    clicked = true;
                }
            }
        });
        clicked
    }
}
