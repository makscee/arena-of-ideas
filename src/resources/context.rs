use super::*;

#[derive(Debug, Default, Clone)]
pub struct Context<'w, 's> {
    t: Option<f32>,
    layers: Vec<ContextLayer<'w>>,
    sources: Vec<ContextSource<'w, 's>>,
}

#[derive(Debug, Clone)]
pub enum ContextSource<'w, 's> {
    Query(&'w StateQuery<'w, 's>),
    World(&'w World),
    BattleSimulation(&'w BattleSimulation),
}

#[derive(Debug, Clone)]
enum ContextLayer<'w> {
    OwnerNode(&'w dyn GetVar),
    Owner(Entity),
    Caster(Entity),
    Target(Entity),
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
    pub fn new_battle_simulation(bs: &'w BattleSimulation) -> Self {
        Self {
            layers: default(),
            sources: vec![ContextSource::BattleSimulation(bs)],
            t: None,
        }
    }
    pub fn with_world(&self, world: &'w World) -> Self {
        self.clone().set_world(world).take()
    }
    pub fn set_world(&mut self, world: &'w World) -> &mut Self {
        self.sources.push(ContextSource::World(world));
        self
    }
    pub fn get_world(&self) -> Option<&World> {
        self.sources.iter().find_map(|s| s.get_world())
    }
    pub fn set_t(&mut self, t: f32) -> &mut Self {
        self.t = Some(t);
        self
    }
    pub fn get_t(&self) -> Option<f32> {
        self.t
    }
    pub fn set_owner(&mut self, owner: Entity) -> &mut Self {
        self.layers.push(ContextLayer::Owner(owner));
        self
    }
    pub fn set_caster(&mut self, owner: Entity) -> &mut Self {
        self.layers.push(ContextLayer::Caster(owner));
        self
    }
    pub fn add_target(&mut self, target: Entity) -> &mut Self {
        self.layers.push(ContextLayer::Target(target));
        self
    }
    pub fn set_var(&mut self, var: VarName, value: VarValue) -> &mut Self {
        self.layers.push(ContextLayer::Var(var, value));
        self
    }
    pub fn set_value(&mut self, value: VarValue) -> &mut Self {
        self.set_var(VarName::value, value)
    }
    pub fn set_owner_node(&mut self, node: &'w dyn GetVar) -> &mut Self {
        self.layers.push(ContextLayer::OwnerNode(node));
        self
    }

    pub fn get_owner(&self) -> Result<Entity, ExpressionError> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_owner())
            .to_e("Owner not found")
    }
    pub fn get_caster(&self) -> Result<Entity, ExpressionError> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_caster())
            .to_e("Caster not found")
    }
    pub fn get_target(&self) -> Result<Entity, ExpressionError> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_target())
            .to_e("Target not found")
    }
    pub fn collect_targets(&self) -> Result<Vec<Entity>, ExpressionError> {
        let targets = self
            .layers
            .iter()
            .filter_map(|l| l.get_target())
            .collect_vec();
        match targets.is_empty() {
            true => Err("No targets found".into()),
            false => Ok(targets),
        }
    }
    pub fn get_var(&self, var: VarName) -> Result<VarValue, ExpressionError> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_var(var, self))
            .to_e_var(var)
    }
    pub fn get_vars(&self, vars: impl Iterator<Item = VarName>) -> HashMap<VarName, VarValue> {
        HashMap::from_iter(vars.filter_map(|var| self.get_var(var).ok().map(|value| (var, value))))
    }
    pub fn get_state<'a>(&'a self, entity: Entity) -> Result<&'a NodeState, ExpressionError> {
        self.sources
            .iter()
            .find_map(|s| s.get_state(entity))
            .to_e("State not found")
    }
    pub fn get_value(&self) -> Result<VarValue, ExpressionError> {
        self.get_var(VarName::value)
    }
    pub fn get_bool(&self, var: VarName) -> Result<bool, ExpressionError> {
        self.get_var(var)?.get_bool()
    }
    pub fn get_i32(&self, var: VarName) -> Result<i32, ExpressionError> {
        self.get_var(var)?.get_i32()
    }
    pub fn get_f32(&self, var: VarName) -> Result<f32, ExpressionError> {
        self.get_var(var)?.get_f32()
    }
    pub fn get_string(&self, var: VarName) -> Result<String, ExpressionError> {
        self.get_var(var)?.get_string()
    }
    pub fn get_color(&self, var: VarName) -> Result<Color32, ExpressionError> {
        self.get_var(var)?.get_color()
    }
    pub fn get_vars_layers(&self) -> HashMap<VarName, VarValue> {
        HashMap::from_iter(self.layers.iter().filter_map(|l| match l {
            ContextLayer::Var(var, value) => Some((*var, value.clone())),
            _ => None,
        }))
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
    pub fn get_parent(&self, entity: Entity) -> Option<Entity> {
        self.sources.iter().rev().find_map(|s| s.get_parent(entity))
    }
    pub fn get_all_units(&self) -> Vec<VarValue> {
        self.sources
            .iter()
            .flat_map(|s| s.get_all_fusions())
            .map(|e| e.to_value())
            .collect()
    }
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        for s in self.sources.iter().rev() {
            if let Some(c) = s.get_component::<T>(entity) {
                return Some(c);
            }
        }
        None
    }
    pub fn find_parent_component<T: Component>(&self, mut entity: Entity) -> Option<&T> {
        while let Some(parent) = self.get_parent(entity) {
            if let Some(c) = self.get_component::<T>(parent) {
                return Some(c);
            }
            entity = parent;
        }
        None
    }
    pub fn children_components<T: Component>(&self, entity: Entity) -> Vec<&T> {
        self.sources
            .iter()
            .flat_map(|s| s.children_components::<T>(entity))
            .collect()
    }
    pub fn children_components_recursive<T: Component>(&self, entity: Entity) -> Vec<&T> {
        self.sources
            .iter()
            .flat_map(|s| s.children_components_recursive::<T>(entity))
            .collect()
    }
    pub fn all_allies(&self, entity: Entity) -> Vec<VarValue> {
        self.sources
            .iter()
            .flat_map(|s| s.collect_allies(entity))
            .map(|e| e.to_value())
            .collect()
    }
    pub fn all_enemies(&self, entity: Entity) -> Vec<VarValue> {
        self.sources
            .iter()
            .flat_map(|s| s.collect_enemies(entity))
            .map(|e| e.to_value())
            .collect()
    }
    pub fn offset_unit(&self, entity: Entity, offset: i32) -> Option<VarValue> {
        let entities = self
            .sources
            .iter()
            .flat_map(|s| s.collect_allies(entity))
            .collect_vec();
        if let Some(pos) = entities.iter().position(|e| *e == entity) {
            let i = pos as i32 + offset;
            if i >= 0 && (i as usize) < entities.len() {
                return Some(entities[i as usize].to_value());
            }
        }
        None
    }
    pub fn adjacent_allies(&self, entity: Entity) -> Vec<VarValue> {
        self.offset_unit(entity, 1)
            .into_iter()
            .chain(self.offset_unit(entity, -1))
            .collect()
    }

    pub fn clear(&mut self) {
        self.layers.clear();
    }
    pub fn take(&mut self) -> Self {
        mem::take(self)
    }
}

