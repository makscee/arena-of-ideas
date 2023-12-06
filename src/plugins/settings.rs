use super::*;

pub struct SettingsPlugin;

#[derive(Resource, Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct SettingsData {
    pub last_state_on_load: bool,
    pub master_volume: f64,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            last_state_on_load: Default::default(),
            master_volume: 0.5,
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

    pub fn ui(ui: &mut Ui, world: &mut World) {
        let mut data = *SettingsData::get(world);
        CollapsingHeader::new(RichText::new("Settings").size(25.0)).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.checkbox(&mut data.last_state_on_load, "load from last state");
                let master_volume =
                    Slider::new(&mut data.master_volume, 0.0..=1.0).text("master volume");
                ui.add(master_volume);
            })
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

    pub fn get(world: &mut World) -> &Self {
        world.resource::<SettingsData>()
    }
}
