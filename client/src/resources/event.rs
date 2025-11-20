use super::*;

pub trait EventImpl {
    fn update_value(
        &self,
        ctx: &mut ClientContext,
        value: VarValue,
        owner: u64,
    ) -> (VarValue, Vec<BattleAction>);
}

impl EventImpl for Event {
    fn update_value(
        &self,
        ctx: &mut ClientContext,
        value: VarValue,
        owner: u64,
    ) -> (VarValue, Vec<BattleAction>) {
        let mut battle_actions: Vec<BattleAction> = Vec::new();
        match ctx.with_layers(
            [
                ContextLayer::Owner(owner),
                ContextLayer::Var(VarName::value, value.clone()),
            ],
            |ctx| {
                if let Some(actions) = ctx
                    .load::<NUnitBehavior>(owner)
                    .ok()
                    .and_then(|ub| ub.reactions.react_actions(self, ctx).cloned())
                {
                    for action in actions {
                        match action.process(ctx) {
                            Ok(actions) => {
                                battle_actions.extend(actions);
                            }
                            Err(e) => {
                                e.log();
                            }
                        }
                    }
                }
                let statuses = ctx
                    .load_children_ref::<NStatusMagic>(owner)?
                    .into_iter()
                    .cloned()
                    .collect_vec();

                let mut value = ctx.get_var(VarName::value)?;

                for status in statuses {
                    let status_id = status.id;
                    if status.state_ref(ctx)?.stax <= 0 {
                        continue;
                    }
                    let new_value = ctx.with_status(status_id, |ctx| {
                        if let Some(actions) = status
                            .behavior_ref(ctx)?
                            .reactions
                            .react_actions(self, ctx)
                            .cloned()
                        {
                            match actions.process(ctx) {
                                Ok(actions) => {
                                    battle_actions.extend(actions);
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            }
                        }
                        ctx.get_var(VarName::value)
                    });
                    if let Ok(new_value) = new_value {
                        value = new_value;
                    }
                }

                ctx.set_var_layer(VarName::value, value);
                ctx.get_var(VarName::value)
            },
        ) {
            Ok(value) => (value, battle_actions),
            Err(e) => {
                error!("Update event {self} for {owner} failed: {e}");
                (value, battle_actions)
            }
        }
    }
}
