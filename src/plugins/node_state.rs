use super::*;

pub struct NodeStatePlugin;

impl Plugin for NodeStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, Self::inject_vars);
    }
}

#[derive(Component, Debug, Default)]
pub struct NodeState {
    vars: HashMap<VarName, VarValue>,
    source: HashMap<VarName, NodeKind>,
    pub history: HashMap<VarName, VarHistory>,
}
#[derive(Default, Debug)]
pub struct VarHistory {
    changes: Vec<VarChange>,
}

#[derive(Debug)]
struct VarChange {
    value: VarValue,
    t: f32,
    duration: f32,
    tween: Tween,
}

impl NodeStatePlugin {
    fn inject_vars(mut nodes: Query<(&mut NodeState, &dyn GetVar, &GlobalTransform)>) {
        let t = gt().play_head();
        for (mut state, gv, transform) in &mut nodes {
            state.insert(
                t,
                0.1,
                VarName::position,
                transform.translation().xy().into(),
                NodeKind::None,
            );
            for v in gv {
                let kind = v.kind();
                for (var, value) in v.get_vars() {
                    state.insert(t, 0.0, var, value, kind);
                }
            }
        }
    }
    pub fn inject_entity_vars(
        In((entity, t)): In<(Entity, f32)>,
        mut nodes: Query<(&dyn GetVar, &mut NodeState)>,
    ) {
        if let Ok((gv, mut state)) = nodes.get_mut(entity) {
            for v in gv {
                let kind = v.kind();
                for (var, value) in v.get_vars() {
                    state.insert(t, 0.0, var, value, kind);
                }
            }
        }
    }
    pub fn collect_full_state(
        In(entity): In<Entity>,
        nodes: Query<(&dyn GetVar, Option<&Parent>)>,
    ) -> NodeState {
        let mut state = NodeState::default();
        let mut entity = Some(entity);
        while let Some((gv, p)) = entity.and_then(|e| nodes.get(e).ok()) {
            for v in gv {
                for (var, value) in v.get_vars() {
                    if !state.contains(var) {
                        state.insert(0.0, 0.0, var, value, v.kind());
                    }
                }
            }
            entity = p.map(|p| p.get());
        }
        state
    }
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
            c.get_value_at(t).ok()
        } else {
            self.vars.get(&var).cloned()
        }
    }
    pub fn insert(
        &mut self,
        t: f32,
        duration: f32,
        var: VarName,
        value: VarValue,
        source: NodeKind,
    ) -> bool {
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
                .push(VarChange {
                    value,
                    t,
                    duration,
                    tween: Tween::QuartOut,
                });
            self.source.insert(var, source);
        }
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

impl VarHistory {
    fn get_value_at(&self, t: f32) -> Result<VarValue, ExpressionError> {
        if t < 0.0 {
            return Err(ExpressionError::Custom("Not born yet".into()));
        }
        if self.changes.is_empty() {
            return Err(ExpressionError::Custom("History is empty".into()));
        }
        let mut i = match self.changes.binary_search_by(|h| h.t.total_cmp(&t)) {
            Ok(v) | Err(v) => v.at_least(1) - 1,
        };
        while self.changes.get(i + 1).is_some_and(|h| h.t <= t) {
            i += 1;
        }
        let cur_change = &self.changes[i];
        let prev_change = if i > 0 {
            &self.changes[i - 1]
        } else {
            cur_change
        };
        let t = t - cur_change.t;
        cur_change.tween.f(
            &prev_change.value,
            &cur_change.value,
            t,
            cur_change.duration,
        )
    }
}
