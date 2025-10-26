use super::*;

#[derive(Resource)]
pub struct ExplorerState {
    pub inspected_unit: Option<String>,
    pub inspected_house: Option<String>,
    pub inspected_ability: Option<String>,
    pub inspected_status: Option<String>,

    pub view_mode: ViewMode,

    pub pending_actions: Vec<ExplorerAction>,
}

impl Default for ExplorerState {
    fn default() -> Self {
        Self {
            inspected_unit: None,
            inspected_house: None,
            inspected_ability: None,
            inspected_status: None,
            view_mode: ViewMode::default(),
            pending_actions: Vec::new(),
        }
    }
}

impl ExplorerState {
    pub fn refresh_cache(cache: &mut ExplorerCache) {
        cache.rebuild().unwrap();
    }
}
