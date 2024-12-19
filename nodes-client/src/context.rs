use std::mem;

use bevy::prelude::{Component, World};
use utils::default;
use utils_client::get_children;

use super::*;

#[derive(Debug, Default, Clone)]
pub struct Context<'w, 's> {
    layers: Vec<ContextLayer>,
    sources: Vec<ContextSource<'w, 's>>,
}

#[derive(Debug, Clone)]
enum ContextSource<'w, 's> {
    Query(&'w StateQuery<'w, 's>),
    World(&'w World),
}

#[derive(Debug, Clone)]
enum ContextLayer {
    Owner(Entity),
    Var(VarName, VarValue),
}

impl<'w, 's> Context<'w, 's> {
    pub fn new(state: &'w StateQuery<'w, 's>) -> Self {
        Self {
            layers: default(),
            sources: vec![ContextSource::Query(state)],
        }
    }
    pub fn new_world(world: &'w World) -> Self {
        Self {
            layers: default(),
            sources: vec![ContextSource::World(world)],
        }
    }
    pub fn set_owner(&mut self, owner: Entity) -> &mut Self {
        self.layers.push(ContextLayer::Owner(owner));
        self
    }
    pub fn set_var(&mut self, var: VarName, value: VarValue) -> &mut Self {
        self.layers.push(ContextLayer::Var(var, value));
        self
    }

    pub fn get_owner(&self) -> Option<Entity> {
        self.layers.iter().rev().find_map(|l| l.get_owner())
    }
    pub fn get_var(&self, var: VarName) -> Option<VarValue> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_var(var, &self.sources))
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
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        for s in self.sources.iter().rev() {
            if let Some(c) = s.get_component::<T>(entity) {
                return Some(c);
            }
        }
        None
    }

    pub fn clear(&mut self) {
        self.layers.clear();
    }
    pub fn take(&mut self) -> Self {
        mem::take(self)
    }
}

impl ContextSource<'_, '_> {
    fn get_var(&self, entity: Entity, var: VarName) -> Option<VarValue> {
        match self {
            ContextSource::Query(state) => NodeState::get_var_state(var, entity, state),
            ContextSource::World(world) => NodeState::get_var_world(var, entity, world),
        }
    }
    fn get_children(&self, entity: Entity) -> Vec<Entity> {
        match self {
            ContextSource::Query(state) => state.get_children(entity),
            ContextSource::World(world) => get_children(entity, world),
        }
    }
    fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        match self {
            ContextSource::World(world) => world.get::<T>(entity),
            ContextSource::Query(..) => None,
        }
    }
}

impl ContextLayer {
    fn get_owner(&self) -> Option<Entity> {
        match self {
            ContextLayer::Owner(entity) => Some(*entity),
            _ => None,
        }
    }
    fn get_var(&self, var: VarName, sources: &Vec<ContextSource>) -> Option<VarValue> {
        match self {
            ContextLayer::Owner(entity) => sources
                .into_iter()
                .rev()
                .find_map(|s| s.get_var(*entity, var)),
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
