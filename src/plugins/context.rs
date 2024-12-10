use super::*;

#[derive(Debug)]
pub struct Context<'w, 's> {
    state: StateQuery<'w, 's>,
    layers: Vec<ContextLayer>,
}

#[derive(Debug)]
enum ContextLayer {
    Owner(Entity),
    Var(VarName, VarValue),
}

impl<'w, 's> Context<'w, 's> {
    pub fn new(state: StateQuery<'w, 's>) -> Self {
        Self {
            state,
            layers: default(),
        }
    }
    pub fn set_owner(&mut self, owner: Entity) {
        self.layers.push(ContextLayer::Owner(owner));
    }
    pub fn set_var(&mut self, var: VarName, value: VarValue) {
        self.layers.push(ContextLayer::Var(var, value));
    }

    pub fn get_owner(&self) -> Option<Entity> {
        self.layers.iter().rev().find_map(|l| l.get_owner())
    }
    pub fn get_var(&self, var: VarName) -> Option<VarValue> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_var(var, &self.state))
    }

    pub fn clear(&mut self) {
        self.layers.clear();
    }
}

impl ContextLayer {
    fn get_owner(&self) -> Option<Entity> {
        match self {
            ContextLayer::Owner(entity) => Some(*entity),
            _ => None,
        }
    }
    fn get_var(&self, var: VarName, state: &StateQuery) -> Option<VarValue> {
        match self {
            ContextLayer::Owner(entity) => NodeState::get_var_state(var, *entity, state),
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
