use super::*;

pub trait ReactionImpl {
    fn react(&self, event: &Event, context: &Context) -> Option<&Actions>;
}

impl ReactionImpl for Reaction {
    fn react(&self, event: &Event, context: &Context) -> Option<&Actions> {
        for (trigger, actions) in self.trigger.iter() {
            if trigger.fire(event, context) {
                return Some(actions);
            }
        }
        None
    }
}
