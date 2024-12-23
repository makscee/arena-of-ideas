use super::*;

#[derive(Debug, Default, Clone)]
pub struct Context<'w, 's> {
    t: Option<f32>,
    layers: Vec<ContextLayer>,
    sources: Vec<ContextSource<'w, 's>>,
}

#[derive(Debug, Clone)]
pub enum ContextSource<'w, 's> {
    Query(&'w StateQuery<'w, 's>),
    World(&'w World),
}

#[derive(Debug, Clone)]
enum ContextLayer {
    Owner(Entity),
    Var(VarName, VarValue),
}

impl<'w, 's> Context<'w, 's> {
    pub fn new(state: &'w StateQuery<'w, 's>) -> Self {
        Self {
            layers: default(),
            sources: vec![ContextSource::Query(state)],
            t: None,
        }
    }
    pub fn new_world(world: &'w World) -> Self {
        Self {
            layers: default(),
            sources: vec![ContextSource::World(world)],
            t: None,
        }
    }
    pub fn set_t(&mut self, t: f32) -> &mut Self {
        self.t = Some(t);
        self
    }
    pub fn set_owner(&mut self, owner: Entity) -> &mut Self {
        self.layers.push(ContextLayer::Owner(owner));
        self
    }
    pub fn set_var(&mut self, var: VarName, value: VarValue) -> &mut Self {
        self.layers.push(ContextLayer::Var(var, value));
        self
    }

    pub fn get_owner(&self) -> Option<Entity> {
        self.layers.iter().rev().find_map(|l| l.get_owner())
    }
    pub fn get_var(&self, var: VarName) -> Result<VarValue, ExpressionError> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_var(var, &self.sources, self.t))
            .to_e(var)
    }
    pub fn get_children(&self, entity: Entity) -> Vec<Entity> {
        for s in self.sources.iter().rev() {
            let c = s.get_children(entity);
            if !c.is_empty() {
                return c;
            }
        }
        default()
    }
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        for s in self.sources.iter().rev() {
            if let Some(c) = s.get_component::<T>(entity) {
                return Some(c);
            }
        }
        None
    }

    pub fn clear(&mut self) {
        self.layers.clear();
    }
    pub fn take(&mut self) -> Self {
        mem::take(self)
    }
}

impl ContextSource<'_, '_> {
    pub fn get_state(&self, entity: Entity) -> Option<&NodeState> {
        match self {
            ContextSource::Query(q) => NodeState::from_query(entity, q),
            ContextSource::World(w) => NodeState::from_world(entity, w),
        }
    }
    pub fn get_children(&self, entity: Entity) -> Vec<Entity> {
        match self {
            ContextSource::Query(q) => q.get_children(entity),
            ContextSource::World(w) => get_children(entity, w),
        }
    }
    pub fn get_parent(&self, entity: Entity) -> Option<Entity> {
        match self {
            ContextSource::Query(q) => q.get_parent(entity),
            ContextSource::World(w) => get_parent(entity, w),
        }
    }
    fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        match self {
            ContextSource::World(world) => world.get::<T>(entity),
            ContextSource::Query(..) => None,
        }
    }
}

impl ContextLayer {
    fn get_owner(&self) -> Option<Entity> {
        match self {
            ContextLayer::Owner(entity) => Some(*entity),
            _ => None,
        }
    }
    fn get_var(
        &self,
        var: VarName,
        sources: &Vec<ContextSource>,
        t: Option<f32>,
    ) -> Option<VarValue> {
        match self {
            ContextLayer::Owner(entity) => sources
                .into_iter()
                .rev()
                .find_map(|s| NodeState::find_var(var, *entity, t, s)),
            ContextLayer::Var(v, value) => {
                if var.eq(v) {
                    Some(value.clone())
                } else {
                    None
                }
            }
        }
    }
}
