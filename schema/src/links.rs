use super::*;

// Single node link state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkStateSingle<T> {
    Loaded(T),
    Id(u64),
    None,
    Unknown,
}

// Multiple node link state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkStateMultiple<T> {
    Loaded(Vec<T>),
    Ids(Vec<u64>),
    None,
    Unknown,
}

// Single node link trait
pub trait SingleLink<T> {
    fn state<'a>(&'a self) -> &'a LinkStateSingle<T>;
    fn state_mut(&mut self) -> &mut LinkStateSingle<T>;

    fn new_loaded(value: T) -> Self;
    fn new_id(id: u64) -> Self;
    fn none() -> Self;
    fn unknown() -> Self;

    fn get<'a>(&'a self) -> Option<&'a T> {
        match self.state() {
            LinkStateSingle::Loaded(val) => Some(val),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self) -> NodeResult<&'a mut T> {
        self.state_mut().data_mut()
    }

    fn id(&self) -> Option<u64>
    where
        T: Node,
    {
        match self.state() {
            LinkStateSingle::Loaded(val) => Some(val.id()),
            LinkStateSingle::Id(id) => Some(*id),
            _ => None,
        }
    }

    fn is_loaded(&self) -> bool {
        matches!(self.state(), LinkStateSingle::Loaded(_))
    }

    fn is_none(&self) -> bool {
        matches!(self.state(), LinkStateSingle::None)
    }
}

// Multiple node link trait
pub trait MultipleLink<T> {
    fn state<'a>(&'a self) -> &'a LinkStateMultiple<T>;
    fn state_mut(&mut self) -> &mut LinkStateMultiple<T>;

    fn new_loaded(value: Vec<T>) -> Self;
    fn new_ids(ids: Vec<u64>) -> Self;
    fn none() -> Self;
    fn unknown() -> Self;

    fn get<'a>(&'a self) -> Option<&'a Vec<T>> {
        match self.state() {
            LinkStateMultiple::Loaded(val) => Some(val),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self) -> NodeResult<&'a mut Vec<T>> {
        self.state_mut().data_mut()
    }

    fn ids(&self) -> Option<Vec<u64>>
    where
        T: Node,
    {
        match self.state() {
            LinkStateMultiple::Loaded(val) => Some(val.iter().map(|node| node.id()).collect()),
            LinkStateMultiple::Ids(ids) => Some(ids.clone()),
            _ => None,
        }
    }

    fn is_loaded(&self) -> bool {
        matches!(self.state(), LinkStateMultiple::Loaded(_))
    }

    fn is_none(&self) -> bool {
        matches!(self.state(), LinkStateMultiple::None)
    }
}

impl<T> LinkStateSingle<T> {
    pub fn set(&mut self, data: T) {
        *self = LinkStateSingle::Loaded(data);
    }

    pub fn data_mut(&mut self) -> NodeResult<&mut T> {
        match self {
            LinkStateSingle::Loaded(d) => Ok(d),
            _ => Err(NodeError::custom("Single link not loaded")),
        }
    }
}

impl<T> LinkStateMultiple<T> {
    pub fn set(&mut self, data: Vec<T>) {
        *self = LinkStateMultiple::Loaded(data);
    }

    pub fn data_mut(&mut self) -> NodeResult<&mut Vec<T>> {
        match self {
            LinkStateMultiple::Loaded(d) => Ok(d),
            _ => Err(NodeError::custom("Multiple link not loaded")),
        }
    }
}

