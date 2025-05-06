use super::*;

pub trait BehaviorImpl {
    fn react(&self, event: &Event, context: &Context) -> Option<&Vec<Action>>;
}

impl BehaviorImpl for NBehavior {
    fn react(&self, event: &Event, context: &Context) -> Option<&Vec<Action>> {
        for Reaction { trigger, actions } in self.reactions.iter() {
            if trigger.fire(event, context) {
                return Some(actions);
            }
        }
        None
    }
}
