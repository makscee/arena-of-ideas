use super::*;

/// All data that is needed to invoke an Effect
#[derive(Debug, Clone, Default)]
pub struct Context {
    pub layers: Vec<ContextLayer>,
}

#[derive(Debug, Clone, AsRefStr)]
pub enum ContextLayer {
    Caster(Entity),
    Target(Entity),
    Owner(Entity),
    Status(Entity),
    Var(VarName, VarValue),
    Text(String),
}

impl ContextLayer {
    pub fn get_owner(&self) -> Option<Entity> {
        match self {
            ContextLayer::Owner(entity) => Some(*entity),
            _ => None,
        }
    }
    pub fn get_caster(&self) -> Option<Entity> {
        match self {
            ContextLayer::Caster(entity) => Some(*entity),
            _ => None,
        }
    }
    pub fn get_target(&self) -> Option<Entity> {
        match self {
            ContextLayer::Target(entity) => Some(*entity),
            _ => None,
        }
    }
    pub fn get_var(&self, var: VarName, world: &World) -> Option<VarValue> {
        match self {
            ContextLayer::Owner(entity) | ContextLayer::Status(entity) => {
                VarState::get(*entity, world)
                    .get_value_last(var)
                    .ok()
                    .and_then(|mut value| {
                        if let Some(children) = world.get::<Children>(*entity) {
                            let children = children.to_vec();
                            for child in children {
                                if let Some(delta) = world.get::<VarStateDelta>(child) {
                                    value = delta.process_last(var, value);
                                }
                            }
                        }
                        Some(value)
                    })
            }
            ContextLayer::Var(v, value) => match var.eq(v) {
                true => Some(value.clone()),
                false => None,
            },
            _ => None,
        }
    }
}

impl Context {
    pub fn new_empty() -> Self {
        Self { layers: default() }
    }

    pub fn new_named(name: String) -> Self {
        Self {
            layers: vec![ContextLayer::Text(name)],
        }
    }

    pub fn stack(&mut self, layer: ContextLayer, world: &World) -> &mut Self {
        match &layer {
            ContextLayer::Owner(entity) => {
                let entity = *entity;
                if let Some(parent) = world.get::<Parent>(entity) {
                    let parent = parent.get();
                    if world.get::<VarState>(parent).is_some() {
                        self.stack(ContextLayer::Owner(parent), world);
                    }
                }
                self.layers.push(layer);
            }
            _ => self.layers.push(layer),
        }
        self
    }

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
        self.layers.push(ContextLayer::Var(var, value));
        self
    }

    pub fn from_owner(entity: Entity, world: &World) -> Self {
        mem::take(Self::new_empty().stack(ContextLayer::Owner(entity), world))
    }

    pub fn set_owner(&mut self, entity: Entity, world: &World) -> &mut Self {
        self.stack(ContextLayer::Owner(entity), world)
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

    pub fn from_caster(entity: Entity, world: &World) -> Self {
        mem::take(Self::new_empty().stack(ContextLayer::Caster(entity), world))
    }

    pub fn set_caster(&mut self, entity: Entity, world: &World) -> &mut Self {
        self.stack(ContextLayer::Caster(entity), world)
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

    pub fn from_target(entity: Entity, world: &World) -> Self {
        mem::take(Self::new_empty().stack(ContextLayer::Target(entity), world))
    }

    pub fn set_target(&mut self, entity: Entity, world: &World) -> &mut Self {
        self.stack(ContextLayer::Target(entity), world)
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

    pub fn set_status(&mut self, entity: Entity, world: &World) -> &mut Self {
        self.stack(ContextLayer::Status(entity), world)
    }
}

impl std::fmt::Display for ContextLayer {
    fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
        let self_text = format!("{}:", self.as_ref()).bold();
        match self {
            ContextLayer::Caster(entity)
            | ContextLayer::Target(entity)
            | ContextLayer::Owner(entity)
            | ContextLayer::Status(entity) => write!(f, "{self_text} {entity:?}"),
            ContextLayer::Var(var, value) => write!(f, "{self_text} {var} -> {value:?}"),
            ContextLayer::Text(text) => write!(f, "{self_text} {text}"),
        }
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "o:{:?} t:{:?}\n>>>\n{}\n<<<\n",
            self.get_owner(),
            self.get_target(),
            self.layers
                .iter()
                .rev()
                .map(|x| x.to_string())
                .join("\n<- "),
        )
    }
}
