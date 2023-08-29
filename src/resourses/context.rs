use super::*;
use strum_macros::AsRefStr;

#[derive(Debug)]
pub struct Context {
    pub layers: Vec<ContextLayer>,
}

#[derive(Debug, Clone, AsRefStr)]
pub enum ContextLayer {
    Caster { entity: Entity },
    Target { entity: Entity },
}

impl ContextLayer {
    pub fn get_owner(&self) -> Option<Entity> {
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
}

impl Context {
    pub fn from_caster(entity: Entity) -> Self {
        Self {
            layers: vec![ContextLayer::Caster { entity }],
        }
    }

    pub fn set_target(mut self, target: Entity) -> Self {
        self.layers.push(ContextLayer::Target { entity: target });
        self
    }

    pub fn target(&self) -> Option<Entity> {
        let mut owner = None;
        for layer in self.layers.iter().rev() {
            owner = layer.get_target();
            if owner.is_some() {
                break;
            }
        }
        owner
    }

    pub fn owner(&self) -> Option<Entity> {
        let mut owner = None;
        for layer in self.layers.iter().rev() {
            owner = layer.get_owner();
            if owner.is_some() {
                break;
            }
        }
        owner
    }
}
