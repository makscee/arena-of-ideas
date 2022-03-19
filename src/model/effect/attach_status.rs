use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Who {
    Caster,
    Target,
}

fn default_who() -> Who {
    Who::Target
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttachStatusEffect {
    #[serde(default = "default_who")]
    pub who: Who,
    pub status: AttachedStatus,
}

impl AttachStatusEffect {
    pub fn walk_children_mut(&mut self, _f: &mut impl FnMut(&mut Effect)) {}
}
