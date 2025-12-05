use super::*;

#[derive(Debug, Clone)]
pub enum IncubatorAction {
    InspectUnit(u64),
    InspectHouse(u64),
    UpvoteNode(u64),
    DownvoteNode(u64),
}

impl IncubatorAction {
    pub fn execute(&self, world: &mut World) -> NodeResult<()> {
        match self {
            IncubatorAction::InspectUnit(id) => {
                let mut state = world.resource_mut::<IncubatorState>();
                state.inspected_unit = Some(*id);
            }
            IncubatorAction::InspectHouse(id) => {
                let mut state = world.resource_mut::<IncubatorState>();
                state.inspected_house = Some(*id);
            }
            IncubatorAction::UpvoteNode(node_id) => {
                cn().reducers
                    .content_upvote_node(*node_id)
                    .notify_error_op();
            }
            IncubatorAction::DownvoteNode(node_id) => {
                cn().reducers
                    .content_downvote_node(*node_id)
                    .notify_error_op();
            }
        }
        Ok(())
    }
}
