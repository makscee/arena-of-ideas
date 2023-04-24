use colored::Colorize;
use geng::prelude::itertools::Itertools;
use strum_macros::AsRefStr;

use super::*;

#[derive(Debug)]
pub struct Context {
    layers: Vec<ContextLayer>,
}

#[derive(Debug, Clone, AsRefStr)]
pub enum ContextLayer {
    Entity {
        entity: legion::Entity,
    },
    Unit {
        entity: legion::Entity,
    },
    Target {
        entity: legion::Entity,
    },
    Attacker {
        entity: legion::Entity,
    },
    Ability {
        ability: AbilityName,
    },
    Status {
        entity: legion::Entity,
        name: String,
        charges: i32,
    },
    Empty {
        name: String,
    },
    Vars {
        vars: Vars,
    },
    Var {
        var: VarName,
        value: Var,
    },
}

impl ContextLayer {
    pub fn get_var(&self, var: &VarName, world: &legion::World) -> Option<Var> {
        match self {
            ContextLayer::Entity { entity } => {
                ContextState::get(*entity, world).vars.try_get(var).cloned()
            }
            ContextLayer::Status {
                name,
                entity: _,
                charges,
            } => match var {
                VarName::Charges => Some(Var::Int(*charges)),
                VarName::StatusName => Some(Var::String((1, name.to_owned()))),
                _ => None,
            },
            ContextLayer::Vars { vars } => vars.try_get(var).cloned(),
            ContextLayer::Var { var: name, value } => match var == name {
                true => Some(value.clone()),
                false => None,
            },
            ContextLayer::Unit { .. }
            | ContextLayer::Ability { .. }
            | ContextLayer::Empty { .. }
            | ContextLayer::Target { .. }
            | ContextLayer::Attacker { .. } => None,
        }
    }

    pub fn get_status_charges(&self, name: &str, world: &legion::World) -> i32 {
        match self {
            ContextLayer::Entity { entity } => {
                let mut charges = 0;
                if let Ok(entry) = world.entry_ref(*entity) {
                    if let Ok(state) = entry.get_component::<ContextState>() {
                        charges += *state.statuses.get(name).unwrap_or(&0);
                        if let Some(parent) = state.parent {
                            charges +=
                                Self::Entity { entity: parent }.get_status_charges(name, world);
                        }
                    }
                }
                charges
            }
            _ => 0,
        }
    }

