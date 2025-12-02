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
        for reaction in self.iter() {
            match reaction.trigger.fire(event, ctx) {
                Ok(fired) => {
                    if fired {
                        return Some(&reaction.effect.actions);
                    }
                }
                Err(e) => error!("trigger {} fire err: {e}", reaction.trigger),
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
