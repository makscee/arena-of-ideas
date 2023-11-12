use bevy_egui::egui::CollapsingHeader;

use super::*;

pub struct SettingsPlugin;

#[derive(Resource, Serialize, Deserialize, Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct SettingsData {
    pub last_state_on_load: bool,
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::init)
            .add_systems(Update, Self::ui);
    }
}

const PKV_SETTINGS_KEY: &str = "settings";
impl SettingsPlugin {
    fn init(world: &mut World) {
        SettingsData::load(world);
    }

    fn ui(world: &mut World) {
        let mut data = *SettingsData::get(world);
        Window::new("Settings")
            .anchor(Align2::RIGHT_TOP, [-10.0, 10.0])
            .title_bar(false)
            .resizable(false)
            .show(&egui_context(world), |ui| {
                CollapsingHeader::new(RichText::new("Settings").size(25.0)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut data.last_state_on_load, "load from last state");
                    })
                })
            });
        if !data.eq(SettingsData::get(world)) {
            data.save(world).unwrap();
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

    pub fn get(world: &mut World) -> &Self {
        world.resource::<SettingsData>()
    }
}
