use super::*;

#[derive(Resource)]
pub struct ExplorerState {
    pub inspected_unit: Option<u64>,
    pub inspected_house: Option<u64>,

    pub pending_actions: Vec<ExplorerAction>,
}

impl Default for ExplorerState {
    fn default() -> Self {
        Self {
            inspected_unit: None,
            inspected_house: None,
            pending_actions: Vec::new(),
        }
    }
}
