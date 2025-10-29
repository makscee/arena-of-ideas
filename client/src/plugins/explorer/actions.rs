use super::*;

#[derive(Debug)]
pub enum ExplorerAction {
    SelectComponent {
        parent_id: String,
        component_id: String,
        kind: NodeKind,
    },
    InspectUnit(u64),
    InspectHouse(u64),
    InspectAbility(u64),
    InspectStatus(u64),
    SwitchViewMode(ViewMode),
}