// Single node link types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component<T> {
    state: LinkStateSingle<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owned<T> {
    state: LinkStateSingle<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ref<T> {
    state: LinkStateSingle<T>,
}

// Multiple node link types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMultiple<T> {
    state: LinkStateMultiple<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedMultiple<T> {
    state: LinkStateMultiple<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefMultiple<T> {
    state: LinkStateMultiple<T>,
}

// Single link implementations
impl<T> SingleLink<T> for Component<T> {
    fn state(&self) -> &LinkStateSingle<T> {
        &self.state
    }

    fn state_mut(&mut self) -> &mut LinkStateSingle<T> {
        &mut self.state
    }

    fn new_loaded(value: T) -> Self {
        Self {
            state: LinkStateSingle::Loaded(value),
        }
    }

    fn new_id(id: u64) -> Self {
        Self {
            state: LinkStateSingle::Id(id),
        }
    }

    fn none() -> Self {
        Self {
            state: LinkStateSingle::None,
        }
    }

    fn unknown() -> Self {
        Self {
            state: LinkStateSingle::Unknown,
        }
    }
}

impl<T> SingleLink<T> for Owned<T> {
    fn state(&self) -> &LinkStateSingle<T> {
        &self.state
    }

    fn state_mut(&mut self) -> &mut LinkStateSingle<T> {
        &mut self.state
    }

    fn new_loaded(value: T) -> Self {
        Self {
            state: LinkStateSingle::Loaded(value),
        }
    }

    fn new_id(id: u64) -> Self {
        Self {
            state: LinkStateSingle::Id(id),
        }
    }

    fn none() -> Self {
        Self {
            state: LinkStateSingle::None,
        }
    }

    fn unknown() -> Self {
        Self {
            state: LinkStateSingle::Unknown,
        }
    }
}

impl<T> SingleLink<T> for Ref<T> {
    fn state(&self) -> &LinkStateSingle<T> {
        &self.state
    }

    fn state_mut(&mut self) -> &mut LinkStateSingle<T> {
        &mut self.state
    }

    fn new_loaded(value: T) -> Self {
        Self {
            state: LinkStateSingle::Loaded(value),
        }
    }

    fn new_id(id: u64) -> Self {
        Self {
            state: LinkStateSingle::Id(id),
        }
    }

    fn none() -> Self {
        Self {
            state: LinkStateSingle::None,
        }
    }

    fn unknown() -> Self {
        Self {
            state: LinkStateSingle::Unknown,
        }
    }
}

// Multiple link implementations
impl<T> MultipleLink<T> for ComponentMultiple<T> {
    fn state(&self) -> &LinkStateMultiple<T> {
        &self.state
    }

    fn state_mut(&mut self) -> &mut LinkStateMultiple<T> {
        &mut self.state
    }

    fn new_loaded(value: Vec<T>) -> Self {
        Self {
            state: LinkStateMultiple::Loaded(value),
        }
    }

    fn new_ids(ids: Vec<u64>) -> Self {
        Self {
            state: LinkStateMultiple::Ids(ids),
        }
    }

    fn none() -> Self {
        Self {
            state: LinkStateMultiple::None,
        }
    }

    fn unknown() -> Self {
        Self {
            state: LinkStateMultiple::Unknown,
        }
    }
}

impl<T> MultipleLink<T> for OwnedMultiple<T> {
    fn state(&self) -> &LinkStateMultiple<T> {
        &self.state
    }

    fn state_mut(&mut self) -> &mut LinkStateMultiple<T> {
        &mut self.state
    }

    fn new_loaded(value: Vec<T>) -> Self {
        Self {
            state: LinkStateMultiple::Loaded(value),
        }
    }

    fn new_ids(ids: Vec<u64>) -> Self {
        Self {
            state: LinkStateMultiple::Ids(ids),
        }
    }

    fn none() -> Self {
        Self {
            state: LinkStateMultiple::None,
        }
    }

    fn unknown() -> Self {
        Self {
            state: LinkStateMultiple::Unknown,
        }
    }
}

impl<T> MultipleLink<T> for RefMultiple<T> {
    fn state(&self) -> &LinkStateMultiple<T> {
        &self.state
    }

    fn state_mut(&mut self) -> &mut LinkStateMultiple<T> {
        &mut self.state
    }

    fn new_loaded(value: Vec<T>) -> Self {
        Self {
            state: LinkStateMultiple::Loaded(value),
        }
    }

    fn new_ids(ids: Vec<u64>) -> Self {
        Self {
            state: LinkStateMultiple::Ids(ids),
        }
    }

    fn none() -> Self {
        Self {
            state: LinkStateMultiple::None,
        }
    }

    fn unknown() -> Self {
        Self {
            state: LinkStateMultiple::Unknown,
        }
    }
}

// Default implementations
impl<T> Default for Component<T> {
    fn default() -> Self {
        Self::unknown()
    }
}

impl<T> Default for Owned<T> {
    fn default() -> Self {
        Self::unknown()
    }
}

impl<T> Default for Ref<T> {
    fn default() -> Self {
        Self::unknown()
    }
}

impl<T> Default for ComponentMultiple<T> {
    fn default() -> Self {
        Self::unknown()
    }
}

impl<T> Default for OwnedMultiple<T> {
    fn default() -> Self {
        Self::unknown()
    }
}

impl<T> Default for RefMultiple<T> {
    fn default() -> Self {
        Self::unknown()
    }
}

// IntoIterator implementations for single links
impl<'a, T> IntoIterator for &'a Owned<T> {
    type Item = &'a T;
    type IntoIter = std::option::IntoIter<&'a T>;

    fn into_iter(self) -> Self::IntoIter {
        self.get().into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Component<T> {
    type Item = &'a T;
    type IntoIter = std::option::IntoIter<&'a T>;

    fn into_iter(self) -> Self::IntoIter {
        self.get().into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Ref<T> {
    type Item = &'a T;
    type IntoIter = std::option::IntoIter<&'a T>;

    fn into_iter(self) -> Self::IntoIter {
        self.get().into_iter()
    }
}

// IntoIterator implementations for multiple links
impl<'a, T> IntoIterator for &'a OwnedMultiple<T> {
    type Item = &'a T;
    type IntoIter = std::iter::Flatten<std::option::IntoIter<std::slice::Iter<'a, T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.get().map(|vec| vec.iter()).into_iter().flatten()
    }
}

impl<'a, T> IntoIterator for &'a ComponentMultiple<T> {
    type Item = &'a T;
    type IntoIter = std::iter::Flatten<std::option::IntoIter<std::slice::Iter<'a, T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.get().map(|vec| vec.iter()).into_iter().flatten()
    }
}

impl<'a, T> IntoIterator for &'a RefMultiple<T> {
    type Item = &'a T;
    type IntoIter = std::iter::Flatten<std::option::IntoIter<std::slice::Iter<'a, T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.get().map(|vec| vec.iter()).into_iter().flatten()
    }
}

// Extension trait for iteration functionality
pub trait LinkIterable<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_>;
}

// Single link iterable implementations
impl<T> LinkIterable<T> for Owned<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(self.get().into_iter())
    }
}

