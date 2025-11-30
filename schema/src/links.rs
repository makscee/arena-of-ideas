use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Component<T> {
    Loaded(u64, T),
    None(u64),
    Unknown(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Owned<T> {
    Loaded(u64, T),
    None(u64),
    Unknown(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Ref<T> {
    Loaded(u64, T),
    None(u64),
    Unknown(u64),
    #[serde(skip)]
    _Phantom(std::marker::PhantomData<T>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnedMultiple<T> {
    Loaded(u64, Vec<T>),
    None(u64),
    Unknown(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefMultiple<T> {
    Loaded(u64, Vec<T>),
    None(u64),
    Unknown(u64),
    #[serde(skip)]
    _Phantom(std::marker::PhantomData<T>),
}

pub trait SingleLink<T> {
    fn new_loaded(parent_id: u64, value: T) -> Self;
    fn none(parent_id: u64) -> Self;
    fn unknown(parent_id: u64) -> Self;

    fn get(&self) -> NodeResult<&T>;
    fn get_mut(&mut self) -> NodeResult<&mut T>;
    fn take_loaded(&mut self) -> NodeResult<T>;

    fn is_loaded(&self) -> bool;
    fn is_none(&self) -> bool;

    fn set_loaded(&mut self, value: T);
    fn set_none(&mut self);

    fn parent_id(&self) -> u64;
}

pub trait MultipleLink<T> {
    fn new_loaded(parent_id: u64, value: Vec<T>) -> Self;
    fn none(parent_id: u64) -> Self;
    fn unknown(parent_id: u64) -> Self;

    fn get(&self) -> NodeResult<&Vec<T>>;
    fn get_mut(&mut self) -> NodeResult<&mut Vec<T>>;
    fn take_loaded(&mut self) -> NodeResult<Vec<T>>;

    fn is_loaded(&self) -> bool;
    fn is_none(&self) -> bool;

    fn set_loaded(&mut self, value: Vec<T>);
    fn set_none(&mut self);

    fn parent_id(&self) -> u64;

    fn push(&mut self, node: T) -> NodeResult<()> {
        match self {
            _ if self.is_loaded() => {
                self.get_mut()?.push(node);
                Ok(())
            }
            _ if self.is_none() => {
                self.set_loaded(vec![node]);
                Ok(())
            }
            _ => Err(NodeError::custom(
                "Cannot push to link that is not Loaded or None",
            )),
        }
    }
}

impl<T> SingleLink<T> for Component<T> {
    fn new_loaded(parent_id: u64, value: T) -> Self {
        Component::Loaded(parent_id, value)
    }

    fn none(parent_id: u64) -> Self {
        Component::None(parent_id)
    }

    fn unknown(parent_id: u64) -> Self {
        Component::Unknown(parent_id)
    }

    fn get(&self) -> NodeResult<&T> {
        match self {
            Component::Loaded(_, val) => Ok(val),
            _ => Err(NodeError::custom("Component link not loaded")),
        }
    }

    fn get_mut(&mut self) -> NodeResult<&mut T> {
        match self {
            Component::Loaded(_, val) => Ok(val),
            _ => Err(NodeError::custom("Component link not loaded")),
        }
    }

    fn take_loaded(&mut self) -> NodeResult<T> {
        let parent_id = self.parent_id();
        match std::mem::replace(self, Component::Unknown(parent_id)) {
            Component::Loaded(_, val) => Ok(val),
            other => {
                *self = other;
                Err(NodeError::custom("Component link not loaded"))
            }
        }
    }

    fn is_loaded(&self) -> bool {
        matches!(self, Component::Loaded(..))
    }

    fn is_none(&self) -> bool {
        matches!(self, Component::None(..))
    }

    fn parent_id(&self) -> u64 {
        match self {
            Component::Loaded(id, _) | Component::None(id) | Component::Unknown(id) => *id,
        }
    }

    fn set_loaded(&mut self, value: T) {
        let parent_id = self.parent_id();
        *self = Component::Loaded(parent_id, value);
    }

    fn set_none(&mut self) {
        let parent_id = self.parent_id();
        *self = Component::None(parent_id);
    }
}

impl<T> SingleLink<T> for Owned<T> {
    fn new_loaded(parent_id: u64, value: T) -> Self {
        Owned::Loaded(parent_id, value)
    }

    fn none(parent_id: u64) -> Self {
        Owned::None(parent_id)
    }

    fn unknown(parent_id: u64) -> Self {
        Owned::Unknown(parent_id)
    }

    fn get(&self) -> NodeResult<&T> {
        match self {
            Owned::Loaded(_, val) => Ok(val),
            _ => Err(NodeError::custom("Owned link not loaded")),
        }
    }

    fn get_mut(&mut self) -> NodeResult<&mut T> {
        match self {
            Owned::Loaded(_, val) => Ok(val),
            _ => Err(NodeError::custom("Owned link not loaded")),
        }
    }

    fn take_loaded(&mut self) -> NodeResult<T> {
        let parent_id = self.parent_id();
        match std::mem::replace(self, Owned::Unknown(parent_id)) {
            Owned::Loaded(_, val) => Ok(val),
            other => {
                *self = other;
                Err(NodeError::custom("Owned link not loaded"))
            }
        }
    }

    fn is_loaded(&self) -> bool {
        matches!(self, Owned::Loaded(..))
    }

    fn is_none(&self) -> bool {
        matches!(self, Owned::None(..))
    }

    fn parent_id(&self) -> u64 {
        match self {
            Owned::Loaded(id, _) | Owned::None(id) | Owned::Unknown(id) => *id,
        }
    }

    fn set_loaded(&mut self, value: T) {
        let parent_id = self.parent_id();
        *self = Owned::Loaded(parent_id, value);
    }

    fn set_none(&mut self) {
        let parent_id = self.parent_id();
        *self = Owned::None(parent_id);
    }
}

impl<T> SingleLink<T> for Ref<T> {
    fn new_loaded(parent_id: u64, value: T) -> Self {
        Ref::Loaded(parent_id, value)
    }

    fn none(parent_id: u64) -> Self {
        Ref::None(parent_id)
    }

    fn unknown(parent_id: u64) -> Self {
        Ref::Unknown(parent_id)
    }

    fn get(&self) -> NodeResult<&T> {
        match self {
            Ref::Loaded(_, val) => Ok(val),
            _ => Err(NodeError::custom("Ref link not loaded")),
        }
    }

    fn get_mut(&mut self) -> NodeResult<&mut T> {
        match self {
            Ref::Loaded(_, val) => Ok(val),
            _ => Err(NodeError::custom("Ref link not loaded")),
        }
    }

    fn take_loaded(&mut self) -> NodeResult<T> {
        let parent_id = self.parent_id();
        match std::mem::replace(self, Ref::Unknown(parent_id)) {
            Ref::Loaded(_, val) => Ok(val),
            other => {
                *self = other;
                Err(NodeError::custom("Ref link not loaded"))
            }
        }
    }

    fn is_loaded(&self) -> bool {
        matches!(self, Ref::Loaded(..))
    }

    fn is_none(&self) -> bool {
        matches!(self, Ref::None(..))
    }

    fn parent_id(&self) -> u64 {
        match self {
            Ref::Loaded(id, _) | Ref::None(id) | Ref::Unknown(id) => *id,
            Ref::_Phantom(_) => 0,
        }
    }

    fn set_loaded(&mut self, value: T) {
        let parent_id = self.parent_id();
        *self = Ref::Loaded(parent_id, value);
    }

    fn set_none(&mut self) {
        let parent_id = self.parent_id();
        *self = Ref::None(parent_id);
    }
}

impl<T> MultipleLink<T> for OwnedMultiple<T> {
    fn new_loaded(parent_id: u64, value: Vec<T>) -> Self {
        OwnedMultiple::Loaded(parent_id, value)
    }

    fn none(parent_id: u64) -> Self {
        OwnedMultiple::None(parent_id)
    }

    fn unknown(parent_id: u64) -> Self {
        OwnedMultiple::Unknown(parent_id)
    }

    fn get(&self) -> NodeResult<&Vec<T>> {
        match self {
            OwnedMultiple::Loaded(_, val) => Ok(val),
            _ => Err(NodeError::custom("OwnedMultiple link not loaded")),
        }
    }

    fn get_mut(&mut self) -> NodeResult<&mut Vec<T>> {
        match self {
            OwnedMultiple::Loaded(_, val) => Ok(val),
            OwnedMultiple::None(_) => {
                self.set_loaded(Vec::new());
                self.get_mut()
            }
            _ => Err(NodeError::custom("OwnedMultiple link not loaded")),
        }
    }

    fn take_loaded(&mut self) -> NodeResult<Vec<T>> {
        let parent_id = self.parent_id();
        match std::mem::replace(self, OwnedMultiple::Unknown(parent_id)) {
            OwnedMultiple::Loaded(_, val) => Ok(val),
            other => {
                *self = other;
                Err(NodeError::custom("OwnedMultiple link not loaded"))
            }
        }
    }

    fn is_loaded(&self) -> bool {
        matches!(self, OwnedMultiple::Loaded(..))
    }

    fn is_none(&self) -> bool {
        matches!(self, OwnedMultiple::None(..))
    }

    fn parent_id(&self) -> u64 {
        match self {
            OwnedMultiple::Loaded(id, _) | OwnedMultiple::None(id) | OwnedMultiple::Unknown(id) => {
                *id
            }
        }
    }

    fn set_loaded(&mut self, value: Vec<T>) {
        let parent_id = self.parent_id();
        *self = OwnedMultiple::Loaded(parent_id, value);
    }

    fn set_none(&mut self) {
        let parent_id = self.parent_id();
        *self = OwnedMultiple::None(parent_id);
    }
}

impl<T> MultipleLink<T> for RefMultiple<T> {
    fn new_loaded(parent_id: u64, value: Vec<T>) -> Self {
        RefMultiple::Loaded(parent_id, value)
    }

    fn none(parent_id: u64) -> Self {
        RefMultiple::None(parent_id)
    }

    fn unknown(parent_id: u64) -> Self {
        RefMultiple::Unknown(parent_id)
    }

    fn get(&self) -> NodeResult<&Vec<T>> {
        match self {
            RefMultiple::Loaded(_, val) => Ok(val),
            _ => Err(NodeError::custom("RefMultiple link not loaded")),
        }
    }

    fn get_mut(&mut self) -> NodeResult<&mut Vec<T>> {
        match self {
            RefMultiple::Loaded(_, val) => Ok(val),
            RefMultiple::None(_) => {
                self.set_loaded(Vec::new());
                self.get_mut()
            }
            _ => Err(NodeError::custom("RefMultiple link not loaded")),
        }
    }

    fn take_loaded(&mut self) -> NodeResult<Vec<T>> {
        let parent_id = self.parent_id();
        match std::mem::replace(self, RefMultiple::Unknown(parent_id)) {
            RefMultiple::Loaded(_, val) => Ok(val),
            other => {
                *self = other;
                Err(NodeError::custom("RefMultiple link not loaded"))
            }
        }
    }

    fn is_loaded(&self) -> bool {
        matches!(self, RefMultiple::Loaded(..))
    }

    fn is_none(&self) -> bool {
        matches!(self, RefMultiple::None(..))
    }

    fn parent_id(&self) -> u64 {
        match self {
            RefMultiple::Loaded(id, _) | RefMultiple::None(id) | RefMultiple::Unknown(id) => *id,
            RefMultiple::_Phantom(_) => 0,
        }
    }

    fn set_loaded(&mut self, value: Vec<T>) {
        let parent_id = self.parent_id();
        *self = RefMultiple::Loaded(parent_id, value);
    }

    fn set_none(&mut self) {
        let parent_id = self.parent_id();
        *self = RefMultiple::None(parent_id);
    }
}

impl<T> Default for Component<T> {
    fn default() -> Self {
        Component::Unknown(0)
    }
}

impl<T> Default for Owned<T> {
    fn default() -> Self {
        Owned::Unknown(0)
    }
}

impl<T> Default for Ref<T> {
    fn default() -> Self {
        Ref::Unknown(0)
    }
}

impl<T> Default for OwnedMultiple<T> {
    fn default() -> Self {
        OwnedMultiple::Unknown(0)
    }
}

impl<T> Default for RefMultiple<T> {
    fn default() -> Self {
        RefMultiple::Unknown(0)
    }
}

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
        self.get().ok().into_iter()
    }
}

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
        self.get().ok().map(|vec| vec.iter()).into_iter().flatten()
    }
}
