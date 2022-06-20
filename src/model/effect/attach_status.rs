use crate::model::status::StatusAction;

use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Who {
    Caster,
    From,
    Target,
}

fn default_who() -> Who {
    Who::Target
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttachStatusEffect {
    #[serde(default = "default_who")]
    pub who: Who,
    pub status: StatusName,
}

impl EffectContainer for AttachStatusEffect {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for AttachStatusEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let status_name = &effect.status;
        let target = context.get(effect.who);
        if let Some(target) = target.and_then(|id| logic.model.units.get_mut(&id)) {
            // Check if unit is immune to status attachment
            if target
                .flags
                .iter()
                .any(|flag| matches!(flag, UnitStatFlag::AttachStatusImmune))
            {
                return;
            }

            if let Some(render) = &mut logic.render {
                render.add_text(
                    target.position,
                    &format!("{:?}", effect.status),
                    Color::BLUE,
                );
            }

            let status = logic.model.statuses.get_config(status_name);
            let status = status.status.clone().attach(
                Some(target.id),
                context.caster,
                &mut logic.model.next_id,
            );
            let attached_status_id = status.id;
            unit_attach_status(status, &mut target.all_statuses);
        }
    }
}
