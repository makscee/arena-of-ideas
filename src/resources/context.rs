use super::*;

#[derive(Debug, Clone, Default)]
pub struct Context {
    layers: Vec<ContextLayer>,
}

impl Context {
    pub fn empty() -> Self {
        Self { ..default() }
    }
    pub fn new(owner: Entity) -> Self {
        Self {
            layers: [ContextLayer::Owner(owner)].into(),
        }
    }
    pub fn set_owner(&mut self, entity: Entity) -> &mut Self {
        self.layers.push(ContextLayer::Owner(entity));
        self
    }
    pub fn owner(&self) -> Entity {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_owner())
            .expect("Context always supposed to have an owner")
    }
    pub fn set_target(&mut self, entity: Entity) -> &mut Self {
        self.layers.push(ContextLayer::Target(entity));
        self
    }
    pub fn target(&self) -> Entity {
        self.get_target().expect("Target not found")
    }
    pub fn get_target(&self) -> Result<Entity> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_target())
            .with_context(|| format!("Failed to get target"))
    }
    pub fn set_caster(&mut self, entity: Entity) -> &mut Self {
        self.layers.push(ContextLayer::Caster(entity));
        self
    }
    pub fn caster(&self) -> Entity {
        self.get_caster().expect("Caster not found")
    }
    pub fn get_caster(&self) -> Result<Entity> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_caster())
            .with_context(|| format!("Failed to get caster"))
    }
    pub fn get_var(&self, var: VarName, world: &World) -> Result<VarValue> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_var(var, world))
            .with_context(|| format!("Failed to find var {var}"))
    }
    pub fn set_var(&mut self, var: VarName, value: VarValue) -> &mut Self {
        self.layers.push(ContextLayer::Var(var, value));
        self
    }
    pub fn set_status(&mut self, owner: Entity, name: String) -> &mut Self {
        self.layers.push(ContextLayer::Status(owner, name));
        self
    }
    pub fn status(&self) -> (Entity, String) {
        self.get_status().expect("Status not found")
    }
    pub fn get_status(&self) -> Result<(Entity, String)> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_status())
            .with_context(|| format!("Failed to get status"))
    }
    pub fn has_status(&self, entity: Entity) -> bool {
        self.layers
            .iter()
            .any(|l| matches!(l, ContextLayer::Status(e, ..) if entity.eq(e)))
    }
    pub fn set_event(&mut self, event: Event) -> &mut Self {
        self.layers.push(ContextLayer::Event(event));
        self
    }
    pub fn get_faction(&self, world: &World) -> Result<Faction> {
        self.get_var(VarName::Faction, world)?.get_faction()
    }

    pub fn take(&mut self) -> Self {
        mem::take(self)
    }
}

#[derive(Debug, Clone, AsRefStr)]
pub enum ContextLayer {
    Caster(Entity),
    Target(Entity),
    Owner(Entity),
    Status(Entity, String),
    Var(VarName, VarValue),
    Event(Event),
}

impl ContextLayer {
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
    fn get_status(&self) -> Option<(Entity, String)> {
        match self {
            ContextLayer::Status(entity, name) => Some((*entity, name.clone())),
            _ => None,
        }
    }
    fn get_var(&self, var: VarName, world: &World) -> Option<VarValue> {
        match self {
            ContextLayer::Owner(entity) => match VarState::try_get(*entity, world)
                .ok()
                .and_then(|state| state.get_value_last(var).ok())
            {
                Some(v) => Some(v),
                None => {
                    if let Some(entity) = entity.get_parent(world) {
                        ContextLayer::Owner(entity).get_var(var, world)
                    } else {
                        None
                    }
                }
            },
            ContextLayer::Var(v, value) => match var.eq(v) {
                true => Some(value.clone()),
                false => None,
            },
            ContextLayer::Status(owner, name) => VarState::get(*owner, world)
                .get_status(&name)?
                .get_value_last(var)
                .ok(),
            _ => None,
        }
    }
}
