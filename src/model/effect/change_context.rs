use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeContextEffect {
    #[serde(default)]
    pub caster: Option<Who>,
    #[serde(default)]
    pub from: Option<Who>,
    #[serde(default)]
    pub target: Option<Who>,
    pub effect: Effect,
}

impl ChangeContextEffect {
    pub fn walk_children_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}
