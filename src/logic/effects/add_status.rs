use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Who {
    Caster,
    Target,
}

fn default_who() -> Who {
    Who::Target
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddStatusEffect {
    #[serde(default = "default_who")]
    pub who: Who,
    pub status: Status,
}

impl AddStatusEffect {
    pub fn walk_children_mut(&mut self, _f: &mut impl FnMut(&mut Effect)) {}
}

impl Logic<'_> {
    pub fn process_add_status_effect(
        &mut self,
        QueuedEffect {
            caster,
            target,
            effect,
        }: QueuedEffect<AddStatusEffect>,
    ) {
        let target = match effect.who {
            Who::Caster => caster,
            Who::Target => target,
        };
        if let Some(target) = target.and_then(|id| self.model.units.get_mut(&id)) {
            if let Some(render) = &mut self.render {
                render.add_text(target.position, effect.status.name(), Color::BLUE);
            }
            target.attached_statuses.push(effect.status);
        }
    }
}
