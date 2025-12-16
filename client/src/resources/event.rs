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
                let statuses = ctx
                    .load_children_ref::<NStatusMagic>(owner)?
                    .into_iter()
                    .cloned()
                    .collect_vec();

                let mut value = ctx.get_var(VarName::value)?;

                for status in statuses {
                    let status_id = status.id;
                    if status.state.load_node(ctx)?.stax <= 0 {
                        continue;
                    }
                    let new_value = ctx.with_status(status_id, |ctx| {
                        if let Ok(behavior) = status.behavior.load_node(ctx) {
                            if behavior.trigger.fire(self, ctx).ok().unwrap_or(false) {
                                let x = match ctx.get_var(VarName::value)? {
                                    VarValue::i32(v) => v as i64,
                                    VarValue::f32(v) => v as i64,
                                    _ => 0,
                                };
                                use crate::plugins::rhai::RhaiScriptStatusExt;
                                if let Ok(actions) =
                                    behavior.effect.execute_status(status.clone(), x, ctx)
                                {
                                    for action in actions {
                                        use crate::plugins::rhai::ToBattleAction;
                                        if let Ok(ba) = action.to_battle_action(ctx, status_id) {
                                            battle_actions.push(ba);
                                        }
                                    }
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
