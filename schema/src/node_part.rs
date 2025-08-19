use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodePartRelation {
    Parent,
    Child,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodePartState<T> {
    None,
    Id(u64),
    Node(T),
    Unknown,
}

impl<T> Default for NodePartState<T> {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodePartsState<T> {
    None,
    Ids(Vec<u64>),
    Nodes(Vec<T>),
    Unknown,
}

impl<T> Default for NodePartsState<T> {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodePart<R, T> {
    pub state: NodePartState<T>,
    _relation: PhantomData<R>,
}

impl<R, T> Default for NodePart<R, T> {
    fn default() -> Self {
        Self {
            state: Default::default(),
            _relation: PhantomData,
        }
    }
}

impl<R, T> NodePart<R, T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(id: u64) -> Self {
        Self {
            state: NodePartState::Id(id),
            _relation: PhantomData,
        }
    }

    pub fn with_node(data: T) -> Self {
        Self {
            state: NodePartState::Node(data),
            _relation: PhantomData,
        }
    }

    pub fn unknown() -> Self {
        Self {
            state: NodePartState::Unknown,
            _relation: PhantomData,
        }
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self.state, NodePartState::Node(_))
    }

    pub fn is_linked(&self) -> bool {
        matches!(self.state, NodePartState::Id(_) | NodePartState::Node(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self.state, NodePartState::None)
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self.state, NodePartState::Unknown)
    }

    pub fn get_id(&self) -> Option<u64> {
        match &self.state {
            NodePartState::Id(id) => Some(*id),
            _ => None,
        }
    }

    pub fn get_data(&self) -> Option<&T> {
        match &self.state {
            NodePartState::Node(data) => Some(data),
            _ => None,
        }
    }

    pub fn get_data_mut(&mut self) -> Option<&mut T> {
        match &mut self.state {
            NodePartState::Node(data) => Some(data),
            _ => None,
        }
    }

    pub fn take_data(&mut self) -> Option<T> {
        match std::mem::take(&mut self.state) {
            NodePartState::Node(data) => Some(data),
            other => {
                self.state = other;
                None
            }
        }
    }

    pub fn set_state(&mut self, state: NodePartState<T>) {
        self.state = state;
    }

    pub fn set_id(&mut self, id: u64) {
        self.state = NodePartState::Id(id);
    }

    pub fn set_data(&mut self, data: T) {
        self.state = NodePartState::Node(data);
    }

    pub fn set_none(&mut self) {
        self.state = NodePartState::None;
    }

    pub fn set_unknown(&mut self) {
        self.state = NodePartState::Unknown;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Parent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Child;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeParts<R, T> {
    pub state: NodePartsState<T>,
    _relation: PhantomData<R>,
}

impl<R, T> Default for NodeParts<R, T> {
    fn default() -> Self {
        Self {
            state: Default::default(),
            _relation: PhantomData,
        }
    }
}

impl<R, T> NodeParts<R, T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_ids(ids: Vec<u64>) -> Self {
        Self {
            state: NodePartsState::Ids(ids),
            _relation: PhantomData,
        }
    }

    pub fn with_nodes(data: Vec<T>) -> Self {
        Self {
            state: NodePartsState::Nodes(data),
            _relation: PhantomData,
        }
    }

    pub fn unknown() -> Self {
        Self {
            state: NodePartsState::Unknown,
            _relation: PhantomData,
        }
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self.state, NodePartsState::Nodes(_))
    }

    pub fn is_linked(&self) -> bool {
        matches!(
            self.state,
            NodePartsState::Ids(_) | NodePartsState::Nodes(_)
        )
    }

    pub fn is_none(&self) -> bool {
        matches!(self.state, NodePartsState::None)
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self.state, NodePartsState::Unknown)
    }

    pub fn get_ids(&self) -> Option<&Vec<u64>> {
        match &self.state {
            NodePartsState::Ids(ids) => Some(ids),
            _ => None,
        }
    }

    pub fn get_data(&self) -> Option<&Vec<T>> {
        match &self.state {
            NodePartsState::Nodes(data) => Some(data),
            _ => None,
        }
    }

    pub fn get_data_mut(&mut self) -> Option<&mut Vec<T>> {
        match &mut self.state {
            NodePartsState::Nodes(data) => Some(data),
            _ => None,
        }
    }

    pub fn take_data(&mut self) -> Option<Vec<T>> {
        match std::mem::take(&mut self.state) {
            NodePartsState::Nodes(data) => Some(data),
            other => {
                self.state = other;
                None
            }
        }
    }

    pub fn set_state(&mut self, state: NodePartsState<T>) {
        self.state = state;
    }

    pub fn set_ids(&mut self, ids: Vec<u64>) {
        self.state = NodePartsState::Ids(ids);
    }

    pub fn set_data(&mut self, data: Vec<T>) {
        self.state = NodePartsState::Nodes(data);
    }

    pub fn set_none(&mut self) {
        self.state = NodePartsState::None;
    }

    pub fn set_unknown(&mut self) {
        self.state = NodePartsState::Unknown;
    }

    pub fn len(&self) -> usize {
        match &self.state {
            NodePartsState::Ids(ids) => ids.len(),
            NodePartsState::Nodes(nodes) => nodes.len(),
            _ => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, item: T) {
        match &mut self.state {
            NodePartsState::Nodes(nodes) => nodes.push(item),
            _ => {
                self.state = NodePartsState::Nodes(vec![item]);
            }
        }
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        match &self.state {
            NodePartsState::Nodes(nodes) => nodes.iter(),
            _ => [].iter(),
        }
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        match &mut self.state {
            NodePartsState::Nodes(nodes) => nodes.iter_mut(),
            _ => [].iter_mut(),
        }
    }

    pub fn collect_nodes(nodes: Vec<T>) -> Self {
        Self {
            state: NodePartsState::Nodes(nodes),
            _relation: PhantomData,
        }
    }

    pub fn collect_ids(ids: Vec<u64>) -> Self {
        Self {
            state: NodePartsState::Ids(ids),
            _relation: PhantomData,
        }
    }
}

impl<R, T> IntoIterator for NodeParts<R, T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self.state {
            NodePartsState::Nodes(nodes) => nodes.into_iter(),
            _ => vec![].into_iter(),
        }
    }
}

impl<'a, R, T> IntoIterator for &'a NodeParts<R, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, R, T> IntoIterator for &'a mut NodeParts<R, T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<R, T> FromIterator<T> for NodeParts<R, T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::collect_nodes(iter.into_iter().collect())
    }
}
