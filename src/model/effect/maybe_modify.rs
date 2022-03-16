use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub struct MaybeModifyEffect {
    pub base_effect: Effect,
    pub condition: Condition,
    pub modifier: Modifier,
}

impl MaybeModifyEffect {
    pub fn walk_children_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        self.base_effect.walk_mut(f);
    }
}
