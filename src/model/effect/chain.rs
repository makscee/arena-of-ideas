use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ChainEffect {
    pub targets: usize,
    pub jump_distance: Coord,
    pub effect: Effect,
    pub jump_modifier: Modifier,
}

impl ChainEffect {
    pub fn walk_children_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}
