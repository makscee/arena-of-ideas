use bevy_egui::egui::Checkbox;

use super::*;

pub struct SettingsPlugin;

#[derive(Resource, Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct SettingsData {
    pub master_volume: f64,
    pub expanded_hint: bool,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            master_volume: 0.5,
            expanded_hint: false,
        }
    }
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::init);
    }
}

const PKV_SETTINGS_KEY: &str = "settings";
impl SettingsPlugin {
    fn init(world: &mut World) {
        let data = SettingsData::load(world);
        Self::updated(data, world);
    }

    pub fn ui(world: &mut World) {
        let mut data = *SettingsData::get(world);
        window("SETTINGS").show(&egui_context(world), |ui| {
            frame(ui, |ui| {
                let master_volume =
                    Slider::new(&mut data.master_volume, 0.0..=1.0).text("master volume");
                ui.add(master_volume);
            });
            frame(ui, |ui| {
                let expanded_hint = Checkbox::new(&mut data.expanded_hint, "always expanded hint");
                ui.add(expanded_hint);
            });
            frame(ui, |ui| {
                if ui
                    .button("CLEAR DATA")
                    .on_hover_text("Clear saved game and other data")
                    .clicked()
                {
                    PersistentData::default().save(world).unwrap();
                    SettingsData::default().save(world).unwrap();
                }
            });
        });
        if !data.eq(SettingsData::get(world)) {
            Self::updated(data, world);
        }
    }

    fn updated(data: SettingsData, world: &mut World) {
        data.save(world).unwrap();
        AudioPlugin::update_settings(&data, world);
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
