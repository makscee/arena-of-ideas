use std::collections::HashMap;

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
    pub child: u64,
    pub parent: u64,
}

impl PackedNodes {
    pub fn get<'a>(&'a self, id: u64) -> Option<&'a NodeData> {
        self.nodes.get(&id)
    }
    pub fn add_node(&mut self, kind: String, data: String, id: u64, parent: u64) {
        self.nodes.insert(id, NodeData { kind, data });
        if parent != 0 {
            self.links.push(NodeLink { child: id, parent });
        }
    }
    fn children(&self, id: u64) -> Vec<u64> {
        self.links
            .iter()
            .filter_map(
                |NodeLink { child, parent }| if *parent == id { Some(*child) } else { None },
            )
            .collect()
    }
    pub fn kind_children<'a>(&'a self, id: u64, kind: &str) -> Vec<(u64, &'a NodeData)> {
        self.children(id)
            .into_iter()
            .filter_map(|id| {
                if let Some(node) = self.nodes.get(&id) {
                    if node.kind.eq(kind) {
                        return Some((id, node));
                    }
                }
                None
            })
            .collect()
    }
}