impl<T> LinkIterable<T> for Component<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(self.get().into_iter())
    }
}

impl<T> LinkIterable<T> for Ref<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(self.get().into_iter())
    }
}

// Multiple link iterable implementations
impl<T> LinkIterable<T> for OwnedMultiple<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        match self.get() {
            Some(vec) => Box::new(vec.iter()),
            None => Box::new(std::iter::empty()),
        }
    }
}

impl<T> LinkIterable<T> for ComponentMultiple<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        match self.get() {
            Some(vec) => Box::new(vec.iter()),
            None => Box::new(std::iter::empty()),
        }
    }
}

impl<T> LinkIterable<T> for RefMultiple<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        match self.get() {
            Some(vec) => Box::new(vec.iter()),
            None => Box::new(std::iter::empty()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_and_multiple_links() {
        use std::collections::{HashMap, HashSet};
        use std::str::FromStr;

        // Create a mock Node struct for testing
        #[derive(Default, Clone, Serialize, Deserialize)]
        struct MockNode {
            id: u64,
        }

        impl Node for MockNode {
            fn id(&self) -> u64 {
                self.id
            }
            fn set_id(&mut self, id: u64) {
                self.id = id;
            }
            fn owner(&self) -> u64 {
                0
            }
            fn set_owner(&mut self, _owner: u64) {}
            fn kind(&self) -> NodeKind {
                NodeKind::from_str("Unit").unwrap()
            }
            fn reassign_ids(&mut self, _next_id: &mut u64) {}
            fn kind_s() -> NodeKind
            where
                Self: Sized,
            {
                NodeKind::from_str("Unit").unwrap()
            }
            fn var_names() -> HashSet<VarName>
            where
                Self: Sized,
            {
                HashSet::new()
            }
            fn set_var(&mut self, _var: VarName, _value: VarValue) -> NodeResult<()> {
                Ok(())
            }
            fn get_var(&self, _var: VarName) -> NodeResult<VarValue> {
                Err("No variables".into())
            }
            fn get_vars(&self) -> HashMap<VarName, VarValue> {
                HashMap::new()
            }
            fn pack_links(
                &self,
                _packed: &mut PackedNodes,
                _visited: &mut std::collections::HashSet<u64>,
            ) {
            }
            fn unpack_links(&mut self, _packed: &PackedNodes) {}
        }

        // Test single Node - loaded state
        let node = MockNode { id: 42 };
        let owned_node: Owned<MockNode> = Owned::new_loaded(node);
        assert_eq!(owned_node.id(), Some(42));
        assert!(owned_node.is_loaded());

        // Test single Node - id state
        let owned_id: Owned<MockNode> = Owned::new_id(123);
        assert_eq!(owned_id.id(), Some(123));
        assert!(!owned_id.is_loaded());

        // Test multiple Nodes - loaded state
        let nodes = vec![
            MockNode { id: 10 },
            MockNode { id: 20 },
            MockNode { id: 30 },
        ];
        let owned_nodes: OwnedMultiple<MockNode> = OwnedMultiple::new_loaded(nodes);
        assert_eq!(owned_nodes.ids(), Some(vec![10, 20, 30]));
        assert!(owned_nodes.is_loaded());

        // Test multiple Nodes - ids state
        let owned_ids: OwnedMultiple<MockNode> = OwnedMultiple::new_ids(vec![1, 2, 3]);
        assert_eq!(owned_ids.ids(), Some(vec![1, 2, 3]));
        assert!(!owned_ids.is_loaded());

        // Test iteration for single links
        let node = MockNode { id: 99 };
        let component_node: Component<MockNode> = Component::new_loaded(node);
        let collected: Vec<&MockNode> = component_node.iter().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].id, 99);

        // Test iteration for multiple links
        let nodes = vec![MockNode { id: 77 }, MockNode { id: 88 }];
        let ref_nodes: RefMultiple<MockNode> = RefMultiple::new_loaded(nodes);
        let collected: Vec<&MockNode> = ref_nodes.iter().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0].id, 77);
        assert_eq!(collected[1].id, 88);

        // Test for-loop iteration
        let nodes = vec![MockNode { id: 100 }, MockNode { id: 200 }];
        let multiple_nodes: ComponentMultiple<MockNode> = ComponentMultiple::new_loaded(nodes);
        let mut sum = 0;
        for node in &multiple_nodes {
            sum += node.id;
        }
        assert_eq!(sum, 300);
    }
}
