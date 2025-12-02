use bevy_egui::egui::{MenuBar, UiKind};

use super::*;

pub struct TopBar;

impl TopBar {
    fn state_btn(
        state: GameState,
        ui: &mut Ui,
        world: &mut World,
        menu: impl FnOnce(&mut Ui, &mut World),
    ) {
        let active = cur_state(world) == state;
        let text = state.to_string().to_lowercase();
        if active {
            ui.menu_button(text.cstr_c(YELLOW).widget(1.0, ui.style()), |ui| {
                menu(ui, world)
            });
        } else {
            if text.button(ui).clicked() {
                state.set_next(world);
            }
        }
    }
    pub fn ui(ui: &mut Ui, world: &mut World) {
        MenuBar::new().ui(ui, |ui| {
            Self::state_btn(GameState::Title, ui, world, |_, _| {});
            Self::state_btn(GameState::Editor, ui, world, |ui, world| {
                if "reset state".cstr().button(ui).clicked() {
                    pd_mut(|d| d.client_state.battle_test = default());
                    BattleEditorPlugin::load_from_client_state(world);

                    ui.close_kind(UiKind::Menu);
                }
            });
            Self::state_btn(GameState::Incubator, ui, world, |_, _| {});
            ui.menu_button("settings", |ui| {
                if "reset tiles".cstr().button(ui).clicked() {
                    pd_mut(|d| {
                        d.client_state.tile_states.clear();
                    });
                    TilePlugin::load_state_tree(cur_state(world), world);
                    ui.close_kind(UiKind::Menu);
                }

                if "edit settings".cstr().button(ui).clicked() {
                    show_settings_confirmation(world);
                    ui.close_kind(UiKind::Menu);
                }
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let Some(fps) = world
                    .resource::<DiagnosticsStore>()
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                {
                    if let Some(fps) = fps.smoothed() {
                        format!("[tl fps:] {fps:.0}").cstr().label(ui);
                    }
                }
                VERSION.cstr().label(ui);
                current_server()
                    .1
                    .cstr_cs(low_contrast_text(), CstrStyle::Bold)
                    .label(ui);
            })
        });
    }
}
