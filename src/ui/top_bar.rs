use super::*;

pub struct TopBar;

impl TopBar {
    pub fn ui(ui: &mut Ui, world: &mut World) {
        egui::menu::bar(ui, |ui| {
            match cur_state(world) {
                GameState::Incubator => {
                    if ui.button("back").clicked() {
                        GameState::Title.set_next(world);
                    }
                }
                _ => {}
            }
            if ui.button("inspector").clicked() {
                todo!()
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let Some(fps) = world
                    .resource::<DiagnosticsStore>()
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                {
                    if let Some(fps) = fps.smoothed() {
                        format!("[tl fps:] {fps:.0}").label(ui);
                    }
                }
                VERSION.cstr().label(ui);
                current_server()
                    .1
                    .cstr_cs(tokens_global().low_contrast_text(), CstrStyle::Bold)
                    .label(ui);
            })
        });
    }
}
