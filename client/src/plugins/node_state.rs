use super::*;

pub struct NodeStatePlugin;

impl Plugin for NodeStatePlugin {
    fn build(&self, _app: &mut App) {
        // app.add_systems(PreUpdate, Self::inject_vars);
    }
}

#[derive(BevyComponent, Debug, Default)]
pub struct NodeState {
    pub vars: HashMap<VarName, VarState>,
    pub kind: NodeKind,
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

impl NodeStatePlugin {}

impl NodeState {
    pub fn new_with(var: VarName, value: VarValue) -> Self {
        let mut state = Self::default();
        state.insert(0.0, 0.0, var, value);
        state
    }
    pub fn load<'a>(entity: Entity, ctx: &'a ClientContext) -> Result<&'a Self, NodeError> {
        ctx.world()
            .to_e_not_found()?
            .get::<Self>(entity)
            .to_e_not_found()
    }
    pub fn contains(&self, var: VarName) -> bool {
        self.vars.contains_key(&var)
    }
    fn get_state<'a>(&'a self, var: VarName) -> Option<&'a VarState> {
        self.vars.get(&var)
    }
    pub fn get(&self, var: VarName) -> Option<VarValue> {
        self.get_state(var).map(|s| s.value.clone())
    }
    pub fn get_at(&self, t: f32, var: VarName) -> Option<VarValue> {
        if let Some(c) = self.get_state(var).map(|s| &s.history) {
            c.get_value_at(t).ok()
        } else {
            self.get(var)
        }
    }
    pub fn init(&mut self, var: VarName, value: VarValue) {
        self.insert(0.0, 0.0, var, value);
    }
    pub fn init_vars(&mut self, vars: Vec<(VarName, VarValue)>) {
        for (var, value) in vars {
            self.init(var, value);
        }
    }
    pub fn insert(&mut self, t: f32, duration: f32, var: VarName, value: VarValue) -> bool {
        if let Some(state) = self.vars.get_mut(&var) {
            if state.value == value {
                return false;
            }
            state.value = value.clone();
            state.history.changes.push(VarChange {
                value,
                t,
                duration,
                tween: Tween::QuartOut,
            });
        } else {
            self.vars.insert(
                var,
                VarState {
                    history: VarHistory::new(value.clone(), t, duration, Tween::QuartOut),
                    value,
                },
            );
        }
        true
    }
    pub fn get_var(context: &ClientContext, var: VarName, entity: Entity) -> Option<VarValue> {
        if let Ok(ns) = context.component::<NodeState>(entity) {
            if let Some(t) = context.t {
                ns.get_at(t, var)
            } else {
                ns.get(var)
            }
        } else {
            None
        }
    }
    pub fn find_var(context: &ClientContext, var: VarName, entity: Entity) -> Option<VarValue> {
        let mut checked: HashSet<Entity> = default();
        let mut q = VecDeque::from([entity]);
        while let Some(entity) = q.pop_front() {
            let v = Self::get_var(context, var, entity);
            if v.is_some() {
                return v;
            }
            let Some(id) = context.id(entity).ok_log() else {
                continue;
            };
            let Some(parents) = context.ids_to_entities(context.parents(id)).ok_log() else {
                continue;
            };
            for parent in parents {
                if checked.insert(parent) {
                    q.push_back(parent);
                }
            }
        }
        None
    }
    pub fn sum_var(
        context: &ClientContext,
        var: VarName,
        entity: Entity,
    ) -> Result<VarValue, NodeError> {
        let mut result = Self::get_var(context, var, entity).unwrap_or_default();
        let ids = context.parents_recursive(context.id(entity)?);
        for entity in context.ids_to_entities(ids)? {
            if let Some(v) = Self::get_var(context, var, entity) {
                result = result.add(&v)?;
            }
        }
        Ok(result)
    }
}

impl VarHistory {
    fn get_value_at(&self, t: f32) -> Result<VarValue, NodeError> {
        if t < 0.0 {
            return Err(NodeError::Custom("Not born yet".into()).into());
        }
        if self.changes.is_empty() {
            return Err(NodeError::Custom("History is empty".into()).into());
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
