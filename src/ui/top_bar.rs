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
        egui::menu::bar(ui, |ui| {
            Self::state_btn(GameState::Title, ui, world, |_, _| {});
            Self::state_btn(GameState::Editor, ui, world, |ui, world| {
                if "reset state".cstr().button(ui).clicked() {
                    pd_mut(|d| d.client_state.battle_test = default());
                    BattlePlugin::load_from_client_state(world);
                    ui.close_menu();
                }
                if "world inspector".cstr().button(ui).clicked() {
                    BattlePlugin::open_world_inspector_window(world);
                    ui.close_menu();
                }
            });
            ui.menu_button("settings", |ui| {
                if "theme".cstr().button(ui).clicked() {
                    Window::new("theme Editor", |ui, _| {
                        let mut colorix = colorix();
                        colorix.ui_mut(ui);
                        let theme = colorix.global().theme().clone();
                        pd_mut(|d| d.client_settings.theme = theme);
                    })
                    .push(world);
                    ui.close_menu();
                }
                if "reset tiles".cstr().button(ui).clicked() {
                    pd_mut(|d| {
                        d.client_state.tile_states.clear();
                    });
                    TilePlugin::load_state_tree(cur_state(world), world);
                    ui.close_menu();
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
                    .cstr_cs(tokens_global().low_contrast_text(), CstrStyle::Bold)
                    .label(ui);
            })
        });
    }
}
