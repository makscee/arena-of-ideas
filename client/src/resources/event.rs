use super::*;

pub trait EventImpl {
    fn update_value(&self, ctx: &mut ClientContext, value: VarValue, owner: u64) -> VarValue;
}

impl EventImpl for Event {
    fn update_value(&self, ctx: &mut ClientContext, value: VarValue, owner: u64) -> VarValue {
        match ctx.with_layers_ref(
            [
                ContextLayer::Owner(owner),
                ContextLayer::Var(VarName::value, value.clone()),
            ],
            |ctx| {
                if let Ok(actions) = ctx
                    .load::<NFusion>(owner)
                    .and_then(|f| f.react_actions(self, ctx))
                {
                    for (_, action) in actions {
                        match action.process(ctx) {
                            Ok(_) => {}
                            Err(e) => {
                                e.log();
                            }
                        }
                    }
                }
                // TODO: Implement load_collect_children_recursive or replace with new API
                let statuses = ctx
                    .load_collect_children::<NStatusMagic>(owner)?
                    .into_iter()
                    .collect_vec();

                let mut value = ctx.get_var(VarName::value)?;

                for status in statuses {
                    // Collect behavior data without borrowing ctx
                    let status_id = status.id;
                    let behavior_opt = status
                        .description_ref(ctx)
                        .and_then(|d| d.behavior_ref(ctx))
                        .ok()
                        .cloned(); // Clone the behavior to avoid borrowing issues

                    if let Some(behavior) = behavior_opt {
                        // TODO: Implement with_status_ref or replace with new API
                        let result = ctx.with_status(status_id, |inner_ctx| {
                            if let Some(actions) = behavior.reactions.react(self, inner_ctx) {
                                match actions.process(inner_ctx) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        return Err(e);
                                    }
                                }
                            }
                            inner_ctx.get_var(VarName::value)
                        });

                        if let Ok(new_value) = result {
                            value = new_value;
                        }
                    }
                }

                ctx.set_var_layer(VarName::value, value);
                ctx.get_var(VarName::value)
            },
        ) {
            Ok(value) => value,
            Err(e) => {
                error!("Update event {self} for {owner} failed: {e}");
                value
            }
        }
    }
}
