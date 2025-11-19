use super::*;

#[derive(Debug, Clone)]
pub enum ExplorerAction {
    InspectUnit(u64),
    InspectHouse(u64),
    SuggestNode(NamedNodeKind, String),
    UpvoteNode(u64),
    DownvoteNode(u64),
}

impl ExplorerAction {
    pub fn execute(&self, world: &mut World) -> NodeResult<()> {
        match self {
            ExplorerAction::InspectUnit(id) => {
                let mut state = world.resource_mut::<ExplorerState>();
                state.inspected_unit = Some(*id);
            }
            ExplorerAction::InspectHouse(id) => {
                let mut state = world.resource_mut::<ExplorerState>();
                state.inspected_house = Some(*id);
            }
            ExplorerAction::SuggestNode(kind, name) => {
                cn().reducers
                    .content_suggest_node(kind.to_string(), name.clone())
                    .notify_error_op();
            }
            ExplorerAction::UpvoteNode(node_id) => {
                cn().reducers
                    .content_upvote_node(*node_id)
                    .notify_error_op();
            }
            ExplorerAction::DownvoteNode(node_id) => {
                cn().reducers
                    .content_downvote_node(*node_id)
                    .notify_error_op();
            }
        }
        Ok(())
    }
}
