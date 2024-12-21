use egui::NumExt;

use super::*;

#[derive(Component, Debug, Default)]
pub struct NodeState {
    vars: HashMap<VarName, VarValue>,
    source: HashMap<VarName, NodeKind>,
    pub history: HashMap<VarName, VarHistory>,
}
#[derive(Default, Debug)]
pub struct VarHistory {
    changes: Vec<(f32, VarValue)>,
}

impl NodeState {
    pub fn contains(&self, var: VarName) -> bool {
        self.vars.contains_key(&var)
    }
    pub fn get(&self, var: VarName) -> Option<VarValue> {
        self.vars.get(&var).cloned()
    }
    pub fn get_at(&self, t: f32, var: VarName) -> Option<VarValue> {
        if let Some(c) = self.history.get(&var) {
            let i = match c.changes.binary_search_by(|(ct, _)| ct.total_cmp(&t)) {
                Ok(i) | Err(i) => i.at_least(1) - 1,
            };
            Some(c.changes[i].1.clone())
        } else {
            self.vars.get(&var).cloned()
        }
    }
    pub fn insert(&mut self, t: f32, var: VarName, value: VarValue, source: NodeKind) {
        let mut update_history = false;
        if let Some(prev) = self.vars.insert(var, value.clone()) {
            if prev != value {
                update_history = true;
            }
        } else {
            update_history = true;
        }
        if update_history {
            self.history
                .entry(var)
                .or_default()
                .changes
                .push((t, value));
        }
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
