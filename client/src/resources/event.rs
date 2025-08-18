use super::*;

pub trait EventImpl {
    fn update_value(&self, context: &mut Context, value: VarValue, owner: Entity) -> VarValue;
}

impl EventImpl for Event {
    fn update_value(&self, context: &mut Context, value: VarValue, owner: Entity) -> VarValue {
        match context.with_layers_r(
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
                    .collect_children_components_recursive::<NStatusAbility>(context.id(owner)?)?
                    .into_iter()
                    .cloned()
                    .collect_vec()
                {
                    let mut value = context.get_value()?;
                    if let Ok(behavior) =
                        context.first_parent_recursive::<NStatusBehavior>(status.id)
                    {
                        context
                            .with_layer_ref_r(ContextLayer::Owner(status.entity()), |context| {
                                if let Some(actions) = behavior.reactions.react(self, context) {
                                    match actions.process(context) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            return Err(e);
                                        }
                                    }
                                }
                                value = context.get_value()?;
                                Ok(())
                            })
                            .log();
                        context.set_value_var(value);
                    }
                }
                context.get_value()
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
