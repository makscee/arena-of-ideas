use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct ExtraNodes<T: Component + Clone> {
    pub nodes: Vec<(u64, T)>,
}

impl<T: Component + Clone> ExtraNodes<T> {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn add(&mut self, node_id: u64, component: T) {
        self.nodes.push((node_id, component));
    }

    pub fn remove(&mut self, node_id: u64) -> Option<T> {
        if let Some(pos) = self.nodes.iter().position(|(id, _)| *id == node_id) {
            Some(self.nodes.remove(pos).1)
        } else {
            None
        }
    }

    pub fn get(&self, node_id: u64) -> Option<&T> {
        self.nodes
            .iter()
            .find(|(id, _)| *id == node_id)
            .map(|(_, c)| c)
    }

    pub fn get_mut(&mut self, node_id: u64) -> Option<&mut T> {
        self.nodes
            .iter_mut()
            .find(|(id, _)| *id == node_id)
            .map(|(_, c)| c)
    }

    pub fn contains(&self, node_id: u64) -> bool {
        self.nodes.iter().any(|(id, _)| *id == node_id)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (u64, &T)> {
        self.nodes.iter().map(|(id, c)| (*id, c))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u64, &mut T)> {
        self.nodes.iter_mut().map(|(id, c)| (*id, c))
    }

    pub fn get_all_ids(&self) -> Vec<u64> {
        self.nodes.iter().map(|(id, _)| *id).collect()
    }
}

impl<T: Component + Clone> Default for ExtraNodes<T> {
    fn default() -> Self {
        Self::new()
    }
}
