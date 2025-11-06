use super::*;
use crate::resources::context::{with_core_source, with_selected_source, with_top_source};

#[derive(Resource)]
pub struct ExplorerState {
    pub inspected_unit: Option<u64>,
    pub inspected_house: Option<u64>,
    pub inspected_ability: Option<u64>,
    pub inspected_status: Option<u64>,

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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    Core,
    Top,
    Selected,
}

impl ViewMode {
    pub fn name(&self) -> &str {
        match self {
            ViewMode::Core => "Core",
            ViewMode::Top => "Top",
            ViewMode::Selected => "Selected",
        }
    }

    pub fn exec_ctx<R, F>(self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        match self {
            ViewMode::Core => with_core_source(f),
            ViewMode::Top => with_top_source(f),
            ViewMode::Selected => with_selected_source(f),
        }
    }
}
