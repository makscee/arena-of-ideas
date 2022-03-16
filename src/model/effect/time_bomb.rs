use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeBombEffect {
    pub time: Time,
    pub effect: Effect,
}

impl TimeBombEffect {
    pub fn walk_children_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}
