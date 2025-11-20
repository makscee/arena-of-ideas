use super::*;

pub trait BehaviorImpl {
    fn react_actions(&self, event: &Event, ctx: &ClientContext) -> Option<&Vec<Action>>;
    fn react_battle_actions(
        &self,
        event: &Event,
        ctx: &mut ClientContext,
    ) -> NodeResult<Vec<BattleAction>>;
}

impl BehaviorImpl for Vec<Reaction> {
    fn react_actions(&self, event: &Event, ctx: &ClientContext) -> Option<&Vec<Action>> {
        for Reaction { trigger, actions } in self.iter() {
            match trigger.fire(event, ctx) {
                Ok(fired) => {
                    if fired {
                        return Some(actions);
                    }
                }
                Err(e) => error!("trigger {trigger} fire err: {e}"),
            }
        }
        None
    }

    fn react_battle_actions(
        &self,
        event: &Event,
        ctx: &mut ClientContext,
    ) -> NodeResult<Vec<BattleAction>> {
        if let Some(actions) = self.react_actions(event, ctx) {
            if !actions.is_empty() {
                let mut battle_actions: Vec<BattleAction> = default();
                for action in actions {
                    battle_actions.extend(action.process(ctx).track()?);
                }
                return Ok(battle_actions);
            }
        }
        Ok(default())
    }
}
