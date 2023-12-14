use super::*;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct PersistentData {
    pub hero_editor_data: HeroEditorData,
    pub stored_save: Option<Save>,
}

const PERSISTENT_DATA_KEY: &str = "persistent_data";
impl PersistentData {
    pub fn load(world: &mut World) -> Self {
        world
            .resource::<PkvStore>()
            .get(PERSISTENT_DATA_KEY)
            .unwrap_or_default()
    }

    pub fn save(&self, world: &mut World) -> Result<()> {
        world
            .resource_mut::<PkvStore>()
            .set(PERSISTENT_DATA_KEY, &self)
            .map_err(|e| anyhow!("{}", e.to_string()))
    }

    pub fn set_hero_editor_data(mut self, data: HeroEditorData) -> Self {
        self.hero_editor_data = data;
        self
    }

    pub fn set_stored_save(mut self, save: Save) -> Self {
        self.stored_save = Some(save);
        self
    }
}
