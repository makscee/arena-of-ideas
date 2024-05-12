use bevy::window::{PresentMode, WindowResized};

use super::*;

pub struct SettingsPlugin;

#[derive(Resource, Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct SettingsData {
    pub master_volume: f64,
    pub expanded_hint: bool,
    pub always_show_card: bool,
    pub dev_mode: bool,
    pub disable_console_output: bool,
    pub window_mode: WindowMode,
    pub resolution: Vec2,
    pub vsync: VsyncMode,
}

#[derive(
    Default, Serialize, Deserialize, Debug, Copy, Clone, PartialEq, EnumString, EnumIter, Display,
)]
pub enum WindowMode {
    #[default]
    Windowed,
    FullScreen,
    BorderlessFullScreen,
}

#[derive(
    Default, Serialize, Deserialize, Debug, Copy, Clone, PartialEq, EnumString, EnumIter, Display,
)]
pub enum VsyncMode {
    #[default]
    On,
    Off,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            master_volume: 0.0,
            expanded_hint: false,
            resolution: vec2(1280.0, 720.0),
            window_mode: default(),
            always_show_card: default(),
            dev_mode: default(),
            disable_console_output: false,
            vsync: default(),
        }
    }
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::init)
            .add_systems(Update, Self::on_resize);
    }
}

const PKV_SETTINGS_KEY: &str = "settings";
impl SettingsPlugin {
    fn init(world: &mut World) {
        let data = SettingsData::load(world);
        Self::updated(data, world);
    }

