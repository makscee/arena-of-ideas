use super::*;

#[derive(Debug)]
pub enum ExplorerAction {
    SelectComponent {
        parent_id: String,
        component_id: String,
        kind: NodeKind,
    },
    InspectUnit(String),
    InspectHouse(String),
    InspectAbility(String),
    InspectStatus(String),
    SwitchViewMode(ViewMode),
}

#[derive(Default, Debug)]
pub enum ViewMode {
    #[default]
    Current,
    Selected,
}
