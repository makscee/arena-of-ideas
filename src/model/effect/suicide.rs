use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SuicideEffect {}

impl SuicideEffect {
    pub fn walk_children_mut(&mut self, _f: &mut impl FnMut(&mut Effect)) {}
}
