use std::collections::HashMap;

use itertools::Itertools;

use super::*;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PackedNodes {
    pub root: u64,
    pub nodes: HashMap<u64, NodeData>,
    pub links: Vec<NodeLink>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct NodeData {
    pub kind: String,
    pub data: String,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct NodeLink {
    pub parent: u64,
    pub child: u64,
    pub parent_kind: String,
    pub child_kind: String,
}

impl PackedNodes {
    pub fn kind(&self) -> &String {
        &self.nodes.get(&self.root).unwrap().kind
    }
    pub fn get<'a>(&'a self, id: u64) -> Option<&'a NodeData> {
        self.nodes.get(&id)
    }
    pub fn add_node(&mut self, kind: String, data: String, id: u64) {
        self.nodes.insert(id, NodeData { kind, data });
    }
    pub fn link_parent_child(
        &mut self,
        parent: u64,
        child: u64,
        parent_kind: String,
        child_kind: String,
    ) {
        self.links.push(NodeLink {
            child,
            parent,
            parent_kind,
            child_kind,
        });
    }
    pub fn kind_parents(&self, id: u64, kind: &str) -> Vec<u64> {
        self.links
            .iter()
            .filter_map(
                |NodeLink {
                     child,
                     parent,
                     parent_kind,
                     ..
                 }| {
                    if *child == id && parent_kind.eq(kind) {
                        Some(*parent)
                    } else {
                        None
                    }
                },
            )
            .collect()
    }
    pub fn kind_children(&self, id: u64, kind: &str) -> Vec<u64> {
        self.links
            .iter()
            .filter_map(
                |NodeLink {
                     child,
                     parent,
                     child_kind,
                     ..
                 }| {
                    if *parent == id && child_kind.eq(kind) {
                        Some(*child)
                    } else {
                        None
                    }
                },
            )
            .collect()
    }
    pub fn reassign_ids(&mut self, next_id: &mut u64) {
        let mut remap: HashMap<u64, u64> = default();
        for (id, data) in self.nodes.drain().collect_vec() {
            let new_id = *next_id;
            *next_id += 1;
            remap.insert(id, new_id);
            self.nodes.insert(new_id, data);
        }
        for NodeLink {
            parent,
            child,
            parent_kind: _,
            child_kind: _,
        } in &mut self.links
        {
            if let Some(id) = remap.get(parent) {
                *parent = *id;
            }
            if let Some(id) = remap.get(child) {
                *child = *id;
            }
        }
    }
}