impl ContextSource<'_, '_> {
    pub fn get_world(&self) -> Option<&World> {
        match self {
            ContextSource::Query(..) => None,
            ContextSource::World(world) => Some(*world),
            ContextSource::BattleSimulation(bs) => Some(&bs.world),
        }
    }
    pub fn get_state(&self, entity: Entity) -> Option<&NodeState> {
        match self {
            ContextSource::Query(q) => NodeState::from_query(entity, q),
            ContextSource::World(w) => NodeState::from_world(entity, w),
            ContextSource::BattleSimulation(bs) => NodeState::from_world(entity, &bs.world),
        }
    }
    pub fn get_children(&self, entity: Entity) -> Vec<Entity> {
        match self {
            ContextSource::Query(q) => q.get_children(entity),
            ContextSource::World(w) => get_children(entity, w),
            ContextSource::BattleSimulation(bs) => get_children(entity, &bs.world),
        }
    }
    pub fn get_children_recursive(&self, entity: Entity) -> Vec<Entity> {
        match self {
            ContextSource::World(w) => get_children_recursive(entity, w),
            ContextSource::BattleSimulation(bs) => get_children_recursive(entity, &bs.world),
            ContextSource::Query(_) => todo!(),
        }
    }
    pub fn get_parent(&self, entity: Entity) -> Option<Entity> {
        match self {
            ContextSource::Query(q) => q.get_parent(entity),
            ContextSource::World(w) => get_parent(entity, w),
            ContextSource::BattleSimulation(bs) => get_parent(entity, &bs.world),
        }
    }
    fn get_all_fusions(&self) -> Vec<Entity> {
        match self {
            ContextSource::BattleSimulation(bs) => bs
                .fusions_left
                .iter()
                .chain(bs.fusions_right.iter())
                .copied()
                .collect(),
            _ => default(),
        }
    }
    fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        match self {
            ContextSource::World(world) => world.get::<T>(entity),
            ContextSource::BattleSimulation(bs) => bs.world.get::<T>(entity),
            _ => None,
        }
    }
    pub fn children_components<T: Component>(&self, entity: Entity) -> Vec<&T> {
        self.get_children(entity)
            .into_iter()
            .filter_map(|e| self.get_component(e))
            .collect()
    }
    pub fn children_components_recursive<T: Component>(&self, entity: Entity) -> Vec<&T> {
        self.get_children_recursive(entity)
            .into_iter()
            .filter_map(|e| self.get_component(e))
            .collect()
    }
    pub fn collect_enemies(&self, entity: Entity) -> Vec<Entity> {
        match self {
            ContextSource::Query(..) | ContextSource::World(..) => default(),
            ContextSource::BattleSimulation(bs) => {
                if bs.fusions_left.contains(&entity) {
                    bs.fusions_right.clone()
                } else if bs.fusions_right.contains(&entity) {
                    bs.fusions_left.clone()
                } else {
                    default()
                }
            }
        }
    }
    pub fn collect_allies(&self, entity: Entity) -> Vec<Entity> {
        match self {
            ContextSource::Query(..) | ContextSource::World(..) => default(),
            ContextSource::BattleSimulation(bs) => {
                if bs.fusions_left.contains(&entity) {
                    bs.fusions_left.clone()
                } else if bs.fusions_right.contains(&entity) {
                    bs.fusions_right.clone()
                } else {
                    default()
                }
            }
        }
    }
}

impl ContextLayer<'_> {
    fn get_owner(&self) -> Option<Entity> {
        match self {
            ContextLayer::Owner(entity) => Some(*entity),
            _ => None,
        }
    }
    fn get_caster(&self) -> Option<Entity> {
        match self {
            ContextLayer::Caster(entity) => Some(*entity),
            _ => None,
        }
    }
    fn get_target(&self) -> Option<Entity> {
        match self {
            ContextLayer::Target(entity) => Some(*entity),
            _ => None,
        }
    }
    fn get_var(&self, var: VarName, context: &Context) -> Option<VarValue> {
        match self {
            ContextLayer::Owner(entity) => NodeState::find_var(var, *entity, context),
            ContextLayer::Var(v, value) => {
                if var.eq(v) {
                    Some(value.clone())
                } else {
                    None
                }
            }
            ContextLayer::OwnerNode(node) => node.get_own_var(var),
            ContextLayer::Caster(..) => None,
            ContextLayer::Target(..) => None,
        }
    }
}
