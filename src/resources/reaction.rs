use super::*;

pub trait BehaviorImpl {
    fn react(&self, event: &Event, context: &Context) -> Option<&Vec<Action>>;
}

impl BehaviorImpl for Vec<Reaction> {
    fn react(&self, event: &Event, context: &Context) -> Option<&Vec<Action>> {
        for Reaction { trigger, actions } in self.iter() {
            match trigger.fire(event, context) {
                Ok(fired) => {
                    if fired {
                        return Some(actions);
                    }
                }
                Err(e) => error!("trigger fire err: {e}"),
            }
        }
        None
    }
}
