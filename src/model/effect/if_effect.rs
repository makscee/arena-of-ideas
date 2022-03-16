use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub struct IfEffect {
    pub condition: Condition,
    pub then: Effect,
    pub r#else: Effect,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Condition {
    UnitHasStatus { who: Who, status: Status },
}

impl IfEffect {
    pub fn walk_children_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        self.then.walk_mut(f);
        self.r#else.walk_mut(f);
    }
}
