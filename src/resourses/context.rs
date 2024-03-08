use super::*;

/// All data that is needed to invoke an Effect
#[derive(Debug, Clone, Default)]
pub struct Context {
    layers: Vec<ContextLayer>,
}

#[derive(Debug, Clone, AsRefStr)]
enum ContextLayer {
    Caster(Entity, String),
    Target(Entity, String),
    Owner(Entity, String),
    Status(Entity, String),
    Var(VarName, VarValue),
    Text(String),
}

impl ContextLayer {
    pub fn get_owner(&self) -> Option<Entity> {
        match self {
            ContextLayer::Owner(entity, ..) => Some(*entity),
            _ => None,
        }
    }
    pub fn get_caster(&self) -> Option<Entity> {
        match self {
            ContextLayer::Caster(entity, ..) => Some(*entity),
            _ => None,
        }
    }
    pub fn get_target(&self) -> Option<Entity> {
        match self {
            ContextLayer::Target(entity, ..) => Some(*entity),
            _ => None,
        }
    }
    pub fn get_status(&self) -> Option<Entity> {
        match self {
            ContextLayer::Status(entity, ..) => Some(*entity),
            _ => None,
        }
    }
    pub fn get_var(&self, var: VarName, world: &World) -> Option<VarValue> {
        match self {
            ContextLayer::Owner(entity, ..) => {
                VarState::try_get(*entity, world).ok().and_then(|state| {
                    state.get_value_last(var).ok().map(|mut value| {
                        if let Some(children) = world.get::<Children>(*entity) {
                            let children = children.to_vec();
                            for child in children {
                                if let Some(delta) = world.get::<VarStateDelta>(child) {
                                    value = delta.process_last(var, value);
                                }
                            }
                        }
                        value
                    })
                })
            }
            ContextLayer::Var(v, value) => match var.eq(v) {
                true => Some(value.clone()),
                false => None,
            },
            ContextLayer::Status(entity, ..) => {
                VarState::get(*entity, world).get_value_last(var).ok()
            }
            _ => None,
        }
    }
}

impl Context {
    pub fn new_empty() -> Self {
        Self { ..default() }
    }

    pub fn new_named(name: String) -> Self {
        Self {
            layers: vec![ContextLayer::Text(name)],
        }
    }

    fn stack(&mut self, layer: ContextLayer, world: &World) -> &mut Self {
        match &layer {
            ContextLayer::Owner(entity, ..) => {
                let entity = *entity;
                if let Some(parent) = world.get::<Parent>(entity) {
                    let parent = parent.get();
                    if let Some(state) = world.get::<VarState>(parent) {
                        let name = state.get_string(VarName::Name).unwrap_or_default();
                        self.stack(ContextLayer::Owner(parent, name), world);
                    }
                }
                self.layers.push(layer);
            }
            _ => self.layers.push(layer),
        }
        self
    }

    pub fn get_all_vars(&self) -> HashMap<VarName, VarValue> {
        let mut result: HashMap<VarName, VarValue> = default();
        for layer in self.layers.iter().rev() {
            if let ContextLayer::Var(var, value) = layer {
                result.insert(*var, value.clone());
            }
        }
        result
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
    pub fn set_var(&mut self, var: VarName, value: VarValue) -> &mut Self {
        self.layers.push(ContextLayer::Var(var, value));
        self
    }

    pub fn from_owner(entity: Entity, world: &World) -> Self {
        Self::new_empty().set_owner(entity, world).take()
    }
    pub fn set_owner(&mut self, entity: Entity, world: &World) -> &mut Self {
        let name = VarState::try_get(entity, world)
            .and_then(|s| s.get_string(VarName::Name))
            .unwrap_or_default();
        self.stack(ContextLayer::Owner(entity, name), world)
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
        Self::new_empty().set_caster(entity, world).take()
    }
    pub fn set_caster(&mut self, entity: Entity, world: &World) -> &mut Self {
        let name = VarState::get(entity, world)
            .get_string(VarName::Name)
            .unwrap_or_default();
        self.stack(ContextLayer::Caster(entity, name), world)
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
        Self::new_empty().set_target(entity, world).take()
    }
    pub fn set_target(&mut self, entity: Entity, world: &World) -> &mut Self {
        let name = VarState::get(entity, world)
            .get_string(VarName::Name)
            .unwrap_or_default();
        self.stack(ContextLayer::Target(entity, name), world)
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
        let name = VarState::get(entity, world)
            .get_string(VarName::Name)
            .unwrap_or_default();
        self.stack(ContextLayer::Status(entity, name), world)
    }
    pub fn status(&self) -> Entity {
        self.get_status().expect("Target not found")
    }
    pub fn get_status(&self) -> Option<Entity> {
        let mut result = None;
        for layer in self.layers.iter().rev() {
            result = layer.get_status();
            if result.is_some() {
                break;
            }
        }
        result
    }
    pub fn has_status(&self, entity: Entity) -> bool {
        self.layers
            .iter()
            .any(|l| matches!(l, ContextLayer::Status(e, ..) if entity.eq(e)))
    }

    pub fn add_text(&mut self, text: String) -> &mut Self {
        self.layers.push(ContextLayer::Text(text));
        self
    }

    pub fn take(&mut self) -> Self {
        mem::take(self)
    }
}

impl std::fmt::Display for ContextLayer {
    fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
        let mut self_text = format!("{}:", self.as_ref()).bold();
        self_text = match self {
            ContextLayer::Caster(..) => self_text.green(),
            ContextLayer::Target(..) => self_text.red(),
            ContextLayer::Owner(..) => self_text.blue(),
            ContextLayer::Status(..) => self_text.bright_cyan(),
            ContextLayer::Var(..) => self_text.purple(),
            ContextLayer::Text(..) => self_text.white(),
        };
        match self {
            ContextLayer::Caster(entity, name)
            | ContextLayer::Target(entity, name)
            | ContextLayer::Owner(entity, name)
            | ContextLayer::Status(entity, name) => write!(f, "{self_text} {entity:?} {name}"),
            ContextLayer::Var(var, value) => write!(f, "{self_text} {var} -> {value:?}"),
            ContextLayer::Text(text) => write!(f, "{self_text} {text}"),
        }
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "o:{:?} t:{:?}\n>>>\n{}\n<<<",
            self.get_owner(),
            self.get_target(),
            self.layers.iter().map(|x| x.to_string()).join(" → "),
        )
    }
}
