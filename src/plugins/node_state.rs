use super::*;

pub struct NodeStatePlugin;

impl Plugin for NodeStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, Self::inject_vars);
    }
}

#[derive(Component, Debug, Default)]
pub struct NodeState {
    pub vars: HashMap<VarName, HashMap<NodeKind, VarState>>,
}

#[derive(Debug)]
pub struct VarState {
    pub value: VarValue,
    pub history: VarHistory,
}
#[derive(Default, Debug)]
pub struct VarHistory {
    changes: Vec<VarChange>,
}

impl VarHistory {
    fn new(value: VarValue, t: f32, duration: f32, tween: Tween) -> Self {
        Self {
            changes: [VarChange {
                value,
                t,
                duration,
                tween,
            }]
            .into(),
        }
    }
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
                for (var, value) in v.get_own_vars() {
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
                let source = v.kind();
                for (var, value) in v.get_own_vars() {
                    state.insert(t, 0.0, var, value, source);
                }
            }
        }
    }
    pub fn collect_vars(
        In(entity): In<Entity>,
        nodes: Query<(&dyn GetVar, Option<&Parent>)>,
    ) -> HashMap<VarName, (VarValue, NodeKind)> {
        let mut entity = Some(entity);
        let mut result: HashMap<VarName, (VarValue, NodeKind)> = default();
        while let Some((gv, p)) = entity.and_then(|e| nodes.get(e).ok()) {
            for v in gv {
                let source = v.kind();
                for (var, value) in v.get_own_vars() {
                    if !result.contains_key(&var) {
                        result.insert(var, (value, source));
                    }
                }
            }
            entity = p.map(|p| p.get());
        }
        result
    }
}

impl NodeState {
    pub fn new_with(var: VarName, value: VarValue) -> Self {
        let mut state = Self::default();
        state.insert(0.0, 0.0, var, value, default());
        state
    }
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
    fn get_state_any<'a>(&'a self, var: VarName) -> Option<&'a VarState> {
        self.vars
            .get(&var)
            .and_then(|s| s.iter().sorted_by_key(|(k, _)| **k).next())
            .map(|(_, s)| s)
    }
    fn get_state<'a>(&'a self, var: VarName, kind: NodeKind) -> Option<&'a VarState> {
        self.vars.get(&var).and_then(|s| s.get(&kind))
    }
    pub fn get(&self, var: VarName, kind: NodeKind) -> Option<VarValue> {
        self.get_state(var, kind).map(|s| s.value.clone())
    }
    pub fn get_at(&self, t: f32, var: VarName, kind: NodeKind) -> Option<VarValue> {
        if let Some(c) = self.get_state(var, kind).map(|s| &s.history) {
            c.get_value_at(t).ok()
        } else {
            self.get(var, kind)
        }
    }
    pub fn get_any(&self, var: VarName) -> Option<VarValue> {
        self.get_state_any(var).map(|v| v.value.clone())
    }
    pub fn get_any_at(&self, t: f32, var: VarName) -> Option<VarValue> {
        if let Some(c) = self.get_state_any(var).map(|s| &s.history) {
            c.get_value_at(t).ok()
        } else {
            self.get_any(var)
        }
    }
    pub fn init(&mut self, var: VarName, value: VarValue) {
        self.insert(0.0, 0.0, var, value, default());
    }
    pub fn init_vars(&mut self, vars: Vec<(VarName, VarValue)>) {
        for (var, value) in vars {
            self.init(var, value);
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
        if self
            .vars
            .get(&var)
            .and_then(|v| v.get(&source))
            .is_some_and(|v| v.value == value)
        {
            return false;
        }
        if !self.vars.contains_key(&var) {
            self.vars.insert(var, default());
        }
        let kinds = self.vars.get_mut(&var).unwrap();
        let Some(state) = kinds.get_mut(&source) else {
            kinds.insert(
                source,
                VarState {
                    history: VarHistory::new(value.clone(), t, duration, Tween::QuartOut),
                    value,
                },
            );
            return true;
        };
        state.value = value.clone();
        state.history.changes.push(VarChange {
            value,
            t,
            duration,
            tween: Tween::QuartOut,
        });
        true
    }
    pub fn find_var(
        var: VarName,
        kind: Option<NodeKind>,
        entity: Entity,
        t: Option<f32>,
        source: &ContextSource,
    ) -> Option<VarValue> {
        let v = source.get_state(entity).and_then(|s| {
            if let Some(t) = t {
                if let Some(kind) = kind {
                    s.get_at(t, var, kind)
                } else {
                    s.get_any_at(t, var)
                }
            } else {
                if let Some(kind) = kind {
                    s.get(var, kind)
                } else {
                    s.get_any(var)
                }
            }
        });
        if v.is_some() {
            v
        } else {
            if let Some(p) = source.get_parent(entity) {
                Self::find_var(var, kind, p, t, source)
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
