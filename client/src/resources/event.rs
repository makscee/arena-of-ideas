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
            ]
            .into(),
            |ctx| {
                if let Ok(actions) = ctx
                    .load::<NFusion>(owner)
                    .cloned()
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
                for status in ctx
                    .load_collect_children_recursive::<NStatusMagic>(owner)?
                    .into_iter()
                    .cloned()
                    .collect_vec()
                {
                    let mut value = ctx.get_var(VarName::value)?;
                    if let Ok(behavior) = status
                        .description_ref(ctx)
                        .and_then(|d| d.behavior_ref(ctx))
                    {
                        ctx.with_status_ref(status.id, |ctx| {
                            if let Some(actions) = behavior.reactions.react(self, ctx) {
                                match actions.process(ctx) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        return Err(e);
                                    }
                                }
                            }
                            value = ctx.get_var(VarName::value)?;
                            Ok(())
                        })
                        .log();
                        ctx.set_var_layer(VarName::value, value);
                    }
                }
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
