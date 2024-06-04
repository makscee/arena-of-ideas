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
