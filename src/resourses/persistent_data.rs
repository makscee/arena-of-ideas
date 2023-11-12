use super::*;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct PersistentData {
    pub last_state: Option<GameState>,
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

    pub fn save_last_state(state: GameState, world: &mut World) {
        let mut pd = Self::load(world);
        pd.last_state = Some(state);
        pd.save(world).unwrap();
    }
}
