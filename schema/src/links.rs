use super::*;

// Single node link state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LinkStateSingle<T> {
    Loaded(T),
    Id(u64),
    None,
    #[default]
    Unknown,
}

// Multiple node link state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LinkStateMultiple<T> {
    Loaded(Vec<T>),
    Ids(Vec<u64>),
    None,
    #[default]
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

impl<T> LinkIterable<T> for RefMultiple<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        match self.get() {
            Some(vec) => Box::new(vec.iter()),
            None => Box::new(std::iter::empty()),
        }
    }
}
