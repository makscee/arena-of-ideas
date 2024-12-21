use super::*;

#[derive(Component, Debug, Default)]
pub struct NodeState {
    pub vars: HashMap<VarName, VarValue>,
    pub source: HashMap<VarName, NodeKind>,
}

impl NodeState {
    pub fn contains(&self, var: VarName) -> bool {
        self.vars.contains_key(&var)
    }
    pub fn insert(&mut self, source: NodeKind, var: VarName, value: VarValue) {
        self.vars.insert(var, value);
        self.source.insert(var, source);
    }
    pub fn get_var_state(var: VarName, entity: Entity, state: &StateQuery) -> Option<VarValue> {
        let v = state
            .get_state(entity)
            .and_then(|s| s.vars.get(&var).cloned());
        if v.is_some() {
            v
        } else {
            if let Some(p) = state.get_parent(entity) {
                Self::get_var_state(var, p, state)
            } else {
                None
            }
        }
    }
    pub fn get_var_world(var: VarName, entity: Entity, world: &World) -> Option<VarValue> {
        let v = world
            .get::<NodeState>(entity)
            .and_then(|s| s.vars.get(&var).cloned());
        if v.is_some() {
            v
        } else {
            if let Some(p) = get_parent(entity, world) {
                Self::get_var_world(var, p, world)
            } else {
                None
            }
        }
    }
}
