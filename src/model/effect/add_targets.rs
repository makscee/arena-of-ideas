use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AddTargetsEffect {
    pub additional_targets: Option<usize>,
    #[serde(default)]
    pub condition: Condition,
    pub effect: Effect,
}

impl AddTargetsEffect {
    pub fn walk_children_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}
