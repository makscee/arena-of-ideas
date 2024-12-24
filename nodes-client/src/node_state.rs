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
    pub fn from_world(entity: Entity, world: &World) -> Option<&Self> {
        world.get::<Self>(entity)
    }
    pub fn from_world_mut(entity: Entity, world: &mut World) -> Option<Mut<Self>> {
        world.get_mut::<Self>(entity)
    }
    pub fn from_query<'a>(entity: Entity, query: &'a StateQuery) -> Option<&'a Self> {
        query.get_state(entity)
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
    pub fn insert(&mut self, t: f32, var: VarName, value: VarValue, source: NodeKind) -> bool {
        let mut updated = false;
        if let Some(prev) = self.vars.insert(var, value.clone()) {
            if prev != value {
                updated = true;
            }
        } else {
            updated = true;
        }
        if updated {
            self.history
                .entry(var)
                .or_default()
                .changes
                .push((t, value));
        }
        self.source.insert(var, source);
        updated
    }
    pub fn find_var(
        var: VarName,
        entity: Entity,
        t: Option<f32>,
        source: &ContextSource,
    ) -> Option<VarValue> {
        let v = source.get_state(entity).and_then(|s| {
            if let Some(t) = t {
                s.get_at(t, var)
            } else {
                s.get(var)
            }
        });
        if v.is_some() {
            v
        } else {
            if let Some(p) = source.get_parent(entity) {
                Self::find_var(var, p, t, source)
            } else {
                None
            }
        }
    }
}
