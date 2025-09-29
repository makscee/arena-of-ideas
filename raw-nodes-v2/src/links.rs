use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkState<T> {
    Loaded(Box<T>),
    Id(u64),
    None,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component<T> {
    state: LinkState<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owned<T> {
    state: LinkState<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ref<T> {
    state: LinkState<T>,
}

impl<T> Component<T> {
    pub fn new_loaded(value: T) -> Self {
        Self {
            state: LinkState::Loaded(Box::new(value)),
        }
    }

    pub fn new_id(id: u64) -> Self {
        Self {
            state: LinkState::Id(id),
        }
    }

    pub fn none() -> Self {
        Self {
            state: LinkState::None,
        }
    }

    pub fn unknown() -> Self {
        Self {
            state: LinkState::Unknown,
        }
    }

    pub fn get(&self) -> Option<&T> {
        match &self.state {
            LinkState::Loaded(val) => Some(val),
            _ => None,
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        match &mut self.state {
            LinkState::Loaded(val) => Some(val),
            _ => None,
        }
    }

    pub fn id(&self) -> Option<u64> {
        match &self.state {
            LinkState::Id(id) => Some(*id),
            _ => None,
        }
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self.state, LinkState::Loaded(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self.state, LinkState::None)
    }

    pub fn state(&self) -> &LinkState<T> {
        &self.state
    }
}

impl<T> Owned<T> {
    pub fn new_loaded(value: T) -> Self {
        Self {
            state: LinkState::Loaded(Box::new(value)),
        }
    }

    pub fn new_id(id: u64) -> Self {
        Self {
            state: LinkState::Id(id),
        }
    }

    pub fn none() -> Self {
        Self {
            state: LinkState::None,
        }
    }

    pub fn unknown() -> Self {
        Self {
            state: LinkState::Unknown,
        }
    }

    pub fn get(&self) -> Option<&T> {
        match &self.state {
            LinkState::Loaded(val) => Some(val),
            _ => None,
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        match &mut self.state {
            LinkState::Loaded(val) => Some(val),
            _ => None,
        }
    }

    pub fn id(&self) -> Option<u64> {
        match &self.state {
            LinkState::Id(id) => Some(*id),
            _ => None,
        }
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self.state, LinkState::Loaded(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self.state, LinkState::None)
    }

    pub fn state(&self) -> &LinkState<T> {
        &self.state
    }
}

impl<T> Ref<T> {
    pub fn new_loaded(value: T) -> Self {
        Self {
            state: LinkState::Loaded(Box::new(value)),
        }
    }

    pub fn new_id(id: u64) -> Self {
        Self {
            state: LinkState::Id(id),
        }
    }

    pub fn none() -> Self {
        Self {
            state: LinkState::None,
        }
    }

    pub fn unknown() -> Self {
        Self {
            state: LinkState::Unknown,
        }
    }

    pub fn get(&self) -> Option<&T> {
        match &self.state {
            LinkState::Loaded(val) => Some(val),
            _ => None,
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        match &mut self.state {
            LinkState::Loaded(val) => Some(val),
            _ => None,
        }
    }

    pub fn id(&self) -> Option<u64> {
        match &self.state {
            LinkState::Id(id) => Some(*id),
            _ => None,
        }
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self.state, LinkState::Loaded(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self.state, LinkState::None)
    }

    pub fn state(&self) -> &LinkState<T> {
        &self.state
    }
}

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
