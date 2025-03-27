mod nodes;

use super::*;

pub use nodes::*;

#[derive(SystemParam, Debug)]
pub struct StateQuery<'w, 's> {
    states: Query<
        'w,
        's,
        (
            Entity,
            &'static NodeState,
            Option<&'static Parent>,
            Option<&'static Children>,
        ),
    >,
}

impl<'w, 's> StateQuery<'w, 's> {
    pub fn get_state(&self, entity: Entity) -> Option<&NodeState> {
        self.states.get(entity).map(|(_, s, _, _)| s).ok()
    }
    pub fn get_parent(&self, entity: Entity) -> Option<Entity> {
        self.states
            .get(entity)
            .ok()
            .and_then(|(_, _, p, _)| p.map(|p| p.get()))
    }
    pub fn get_children(&self, entity: Entity) -> Vec<Entity> {
        self.states
            .get(entity)
            .ok()
            .and_then(|(_, _, _, c)| c.map(|c| c.to_vec()))
            .unwrap_or_default()
    }
}

pub trait EntityConverter {
    fn to_e(&self) -> Entity;
}

impl EntityConverter for u64 {
    fn to_e(&self) -> Entity {
        Entity::from_bits(*self)
    }
}

impl From<&String> for TNode {
    fn from(value: &String) -> Self {
        ron::from_str::<SerdeWrapper<TNode>>(value).unwrap().0
    }
}
impl Into<String> for TNode {
    fn into(self) -> String {
        self.to_ron()
    }
}