    pub fn ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };
        let mut data = *SettingsData::get(world);
        window("SETTINGS")
            .set_width(400.0)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                frame(ui, |ui| {
                    ui.columns(2, |ui| {
                        "master volume".to_colored().label(&mut ui[0]);
                        Slider::new(&mut data.master_volume, 0.0..=1.0)
                            .step_by(0.01)
                            .ui(&mut ui[1]);
                    })
                });
                frame(ui, |ui| {
                    ui.columns(2, |ui| {
                        "always expand hint".to_colored().label(&mut ui[0]);
                        ui[1].vertical_centered_justified(|ui| {
                            let value = &mut data.expanded_hint;
                            if ui
                                .button_or_primary(
                                    if *value { "ENABLED" } else { "DISABLED" },
                                    *value,
                                )
                                .clicked()
                            {
                                *value = !*value;
                            }
                        });
                    });
                });
                frame(ui, |ui| {
                    ui.columns(2, |ui| {
                        "always show card".to_colored().label(&mut ui[0]);
                        ui[1].vertical_centered_justified(|ui| {
                            let value = &mut data.always_show_card;
                            if ui
                                .button_or_primary(
                                    if *value { "ENABLED" } else { "DISABLED" },
                                    *value,
                                )
                                .clicked()
                            {
                                *value = !*value;
                            }
                        });
                    });
                });
                frame(ui, |ui| {
                    let value = &mut data.window_mode;
                    ui.columns(2, |ui| {
                        "window mode".to_colored().label(&mut ui[0]);
                        ui[1].vertical_centered_justified(|ui| {
                            ComboBox::from_id_source("window mode")
                                .width(240.0)
                                .selected_text(value.to_string())
                                .show_ui(ui, |ui| {
                                    for option in WindowMode::iter() {
                                        let text = option.to_string();
                                        ui.selectable_value(value, option, text).changed();
                                    }
                                });
                        });
                    });
                });
                frame(ui, |ui| {
                    let value = &mut data.vsync;
                    ui.columns(2, |ui| {
                        "vsync".to_colored().label(&mut ui[0]);
                        ui[1].vertical_centered_justified(|ui| {
                            ComboBox::from_id_source("vsync mode")
                                .width(240.0)
                                .selected_text(value.to_string())
                                .show_ui(ui, |ui| {
                                    for option in VsyncMode::iter() {
                                        let text = option.to_string();
                                        ui.selectable_value(value, option, text).changed();
                                    }
                                });
                        });
                    });
                });
                frame(ui, |ui| {
                    ui.columns(3, |ui| {
                        "resolution".to_colored().label(&mut ui[0]);
                        ui[1].vertical_centered_justified(|ui| {
                            DragValue::new(&mut data.resolution.x).ui(ui);
                        });
                        ui[2].vertical_centered_justified(|ui| {
                            DragValue::new(&mut data.resolution.y).ui(ui);
                        });
                    })
                });

                frame(ui, |ui| {
                    ui.columns(2, |ui| {
                        "dev mode".to_colored().label(&mut ui[0]);
                        ui[1].vertical_centered_justified(|ui| {
                            let value = &mut data.dev_mode;
                            if ui
                                .button_or_primary(
                                    if *value { "ENABLED" } else { "DISABLED" },
                                    *value,
                                )
                                .clicked()
                            {
                                *value = !*value;
                            }
                        });
                    });
                });
                frame(ui, |ui| {
                    ui.columns(2, |ui| {
                        "console output".to_colored().label(&mut ui[0]);
                        ui[1].vertical_centered_justified(|ui| {
                            let value = &mut data.disable_console_output;
                            if ui
                                .button_or_primary(
                                    if *value { "ENABLED" } else { "DISABLED" },
                                    *value,
                                )
                                .clicked()
                            {
                                *value = !*value;
                            }
                            if *value {
                                log::set_max_level(LevelFilter::Info);
                            } else {
                                log::set_max_level(LevelFilter::Off);
                            }
                        });
                    });
                });
                frame(ui, |ui| {
                    if ui.button_red("RESET TO DEFAULTS").clicked() {
                        data = default();
                    }
                });
                frame(ui, |ui| {
                    if ui
                        .button_red("CLEAR DATA")
                        .on_hover_text("Clear any persistent data")
                        .clicked()
                    {
                        PersistentData::default().save(world).unwrap();
                        SettingsData::default().save(world).unwrap();
                    }
                });
                ui.add_space(30.0);
                frame(ui, |ui| {
                    if ui.button_primary("APPLY").clicked() {
                        Self::updated(data, world);
                    }
                });
            });
        if !data.eq(SettingsData::get(world)) {
            data.save(world).unwrap();
        }
    }

    fn on_resize(mut resize_reader: EventReader<WindowResized>) {
        for e in resize_reader.read() {
            debug!("On resize {e:?}");
            let size = vec2(e.width, e.height);
            OperationsPlugin::add(move |world| {
                let mut settings = SettingsData::load(world);
                settings.resolution = size;
                settings.save(world).unwrap();
            });
        }
    }

    fn updated(data: SettingsData, world: &mut World) {
        data.save(world).unwrap();
        AudioPlugin::update_settings(&data, world);
        if let Some(mut window) = world
            .query::<&mut bevy::window::Window>()
            .iter_mut(world)
            .next()
        {
            window.mode = match data.window_mode {
                WindowMode::Windowed => bevy::window::WindowMode::Windowed,
                WindowMode::FullScreen => bevy::window::WindowMode::Fullscreen,
                WindowMode::BorderlessFullScreen => bevy::window::WindowMode::BorderlessFullscreen,
            };
            window.resolution.set(data.resolution.x, data.resolution.y);
            window.present_mode = match data.vsync {
                VsyncMode::On => PresentMode::AutoVsync,
                VsyncMode::Off => PresentMode::AutoNoVsync,
            }
        }
    }
}

impl SettingsData {
    pub fn load(world: &mut World) -> Self {
        let data = match world.resource::<PkvStore>().get::<Self>(PKV_SETTINGS_KEY) {
            Ok(value) => value,
            Err(_) => default(),
        };
        world.insert_resource(data);
        data
    }

    pub fn save(self, world: &mut World) -> Result<Self> {
        world
            .resource_mut::<PkvStore>()
            .set(PKV_SETTINGS_KEY, &self)
            .map_err(|e| anyhow!("{}", e.to_string()))?;
        world.insert_resource(self);
        Ok(self)
    }

    pub fn get(world: &World) -> &Self {
        world.resource::<SettingsData>()
    }
}
