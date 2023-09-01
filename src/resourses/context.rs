use super::*;
use strum_macros::AsRefStr;

#[derive(Debug, Clone)]
pub struct Context {
    pub layers: Vec<ContextLayer>,
}

#[derive(Debug, Clone, AsRefStr)]
pub enum ContextLayer {
    Caster { entity: Entity },
    Target { entity: Entity },
    Owner { entity: Entity },
    Var { var: VarName, value: VarValue },
    Status { entity: Entity },
}

impl ContextLayer {
    pub fn get_owner(&self) -> Option<Entity> {
        match self {
            ContextLayer::Owner { entity } => Some(*entity),
            _ => None,
        }
    }
    pub fn get_caster(&self) -> Option<Entity> {
        match self {
            ContextLayer::Caster { entity } => Some(*entity),
            _ => None,
        }
    }
    pub fn get_target(&self) -> Option<Entity> {
        match self {
            ContextLayer::Target { entity } => Some(*entity),
            _ => None,
        }
    }
    pub fn get_var(&self, var: VarName, world: &World) -> Option<VarValue> {
        match self {
            ContextLayer::Owner { entity } | ContextLayer::Status { entity } => {
                VarState::get(*entity, world).get_value_last(var).ok()
            }
            ContextLayer::Var { var: v, value } => match var.eq(v) {
                true => Some(value.clone()),
                false => None,
            },
            _ => None,
        }
    }
}

impl Context {
    pub fn get_var(&self, var: VarName, world: &World) -> Option<VarValue> {
        let mut result = None;
        for layer in self.layers.iter().rev() {
            result = layer.get_var(var, world);
            if result.is_some() {
                break;
            }
        }
        result
    }

    pub fn set_var(mut self, var: VarName, value: VarValue) -> Self {
        self.layers.push(ContextLayer::Var { var, value });
        self
    }

    pub fn from_owner(entity: Entity) -> Self {
        Self {
            layers: vec![ContextLayer::Owner { entity }],
        }
    }

    pub fn set_owner(mut self, entity: Entity) -> Self {
        self.layers.push(ContextLayer::Owner { entity });
        self
    }

    pub fn owner(&self) -> Entity {
        self.get_owner().expect("Owner not found")
    }

    pub fn get_owner(&self) -> Option<Entity> {
        let mut result = None;
        for layer in self.layers.iter().rev() {
            result = layer.get_owner();
            if result.is_some() {
                break;
            }
        }
        result
    }

    pub fn from_caster(entity: Entity) -> Self {
        Self {
            layers: vec![ContextLayer::Caster { entity }],
        }
    }

    pub fn set_caster(mut self, entity: Entity) -> Self {
        self.layers.push(ContextLayer::Caster { entity });
        self
    }

    pub fn caster(&self) -> Entity {
        self.get_caster().expect("Caster not found")
    }

    pub fn get_caster(&self) -> Option<Entity> {
        let mut result = None;
        for layer in self.layers.iter().rev() {
            result = layer.get_caster();
            if result.is_some() {
                break;
            }
        }
        result
    }

    pub fn from_target(entity: Entity) -> Self {
        Self {
            layers: vec![ContextLayer::Target { entity }],
        }
    }

    pub fn set_target(mut self, entity: Entity) -> Self {
        self.layers.push(ContextLayer::Target { entity });
        self
    }

    pub fn target(&self) -> Entity {
        self.get_target().expect("Target not found")
    }

    pub fn get_target(&self) -> Option<Entity> {
        let mut result = None;
        for layer in self.layers.iter().rev() {
            result = layer.get_target();
            if result.is_some() {
                break;
            }
        }
        result
    }

    pub fn set_status(mut self, entity: Entity) -> Self {
        self.layers.push(ContextLayer::Status { entity });
        self
    }
}