    pub fn extend_statuses(&self, statuses: &mut HashMap<String, i32>, world: &legion::World) {
        match self {
            ContextLayer::Entity { entity } => {
                if let Ok(entry) = world.entry_ref(*entity) {
                    if let Ok(state) = entry.get_component::<ContextState>() {
                        for (name, charges) in state.statuses.iter() {
                            *statuses.entry(name.to_owned()).or_default() += *charges;
                        }
                        if let Some(parent) = state.parent {
                            Self::Entity { entity: parent }.extend_statuses(statuses, world);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn extend_ability_vars(
        &self,
        ability: &AbilityName,
        vars: &mut Vars,
        world: &legion::World,
    ) {
        match self {
            ContextLayer::Entity { entity } => {
                if let Ok(entry) = world.entry_ref(*entity) {
                    if let Ok(state) = entry.get_component::<ContextState>() {
                        if let Some(ability_vars) = &state.ability_vars.get(ability) {
                            vars.merge_mut(ability_vars, false);
                        }
                        if let Some(parent) = state.parent {
                            Self::Entity { entity: parent }
                                .extend_ability_vars(ability, vars, world);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn get_target(&self) -> Option<legion::Entity> {
        match self {
            ContextLayer::Target { entity } => Some(*entity),
            _ => None,
        }
    }

    pub fn get_attacker(&self) -> Option<legion::Entity> {
        match self {
            ContextLayer::Attacker { entity } => Some(*entity),
            _ => None,
        }
    }

    pub fn get_owner(&self) -> Option<legion::Entity> {
        match self {
            ContextLayer::Unit { entity } => Some(*entity),
            _ => None,
        }
    }
}

impl Context {
    pub fn new(layer: ContextLayer, world: &legion::World, resources: &Resources) -> Self {
        let mut context = Self { layers: default() };
        context.stack(layer, world, resources);
        context
    }

    pub fn new_empty(text: &str) -> Self {
        let mut context = Self { layers: default() };
        context.stack_string(text);
        context
    }

    pub fn clone_stack(
        &self,
        layer: ContextLayer,
        world: &legion::World,
        resources: &Resources,
    ) -> Context {
        let mut context = Context {
            layers: self.layers.clone(),
        };
        context.stack(layer, world, resources);
        context
    }

    pub fn clone_stack_string(&self, text: &str) -> Context {
        let mut context = Context {
            layers: self.layers.clone(),
        };
        context.layers.push(ContextLayer::Empty {
            name: text.to_owned(),
        });
        context
    }

    pub fn stack_string(&mut self, text: &str) -> &mut Self {
        self.layers.push(ContextLayer::Empty {
            name: text.to_owned(),
        });
        self
    }

    pub fn stack(
        &mut self,
        layer: ContextLayer,
        world: &legion::World,
        resources: &Resources,
    ) -> &mut Self {
        match &layer {
            ContextLayer::Unit { entity } => {
                self.stack(ContextLayer::Entity { entity: *entity }, world, resources);
                self.layers.push(layer);
                self.layers.push(ContextLayer::Var {
                    var: VarName::HpOriginalValue,
                    value: self.get_var(&VarName::HpValue, world).unwrap(),
                });
                self.layers.push(ContextLayer::Var {
                    var: VarName::AttackOriginalValue,
                    value: self.get_var(&VarName::AttackValue, world).unwrap(),
                });
                Event::ModifyContext.calculate(self, world, resources);
            }
            ContextLayer::Entity { entity } => {
                if let Ok(entry) = world.entry_ref(*entity) {
                    if let Ok(state) = entry.get_component::<ContextState>() {
                        if let Some(parent) = state.parent {
                            self.stack(ContextLayer::Entity { entity: parent }, world, resources);
                        }
                    }
                }
                self.layers.push(layer);
            }
            ContextLayer::Ability { ability } => {
                let mut vars: Vars = default();
                for layer in self.layers.iter().rev() {
                    layer.extend_ability_vars(ability, &mut vars, world);
                }
                self.layers.push(ContextLayer::Vars { vars })
            }
            ContextLayer::Status { .. }
            | ContextLayer::Target { .. }
            | ContextLayer::Attacker { .. }
            | ContextLayer::Empty { .. }
            | ContextLayer::Vars { .. }
            | ContextLayer::Var { .. } => self.layers.push(layer),
        }
        self
    }

    pub fn set_target(mut self, target: legion::Entity) -> Self {
        self.layers.push(ContextLayer::Target { entity: target });
        self
    }

    pub fn set_target_ref(&mut self, target: legion::Entity) -> &mut Self {
        self.layers.push(ContextLayer::Target { entity: target });
        self
    }

    pub fn target(&self) -> Option<legion::Entity> {
        let mut target = None;
        for layer in self.layers.iter().rev() {
            target = layer.get_target();
            if target.is_some() {
                break;
            }
        }
        target
    }

    pub fn set_attacker(mut self, attacker: legion::Entity) -> Self {
        self.layers
            .push(ContextLayer::Attacker { entity: attacker });
        self
    }

    pub fn set_attacker_ref(&mut self, attacker: legion::Entity) -> &mut Self {
        self.layers
            .push(ContextLayer::Attacker { entity: attacker });
        self
    }

    pub fn attacker(&self) -> Option<legion::Entity> {
        let mut attacker = None;
        for layer in self.layers.iter().rev() {
            attacker = layer.get_attacker();
            if attacker.is_some() {
                break;
            }
        }
        attacker
    }

    pub fn owner(&self) -> Option<legion::Entity> {
        let mut owner = None;
        for layer in self.layers.iter().rev() {
            owner = layer.get_owner();
            if owner.is_some() {
                break;
            }
        }
        owner
    }

    pub fn insert_var(&mut self, var: VarName, value: Var) -> &mut Self {
        self.layers.push(ContextLayer::Var { var, value });
        self
    }

    pub fn get_var(&self, var: &VarName, world: &legion::World) -> Option<Var> {
        let mut result = None;
        for layer in self.layers.iter().rev() {
            result = layer.get_var(var, world);
            if result.is_some() {
                break;
            }
        }
        result
    }

    pub fn insert_int(&mut self, var: VarName, value: i32) -> &mut Self {
        self.insert_var(var, Var::Int(value))
    }

    pub fn get_int(&self, var: &VarName, world: &legion::World) -> Option<i32> {
        match self.get_var(var, world) {
            Some(value) => match value {
                Var::Int(value) => Some(value),
                _ => None,
            },
            None => None,
        }
    }

    pub fn insert_vec2(&mut self, var: VarName, value: vec2<f32>) -> &mut Self {
        self.insert_var(var, Var::Vec2(value))
    }

    pub fn get_vec2(&self, var: &VarName, world: &legion::World) -> Option<vec2<f32>> {
        match self.get_var(var, world) {
            Some(value) => match value {
                Var::Vec2(value) => Some(value),
                _ => None,
            },
            None => None,
        }
    }

    pub fn insert_color(&mut self, var: VarName, value: Rgba<f32>) -> &mut Self {
        self.insert_var(var, Var::Color(value))
    }

    pub fn get_color(&self, var: &VarName, world: &legion::World) -> Option<Rgba<f32>> {
        match self.get_var(var, world) {
            Some(value) => match value {
                Var::Color(value) => Some(value),
                _ => None,
            },
            None => None,
        }
    }

    pub fn insert_string(&mut self, var: VarName, value: (usize, String)) -> &mut Self {
        self.insert_var(var, Var::String(value))
    }

    pub fn get_string(&self, var: &VarName, world: &legion::World) -> Option<String> {
        match self.get_var(var, world) {
            Some(value) => match value {
                Var::String(value) => Some(value.1),
                _ => None,
            },
            None => None,
        }
    }

    pub fn insert_faction(&mut self, var: VarName, value: Faction) -> &mut Self {
        self.insert_var(var, Var::Faction(value))
    }

    pub fn get_faction(&self, var: &VarName, world: &legion::World) -> Option<Faction> {
        match self.get_var(var, world) {
            Some(value) => match value {
                Var::Faction(value) => Some(value),
                _ => None,
            },
            None => None,
        }
    }

    pub fn get_status_charges(&self, name: &str, world: &legion::World) -> i32 {
        let mut result = 0;
        for layer in self.layers.iter() {
            result += layer.get_status_charges(name, world);
        }
        result
    }

    pub fn collect_statuses(&self, world: &legion::World) -> HashMap<String, i32> {
        let mut result = default();
        for layer in self.layers.iter() {
            layer.extend_statuses(&mut result, world);
        }
        result
    }

    pub fn len(&self) -> usize {
        self.layers.len()
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "o:{:?} t:{:?}\n>>>\n{}\n<<<\n",
            self.owner(),
            self.target(),
            self.layers
                .iter()
                .rev()
                .map(|x| x.to_string())
                .join("\n<- "),
        )
    }
}

impl Display for ContextLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let self_text = format!("{}:", self.as_ref()).bold();
        match self {
            ContextLayer::Target { entity }
            | ContextLayer::Attacker { entity }
            | ContextLayer::Unit { entity }
            | ContextLayer::Entity { entity } => write!(f, "{self_text} {entity:?}"),
            ContextLayer::Ability { ability } => write!(f, "{self_text} {ability}"),
            ContextLayer::Status {
                entity,
                name,
                charges,
            } => write!(f, "{self_text} {entity:?} {name} c:{charges}"),
            ContextLayer::Empty { name } => write!(f, "{self_text} {name}"),
            ContextLayer::Vars { vars } => write!(f, "{self_text} {vars}"),
            ContextLayer::Var { var, value } => write!(f, "{self_text} {var} -> {value}"),
        }
    }
}
