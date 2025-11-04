use super::*;

// Single link types with state as direct enum variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Component<T> {
    Loaded(T),
    Id(u64),
    None,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Owned<T> {
    Loaded(T),
    Id(u64),
    None,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Ref<T> {
    Id(u64),
    None,
    Unknown,
    #[serde(skip)]
    _Phantom(std::marker::PhantomData<T>),
}

// Multiple link types with state as direct enum variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnedMultiple<T> {
    Loaded(Vec<T>),
    Ids(Vec<u64>),
    None,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefMultiple<T> {
    Ids(Vec<u64>),
    None,
    Unknown,
    #[serde(skip)]
    _Phantom(std::marker::PhantomData<T>),
}

// Single node link trait
pub trait SingleLink<T> {
    fn new_loaded(value: T) -> Self;
    fn new_id(id: u64) -> Self;
    fn none() -> Self;
    fn unknown() -> Self;

    fn get(&self) -> NodeResult<&T>;
    fn get_mut(&mut self) -> NodeResult<&mut T>;
    fn take_loaded(&mut self) -> NodeResult<T>;

    fn id(&self) -> Option<u64>
    where
        T: Node;

    fn is_loaded(&self) -> bool;
    fn is_none(&self) -> bool;

    fn set_loaded(&mut self, value: T) -> NodeResult<()>;
    fn set_id(&mut self, id: u64) -> NodeResult<()>;
    fn set_none(&mut self) -> NodeResult<()>;
}

// Multiple node link trait
pub trait MultipleLink<T> {
    fn new_loaded(value: Vec<T>) -> Self;
    fn new_ids(ids: Vec<u64>) -> Self;
    fn none() -> Self;
    fn unknown() -> Self;

    fn get(&self) -> NodeResult<&Vec<T>>;
    fn get_mut(&mut self) -> NodeResult<&mut Vec<T>>;
    fn take_loaded(&mut self) -> NodeResult<Vec<T>>;

    fn ids(&self) -> Option<Vec<u64>>
    where
        T: Node;

    fn is_loaded(&self) -> bool;
    fn is_none(&self) -> bool;

    fn set_loaded(&mut self, value: Vec<T>) -> NodeResult<()>;
    fn set_ids(&mut self, ids: Vec<u64>) -> NodeResult<()>;
    fn set_none(&mut self) -> NodeResult<()>;
}

// Component implementation
impl<T> SingleLink<T> for Component<T> {
    fn new_loaded(value: T) -> Self {
        Component::Loaded(value)
    }

    fn new_id(id: u64) -> Self {
        Component::Id(id)
    }

    fn none() -> Self {
        Component::None
    }

    fn unknown() -> Self {
        Component::Unknown
    }

    fn get(&self) -> NodeResult<&T> {
        match self {
            Component::Loaded(val) => Ok(val),
            _ => Err(NodeError::custom("Component link not loaded")),
        }
    }

    fn get_mut(&mut self) -> NodeResult<&mut T> {
        match self {
            Component::Loaded(val) => Ok(val),
            _ => Err(NodeError::custom("Component link not loaded")),
        }
    }

    fn take_loaded(&mut self) -> NodeResult<T> {
        match std::mem::replace(self, Component::Unknown) {
            Component::Loaded(val) => Ok(val),
            other => {
                *self = other;
                Err(NodeError::custom("Component link not loaded"))
            }
        }
    }

    fn id(&self) -> Option<u64>
    where
        T: Node,
    {
        match self {
            Component::Loaded(val) => Some(val.id()),
            Component::Id(id) => Some(*id),
            _ => None,
        }
    }

    fn is_loaded(&self) -> bool {
        matches!(self, Component::Loaded(_))
    }

    fn is_none(&self) -> bool {
        matches!(self, Component::None)
    }

    fn set_loaded(&mut self, value: T) -> NodeResult<()> {
        *self = Component::Loaded(value);
        Ok(())
    }

    fn set_id(&mut self, id: u64) -> NodeResult<()> {
        *self = Component::Id(id);
        Ok(())
    }

    fn set_none(&mut self) -> NodeResult<()> {
        *self = Component::None;
        Ok(())
    }
}

// Owned implementation
impl<T> SingleLink<T> for Owned<T> {
    fn new_loaded(value: T) -> Self {
        Owned::Loaded(value)
    }

    fn new_id(id: u64) -> Self {
        Owned::Id(id)
    }

    fn none() -> Self {
        Owned::None
    }

    fn unknown() -> Self {
        Owned::Unknown
    }

    fn get(&self) -> NodeResult<&T> {
        match self {
            Owned::Loaded(val) => Ok(val),
            _ => Err(NodeError::custom("Owned link not loaded")),
        }
    }

    fn get_mut(&mut self) -> NodeResult<&mut T> {
        match self {
            Owned::Loaded(val) => Ok(val),
            _ => Err(NodeError::custom("Owned link not loaded")),
        }
    }

    fn take_loaded(&mut self) -> NodeResult<T> {
        match std::mem::replace(self, Owned::Unknown) {
            Owned::Loaded(val) => Ok(val),
            other => {
                *self = other;
                Err(NodeError::custom("Owned link not loaded"))
            }
        }
    }

    fn id(&self) -> Option<u64>
    where
        T: Node,
    {
        match self {
            Owned::Loaded(val) => Some(val.id()),
            Owned::Id(id) => Some(*id),
            _ => None,
        }
    }

    fn is_loaded(&self) -> bool {
        matches!(self, Owned::Loaded(_))
    }

    fn is_none(&self) -> bool {
        matches!(self, Owned::None)
    }

    fn set_loaded(&mut self, value: T) -> NodeResult<()> {
        *self = Owned::Loaded(value);
        Ok(())
    }

    fn set_id(&mut self, id: u64) -> NodeResult<()> {
        *self = Owned::Id(id);
        Ok(())
    }

    fn set_none(&mut self) -> NodeResult<()> {
        *self = Owned::None;
        Ok(())
    }
}

// Ref implementation - Note: does NOT support Loaded state
impl<T> SingleLink<T> for Ref<T> {
    fn new_loaded(_value: T) -> Self {
        panic!("Ref links do not support loaded state - use new_id() instead")
    }

    fn new_id(id: u64) -> Self {
        Ref::Id(id)
    }

    fn none() -> Self {
        Ref::None
    }

    fn unknown() -> Self {
        Ref::Unknown
    }

    fn get(&self) -> NodeResult<&T> {
        Err(NodeError::custom(
            "Ref links do not support getting loaded data",
        ))
    }

    fn get_mut(&mut self) -> NodeResult<&mut T> {
        Err(NodeError::custom(
            "Ref links do not support getting loaded data",
        ))
    }

    fn take_loaded(&mut self) -> NodeResult<T> {
        Err(NodeError::custom(
            "Ref links do not support getting loaded data",
        ))
    }

    fn id(&self) -> Option<u64>
    where
        T: Node,
    {
        match self {
            Ref::Id(id) => Some(*id),
            _ => None,
        }
    }

    fn is_loaded(&self) -> bool {
        false // Ref never has loaded state
    }

    fn is_none(&self) -> bool {
        matches!(self, Ref::None)
    }

    fn set_loaded(&mut self, _value: T) -> NodeResult<()> {
        Err(NodeError::custom("Ref links do not support loaded state"))
    }

    fn set_id(&mut self, id: u64) -> NodeResult<()> {
        *self = Ref::Id(id);
        Ok(())
    }

    fn set_none(&mut self) -> NodeResult<()> {
        *self = Ref::None;
        Ok(())
    }
}

// OwnedMultiple implementation
impl<T> MultipleLink<T> for OwnedMultiple<T> {
    fn new_loaded(value: Vec<T>) -> Self {
        OwnedMultiple::Loaded(value)
    }

    fn new_ids(ids: Vec<u64>) -> Self {
        OwnedMultiple::Ids(ids)
    }

    fn none() -> Self {
        OwnedMultiple::None
    }

    fn unknown() -> Self {
        OwnedMultiple::Unknown
    }

    fn get(&self) -> NodeResult<&Vec<T>> {
        match self {
            OwnedMultiple::Loaded(val) => Ok(val),
            _ => Err(NodeError::custom("OwnedMultiple link not loaded")),
        }
    }

    fn get_mut(&mut self) -> NodeResult<&mut Vec<T>> {
        match self {
            OwnedMultiple::Loaded(val) => Ok(val),
            OwnedMultiple::None => {
                self.set_loaded(default())?;
                self.get_mut()
            }
            _ => Err(NodeError::custom("OwnedMultiple link not loaded")),
        }
    }

    fn take_loaded(&mut self) -> NodeResult<Vec<T>> {
        match std::mem::replace(self, OwnedMultiple::Unknown) {
            OwnedMultiple::Loaded(val) => Ok(val),
            other => {
                *self = other;
                Err(NodeError::custom("OwnedMultiple link not loaded"))
            }
        }
    }

    fn ids(&self) -> Option<Vec<u64>>
    where
        T: Node,
    {
        match self {
            OwnedMultiple::Loaded(val) => Some(val.iter().map(|node| node.id()).collect()),
            OwnedMultiple::Ids(ids) => Some(ids.clone()),
            _ => None,
        }
    }

    fn is_loaded(&self) -> bool {
        matches!(self, OwnedMultiple::Loaded(_))
    }

    fn is_none(&self) -> bool {
        matches!(self, OwnedMultiple::None)
    }

    fn set_loaded(&mut self, value: Vec<T>) -> NodeResult<()> {
        *self = OwnedMultiple::Loaded(value);
        Ok(())
    }

    fn set_ids(&mut self, ids: Vec<u64>) -> NodeResult<()> {
        *self = OwnedMultiple::Ids(ids);
        Ok(())
    }

    fn set_none(&mut self) -> NodeResult<()> {
        *self = OwnedMultiple::None;
        Ok(())
    }
}

// RefMultiple implementation - Note: does NOT support Loaded state
impl<T> MultipleLink<T> for RefMultiple<T> {
    fn new_loaded(_value: Vec<T>) -> Self {
        panic!("RefMultiple links do not support loaded state - use new_ids() instead")
    }

    fn new_ids(ids: Vec<u64>) -> Self {
        RefMultiple::Ids(ids)
    }

    fn none() -> Self {
        RefMultiple::None
    }

    fn unknown() -> Self {
        RefMultiple::Unknown
    }

    fn get(&self) -> NodeResult<&Vec<T>> {
        Err(NodeError::custom(
            "RefMultiple links do not support getting loaded data",
        ))
    }

    fn get_mut(&mut self) -> NodeResult<&mut Vec<T>> {
        Err(NodeError::custom(
            "RefMultiple links do not support getting loaded data",
        ))
    }

    fn take_loaded(&mut self) -> NodeResult<Vec<T>> {
        Err(NodeError::custom(
            "RefMultiple links do not support getting loaded data",
        ))
    }

    fn ids(&self) -> Option<Vec<u64>>
    where
        T: Node,
    {
        match self {
            RefMultiple::Ids(ids) => Some(ids.clone()),
            _ => None,
        }
    }

    fn is_loaded(&self) -> bool {
        false // RefMultiple never has loaded state
    }

    fn is_none(&self) -> bool {
        matches!(self, RefMultiple::None)
    }

    fn set_loaded(&mut self, _value: Vec<T>) -> NodeResult<()> {
        Err(NodeError::custom(
            "RefMultiple links do not support loaded state",
        ))
    }

    fn set_ids(&mut self, ids: Vec<u64>) -> NodeResult<()> {
        *self = RefMultiple::Ids(ids);
        Ok(())
    }

    fn set_none(&mut self) -> NodeResult<()> {
        *self = RefMultiple::None;
        Ok(())
    }
}

// Default implementations
impl<T> Default for Component<T> {
    fn default() -> Self {
        Component::Unknown
    }
}

impl<T> Default for Owned<T> {
    fn default() -> Self {
        Owned::Unknown
    }
}

impl<T> Default for Ref<T> {
    fn default() -> Self {
        Ref::Unknown
    }
}

impl<T> Default for OwnedMultiple<T> {
    fn default() -> Self {
        OwnedMultiple::Unknown
    }
}

impl<T> Default for RefMultiple<T> {
    fn default() -> Self {
        RefMultiple::Unknown
    }
}

// IntoIterator implementations for single links
impl<'a, T> IntoIterator for &'a Owned<T> {
    type Item = &'a T;
    type IntoIter = std::option::IntoIter<&'a T>;

    fn into_iter(self) -> Self::IntoIter {
        self.get().ok().into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Component<T> {
    type Item = &'a T;
    type IntoIter = std::option::IntoIter<&'a T>;

    fn into_iter(self) -> Self::IntoIter {
        self.get().ok().into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Ref<T> {
    type Item = &'a T;
    type IntoIter = std::option::IntoIter<&'a T>;

    fn into_iter(self) -> Self::IntoIter {
        None.into_iter() // Ref never has loaded data
    }
}

// IntoIterator implementations for multiple links
impl<'a, T> IntoIterator for &'a OwnedMultiple<T> {
    type Item = &'a T;
    type IntoIter = std::iter::Flatten<std::option::IntoIter<std::slice::Iter<'a, T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.get().ok().map(|vec| vec.iter()).into_iter().flatten()
    }
}

impl<'a, T> IntoIterator for &'a RefMultiple<T> {
    type Item = &'a T;
    type IntoIter = std::iter::Flatten<std::option::IntoIter<std::slice::Iter<'a, T>>>;

    fn into_iter(self) -> Self::IntoIter {
        None.map(|vec: &Vec<T>| vec.iter()).into_iter().flatten() // RefMultiple never has loaded data
    }
}

// Extension trait for iteration functionality
pub trait LinkIterable<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_>;
}

// Single link iterable implementations
impl<T> LinkIterable<T> for Owned<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(self.get().ok().into_iter())
    }
}

impl<T> LinkIterable<T> for Component<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(self.get().ok().into_iter())
    }
}

impl<T> LinkIterable<T> for Ref<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(std::iter::empty()) // Ref never has loaded data
    }
}

// Multiple link iterable implementations
impl<T> LinkIterable<T> for OwnedMultiple<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        match self.get() {
            Ok(vec) => Box::new(vec.iter()),
            Err(_) => Box::new(std::iter::empty()),
        }
    }
}

impl<T> LinkIterable<T> for RefMultiple<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(std::iter::empty()) // RefMultiple never has loaded data
    }
}
