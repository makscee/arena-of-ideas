use super::*;

pub struct PanelsPlugin;

#[derive(Resource)]
struct TopOpenWindows(ordered_hash_map::OrderedHashMap<TopButton, bool>);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, EnumIter, Display)]
enum TopButton {
    Exit,
    Settings,
    Profile,
    Leaderboard,
}

impl Plugin for PanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::ui)
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

    fn enabled(&self) -> bool {
        match self {
            TopButton::Profile | TopButton::Leaderboard => identity().is_ok(),
            TopButton::Exit | TopButton::Settings => true,
        }
    }

    fn click(&self, world: &mut World) {
        let open = match self {
            TopButton::Exit => {
                world
                    .resource::<State<GameState>>()
                    .get()
                    .clone()
                    .exit(world);
                false
            }
            TopButton::Settings | TopButton::Profile | TopButton::Leaderboard => {
                let mut data = world.resource_mut::<TopOpenWindows>();
                let entry = data.0.get_mut(self).unwrap();
                *entry = !*entry;
                *entry
            }
        };
        if open && self.eq(&TopButton::Leaderboard) {
            LeaderboardPlugin::load(world);
        }
        if open && self.eq(&TopButton::Profile) {
            ProfilePlugin::load(world);
        }
    }

    fn show(&self, world: &mut World) {
        match self {
            TopButton::Settings => SettingsPlugin::ui(world),
            TopButton::Profile => ProfilePlugin::ui(world),
            TopButton::Leaderboard => LeaderboardPlugin::ui(world),
            TopButton::Exit => {}
        }
    }
}

impl PanelsPlugin {
    pub fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        let top_data = world.resource::<TopOpenWindows>().0.clone();
        TopBottomPanel::top("top")
            .frame(
                Frame::window(&ctx.style())
                    .rounding(0.0)
                    .stroke(Stroke::NONE),
            )
            .show(&egui_context(world), |ui| {
                let mut margin = Margin::same(4.0);
                margin.top = 1.0;
                Frame::none().inner_margin(margin).show(ui, |ui| {
                    let columns = top_data.len();
                    ui.set_max_width(columns as f32 * 150.0);
                    ui.columns(columns, |ui| {
                        for (ind, (t, value)) in top_data.iter().enumerate() {
                            ui[ind].vertical_centered_justified(|ui| {
                                ui.set_enabled(t.enabled());
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
                });
            });
        for (button, value) in top_data {
            if value {
                button.show(world);
            }
        }
    }
}
