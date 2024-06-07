use super::*;

#[derive(Debug, Clone, Default)]
pub struct Context {
    layers: Vec<ContextLayer>,
}

impl Context {
    pub fn new(owner: Entity) -> Self {
        Self {
            layers: [ContextLayer::Owner(owner)].into(),
        }
    }
    fn stack(&mut self, layer: ContextLayer, world: &World) -> &mut Self {
        match &layer {
            ContextLayer::Owner(entity) => {
                let entity = *entity;
                if let Some(parent) = world.get::<Parent>(entity) {
                    let parent = parent.get();
                    self.stack(ContextLayer::Owner(parent), world);
                }
                self.layers.push(layer);
            }
            _ => self.layers.push(layer),
        }
        self
    }
    pub fn owner(&self) -> Entity {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_owner())
            .expect("Context always supposed to have an owner")
    }
    pub fn set_target(&mut self, entity: Entity, world: &World) -> &mut Self {
        self.stack(ContextLayer::Target(entity), world)
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
    pub fn set_caster(&mut self, entity: Entity, world: &World) -> &mut Self {
        self.stack(ContextLayer::Caster(entity), world)
    }
    pub fn caster(&self) -> Entity {
        self.get_caster().expect("Target not found")
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

    pub fn take(&mut self) -> Self {
        mem::take(self)
    }
}

#[derive(Debug, Clone, AsRefStr)]
pub enum ContextLayer {
    Caster(Entity),
    Target(Entity),
    Owner(Entity),
    Status(Entity),
    Var(VarName, VarValue),
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
    fn get_status(&self) -> Option<Entity> {
        match self {
            ContextLayer::Status(entity) => Some(*entity),
            _ => None,
        }
    }
    fn get_var(&self, var: VarName, world: &World) -> Option<VarValue> {
        match self {
            ContextLayer::Owner(entity, ..) => VarState::try_get(*entity, world)
                .ok()
                .and_then(|state| state.get_value_last(var).ok()),
            ContextLayer::Var(v, value) => match var.eq(v) {
                true => Some(value.clone()),
                false => None,
            },
            _ => None,
        }
    }
}
