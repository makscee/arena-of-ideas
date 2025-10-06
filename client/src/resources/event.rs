use super::*;

pub trait EventImpl {
    fn update_value(&self, context: &mut ClientContext, value: VarValue, owner: u64) -> VarValue;
}

impl EventImpl for Event {
    fn update_value(&self, ctx: &mut ClientContext, value: VarValue, owner: u64) -> VarValue {
        match ctx.with_temp_layers(
            [
                ContextLayer::Owner(owner),
                ContextLayer::Var(VarName::value, value.clone()),
            ]
            .into(),
            |context| {
                // if let Ok(fusion) = context.get::<NFusion>(entity) {
                //     fusion.react(event, context).log();
                // }
                for status in context
                    .load_collect_children_recursive::<NStatusMagic>(owner)?
                    .into_iter()
                    .cloned()
                    .collect_vec()
                {
                    let mut value = context.get_var(VarName::value)?;
                    if let Ok(behavior) =
                        context.load_first_child_recursive::<NStatusBehavior>(status.id)
                    {
                        context
                            .with_temp_owner(status.id, |context| {
                                if let Some(actions) = behavior.reactions.react(self, context) {
                                    match actions.process(context) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            return Err(e);
                                        }
                                    }
                                }
                                value = context.get_var(VarName::value)?;
                                Ok(())
                            })
                            .log();
                        context.set_var(VarName::value, value);
                    }
                }
                context.get_var(VarName::value)
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
