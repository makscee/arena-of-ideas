use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct CustomTriggerEffect {
    name: String,
    who: Option<Who>,
}

impl EffectContainer for CustomTriggerEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut LogicEffect)) {}
}

impl EffectImpl for CustomTriggerEffect {
    fn process(self: Box<Self>, context: LogicEffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let name = effect.name.clone();
        let target_id = effect.who.and_then(|who| Some(context.get_id(who)));
        // todo: reimplement
        // for unit in &logic.model.units {
        //     if target_id
        //         .and_then(|id| Some(id != unit.id))
        //         .or(Some(false))
        //         .unwrap()
        //     {
        //         continue;
        //     }
        //     for trigger_effect in unit.trigger().filter(|event| {
        //         let status_match = if let Some(status_id) = context.status_id {
        //             event.status_id == status_id
        //         } else {
        //             true
        //         };
        //         match &event.trigger {
        //             StatusTrigger::Custom { name } if status_match => name == &effect.name,
        //             _ => false,
        //         }
        //     }) {
        //         let mut vars = trigger_effect.vars.clone();
        //         vars.extend(context.vars.clone());
        //         logic.effects.push_back(
        //             LogicEffectContext {
        //                 creator: context.owner,
        //                 owner: unit.id,
        //                 vars,
        //                 color: trigger_effect.status_color,
        //                 ..context.clone()
        //             },
        //             trigger_effect.effect,
        //         )
        //     }
        // }
    }
}
