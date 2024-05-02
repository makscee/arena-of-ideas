use bevy::input::common_conditions::input_just_pressed;

use super::*;

pub struct PanelsPlugin;

#[derive(Resource)]
struct TopOpenWindows(ordered_hash_map::OrderedHashMap<TopButton, bool>);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, EnumIter, Display)]
pub enum TopButton {
    Exit,
    Settings,
    Profile,
    Leaderboard,
    Help,
    Report,
}

impl Plugin for PanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                Self::ui,
                Self::close_all.run_if(input_just_pressed(KeyCode::Escape).or_else(
                    state_changed::<GameState>.and_then(not(in_state(GameState::MainMenu))),
                )),
            ),
        )
        .init_resource::<TopOpenWindows>();
    }
}

impl Default for TopOpenWindows {
    fn default() -> Self {
        Self(ordered_hash_map::OrderedHashMap::from_iter(
            TopButton::iter().map(|v| (v, false)),
        ))
    }
}

impl TopButton {
    fn name(&self) -> String {
        self.to_string().to_uppercase()
    }

    fn enabled(&self, world: &World) -> bool {
        match self {
            Self::Profile | Self::Leaderboard => LoginPlugin::is_connected(),
            Self::Exit | Self::Settings | Self::Help => true,
            Self::Report => match world.resource::<State<GameState>>().get() {
                GameState::Battle | GameState::Shop => true,
                _ => false,
            },
        }
    }

    pub fn click(&self, world: &mut World) {
        let open = match self {
            Self::Exit => {
                world
                    .resource::<State<GameState>>()
                    .get()
                    .clone()
                    .exit(world);

                false
            }

            Self::Settings | Self::Profile | Self::Leaderboard | Self::Help => {
                let mut data = world.resource_mut::<TopOpenWindows>();
                let entry = data.0.get_mut(self).unwrap();
                *entry = !*entry;
                *entry
            }
            Self::Report => true,
        };
        if open && self.eq(&Self::Profile) {
            ProfilePlugin::load(world);
        }
        if open && self.eq(&Self::Report) {
            info!("Report data saved to clipboard");
            let text = "Report data will be copied to clipboard,\nopen new thread in Discord and paste it there please!".to_owned();
            AlertPlugin::add(
                Some("SUBMIT BUG REPORT".to_owned()),
                text,
                Some(Box::new(|world| {
                    ReportPlugin::save_to_clipboard(world);
                    egui_context(world).unwrap().open_url(egui::OpenUrl {
                        url: "https://discord.com/channels/1034174161679044660/1234637719423029248"
                            .to_owned(),
                        new_tab: false,
                    })
                })),
            );
        }
    }

    fn show(&self, world: &mut World) {
        match self {
            Self::Settings => SettingsPlugin::ui(world),
            Self::Profile => ProfilePlugin::edit_ui(world),
            Self::Leaderboard => LeaderboardPlugin::ui(world),
            Self::Help => HelpPlugin::ui(world),
            Self::Exit | Self::Report => {}
        }
    }
}

impl PanelsPlugin {
    fn close_all(world: &mut World) {
        *world.resource_mut::<TopOpenWindows>() = default();
    }

    pub fn ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };
        let top_data = world.resource::<TopOpenWindows>().0.clone();
        TopBottomPanel::top("top")
            .frame(
                Frame::window(&ctx.style())
                    .rounding(0.0)
                    .stroke(Stroke::NONE),
            )
            .show(ctx, |ui| {
                let mut margin = Margin::same(4.0);
                margin.top = 2.0;
                Frame::none().inner_margin(margin).show(ui, |ui| {
                    let columns = top_data.len();
                    ui.horizontal(|ui| {
                        let width = columns as f32 * 150.0;
                        ui.set_max_width(width);
                        ui.columns(columns, |ui| {
                            for (ind, (t, value)) in top_data.iter().enumerate() {
                                ui[ind].vertical_centered_justified(|ui| {
                                    ui.set_enabled(t.enabled(world));
                                    let name = t.name();
                                    let btn = if *value {
                                        ui.button_primary(name)
                                    } else {
                                        ui.button(name)
                                    };
                                    if btn.clicked() {
                                        t.click(world);
                                    }
                                });
                            }
                        });
                        ui.set_max_width(ctx.screen_rect().width() - width - 20.0);
                        ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                            if let Some(fps) = world
                                .resource::<DiagnosticsStore>()
                                .get(&FrameTimeDiagnosticsPlugin::FPS)
                            {
                                if let Some(fps) = fps.smoothed() {
                                    ui.label(format!("fps: {fps:.0}"));
                                }
                            }
                            ui.label(format!("arena-of-ideas {VERSION}"));
                        })
                    })
                });
            });
        for (button, value) in top_data {
            if value {
                button.show(world);
            }
        }
    }
}
