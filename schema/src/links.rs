use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Component<T> {
    Loaded { parent_id: u64, data: T },
    None { parent_id: u64 },
    Unknown { parent_id: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Owned<T> {
    Loaded { parent_id: u64, data: T },
    None { parent_id: u64 },
    Unknown { parent_id: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnedMultiple<T> {
    Loaded { parent_id: u64, data: Vec<T> },
    None { parent_id: u64 },
    Unknown { parent_id: u64 },
}

pub trait SingleLink<T: Node> {
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
    fn set_parent_id(&mut self, parent_id: u64);
}

pub trait MultipleLink<T: Node> {
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
    fn set_parent_id(&mut self, parent_id: u64);

    fn push(&mut self, node: T) -> NodeResult<&mut T> {
        match self {
            _ if self.is_loaded() => {
                self.get_mut()?.push(node);
                Ok(self.get_mut()?.last_mut().unwrap())
            }
            _ if self.is_none() => {
                self.set_loaded(vec![node]);
                Ok(self.get_mut()?.first_mut().unwrap())
            }
            _ => Err(NodeError::invalid_state(
                "Cannot push to link that is not Loaded or None",
            )),
        }
    }
}

impl<T: Node> SingleLink<T> for Component<T> {
    fn new_loaded(parent_id: u64, data: T) -> Self {
        Component::Loaded { parent_id, data }
    }

    fn none(parent_id: u64) -> Self {
        Component::None { parent_id }
    }

    fn unknown(parent_id: u64) -> Self {
        Component::Unknown { parent_id }
    }

    fn get(&self) -> NodeResult<&T> {
        match self {
            Component::Loaded { data, .. } => Ok(data),
            Component::None { .. } => Err(NodeError::not_in_context("Component link")),
            _ => Err(NodeError::not_in_context("Component link")),
        }
    }

    fn get_mut(&mut self) -> NodeResult<&mut T> {
        match self {
            Component::Loaded { data, .. } => Ok(data),
            Component::None { .. } => Err(NodeError::not_in_context("Component link")),
            _ => Err(NodeError::not_in_context("Component link")),
        }
    }

    fn take_loaded(&mut self) -> NodeResult<T> {
        let parent_id = self.parent_id();
        match std::mem::replace(self, Component::Unknown { parent_id }) {
            Component::Loaded { data, .. } => Ok(data),
            other => {
                *self = other;
                Err(NodeError::not_in_context("Component link"))
            }
        }
    }

    fn is_loaded(&self) -> bool {
        matches!(self, Component::Loaded { .. })
    }

    fn is_none(&self) -> bool {
        matches!(self, Component::None { .. })
    }

    fn parent_id(&self) -> u64 {
        match self {
            Component::Loaded { parent_id, .. }
            | Component::None { parent_id }
            | Component::Unknown { parent_id } => *parent_id,
        }
    }

    fn set_parent_id(&mut self, new_parent_id: u64) {
        match self {
            Self::Loaded { parent_id, .. }
            | Self::None { parent_id }
            | Self::Unknown { parent_id } => *parent_id = new_parent_id,
        }
    }

    fn set_loaded(&mut self, data: T) {
        let parent_id = self.parent_id();
        *self = Component::Loaded { parent_id, data };
    }

    fn set_none(&mut self) {
        let parent_id = self.parent_id();
        *self = Component::None { parent_id };
    }
}

impl<T: Node> SingleLink<T> for Owned<T> {
    fn new_loaded(parent_id: u64, data: T) -> Self {
        Owned::Loaded { parent_id, data }
    }

    fn none(parent_id: u64) -> Self {
        Owned::None { parent_id }
    }

    fn unknown(parent_id: u64) -> Self {
        Owned::Unknown { parent_id }
    }

    fn get(&self) -> NodeResult<&T> {
        match self {
            Owned::Loaded { data, .. } => Ok(data),
            Owned::None { .. } => Err(NodeError::not_in_context("Owned link")),
            _ => Err(NodeError::not_in_context("Owned link")),
        }
    }

    fn get_mut(&mut self) -> NodeResult<&mut T> {
        match self {
            Owned::Loaded { data, .. } => Ok(data),
            Owned::None { .. } => Err(NodeError::not_in_context("Owned link")),
            _ => Err(NodeError::not_in_context("Owned link")),
        }
    }

    fn take_loaded(&mut self) -> NodeResult<T> {
        let parent_id = self.parent_id();
        match std::mem::replace(self, Owned::Unknown { parent_id }) {
            Owned::Loaded { data, .. } => Ok(data),
            other => {
                *self = other;
                Err(NodeError::not_in_context("Owned link"))
            }
        }
    }

    fn is_loaded(&self) -> bool {
        matches!(self, Owned::Loaded { .. })
    }

    fn is_none(&self) -> bool {
        matches!(self, Owned::None { .. })
    }

    fn parent_id(&self) -> u64 {
        match self {
            Owned::Loaded { parent_id, .. }
            | Owned::None { parent_id }
            | Owned::Unknown { parent_id } => *parent_id,
        }
    }

    fn set_parent_id(&mut self, new_parent_id: u64) {
        match self {
            Self::Loaded { parent_id, .. }
            | Self::None { parent_id }
            | Self::Unknown { parent_id } => *parent_id = new_parent_id,
        }
    }

    fn set_loaded(&mut self, data: T) {
        let parent_id = self.parent_id();
        *self = Owned::Loaded { parent_id, data };
    }

    fn set_none(&mut self) {
        let parent_id = self.parent_id();
        *self = Owned::None { parent_id };
    }
}

impl<T: Node> MultipleLink<T> for OwnedMultiple<T> {
    fn new_loaded(parent_id: u64, data: Vec<T>) -> Self {
        OwnedMultiple::Loaded { parent_id, data }
    }

    fn none(parent_id: u64) -> Self {
        OwnedMultiple::None { parent_id }
    }

    fn unknown(parent_id: u64) -> Self {
        OwnedMultiple::Unknown { parent_id }
    }

    fn get(&self) -> NodeResult<&Vec<T>> {
        match self {
            OwnedMultiple::Loaded { data, .. } => Ok(data),
            _ => Err(NodeError::not_in_context("OwnedMultiple link")),
        }
    }

    fn get_mut(&mut self) -> NodeResult<&mut Vec<T>> {
        match self {
            OwnedMultiple::Loaded { data, .. } => Ok(data),
            OwnedMultiple::None { .. } => {
                self.set_loaded(Vec::new());
                self.get_mut()
            }
            _ => Err(NodeError::not_in_context("OwnedMultiple link")),
        }
    }

    fn take_loaded(&mut self) -> NodeResult<Vec<T>> {
        let parent_id = self.parent_id();
        match std::mem::replace(self, OwnedMultiple::Unknown { parent_id }) {
            OwnedMultiple::Loaded { data, .. } => Ok(data),
            other => {
                *self = other;
                Err(NodeError::not_in_context("OwnedMultiple link"))
            }
        }
    }

    fn is_loaded(&self) -> bool {
        matches!(self, OwnedMultiple::Loaded { .. })
    }

    fn is_none(&self) -> bool {
        matches!(self, OwnedMultiple::None { .. })
    }

    fn parent_id(&self) -> u64 {
        match self {
            OwnedMultiple::Loaded { parent_id, .. }
            | OwnedMultiple::None { parent_id }
            | OwnedMultiple::Unknown { parent_id } => *parent_id,
        }
    }

    fn set_parent_id(&mut self, new_parent_id: u64) {
        match self {
            Self::Loaded { parent_id, .. }
            | Self::None { parent_id }
            | Self::Unknown { parent_id } => *parent_id = new_parent_id,
        }
    }

    fn set_loaded(&mut self, data: Vec<T>) {
        let parent_id = self.parent_id();
        *self = OwnedMultiple::Loaded { parent_id, data };
    }

    fn set_none(&mut self) {
        let parent_id = self.parent_id();
        *self = OwnedMultiple::None { parent_id };
    }
}

impl<T> Default for OwnedMultiple<T> {
    fn default() -> Self {
        OwnedMultiple::Unknown { parent_id: 0 }
    }
}
impl<T> Default for Component<T> {
    fn default() -> Self {
        Component::Unknown { parent_id: 0 }
    }
}

impl<T> Default for Owned<T> {
    fn default() -> Self {
        Owned::Unknown { parent_id: 0 }
    }
}
