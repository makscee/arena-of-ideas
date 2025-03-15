use super::*;

pub trait BehaviorImpl {
    fn react(&self, event: &Event, context: &Context) -> Option<&Actions>;
}

impl BehaviorImpl for Behavior {
    fn react(&self, event: &Event, context: &Context) -> Option<&Actions> {
        for Reaction { trigger, actions } in self.triggers.iter() {
            if trigger.fire(event, context) {
                return Some(actions);
            }
        }
        None
    }
}
