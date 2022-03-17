use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct HealEffect {
    pub hp: DamageValue,
}

impl HealEffect {
    pub fn walk_children_mut(&mut self, _f: &mut impl FnMut(&mut Effect)) {}
}
