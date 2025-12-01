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
pub enum Ref<T> {
    Id {
        parent_id: u64,
        node_id: u64,
    },
    None {
        parent_id: u64,
    },
    Unknown {
        parent_id: u64,
    },
    #[serde(skip)]
    _Phantom(std::marker::PhantomData<T>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnedMultiple<T> {
    Loaded { parent_id: u64, data: Vec<T> },
    None { parent_id: u64 },
    Unknown { parent_id: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefMultiple<T> {
    Ids {
        parent_id: u64,
        node_ids: Vec<u64>,
    },
    None {
        parent_id: u64,
    },
    Unknown {
        parent_id: u64,
    },
    #[serde(skip)]
    _Phantom(std::marker::PhantomData<T>),
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
    fn id(&self) -> NodeResult<u64>;
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
    fn ids(&self) -> NodeResult<&Vec<u64>>;

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
            _ => Err(NodeError::custom(
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
            _ => Err(NodeError::custom("Component link not loaded")),
        }
    }

    fn get_mut(&mut self) -> NodeResult<&mut T> {
        match self {
            Component::Loaded { data, .. } => Ok(data),
            _ => Err(NodeError::custom("Component link not loaded")),
        }
    }

    fn take_loaded(&mut self) -> NodeResult<T> {
        let parent_id = self.parent_id();
        match std::mem::replace(self, Component::Unknown { parent_id }) {
            Component::Loaded { data, .. } => Ok(data),
            other => {
                *self = other;
                Err(NodeError::custom("Component link not loaded"))
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

    fn id(&self) -> NodeResult<u64> {
        match self {
            Component::Loaded { parent_id, .. } => Ok(*parent_id),
            _ => Err(NodeError::custom("Component link not loaded")),
        }
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
            _ => Err(NodeError::custom("Owned link not loaded")),
        }
    }

    fn get_mut(&mut self) -> NodeResult<&mut T> {
        match self {
            Owned::Loaded { data, .. } => Ok(data),
            _ => Err(NodeError::custom("Owned link not loaded")),
        }
    }

    fn take_loaded(&mut self) -> NodeResult<T> {
        let parent_id = self.parent_id();
        match std::mem::replace(self, Owned::Unknown { parent_id }) {
            Owned::Loaded { data, .. } => Ok(data),
            other => {
                *self = other;
                Err(NodeError::custom("Owned link not loaded"))
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

    fn id(&self) -> NodeResult<u64> {
        match self {
            Owned::Loaded { parent_id, .. } => Ok(*parent_id),
            _ => Err(NodeError::custom("Owned link not loaded")),
        }
    }
}

impl<T: Node> SingleLink<T> for Ref<T> {
    fn new_loaded(parent_id: u64, _: T) -> Self {
        Ref::Id {
            parent_id,
            node_id: 0,
        }
    }

    fn none(parent_id: u64) -> Self {
        Ref::None { parent_id }
    }

    fn unknown(parent_id: u64) -> Self {
        Ref::Unknown { parent_id }
    }

    fn get(&self) -> NodeResult<&T> {
        Err(NodeError::custom(
            "Ref link stores only id, use id() method",
        ))
    }

    fn get_mut(&mut self) -> NodeResult<&mut T> {
        Err(NodeError::custom(
            "Ref link stores only id, use id() method",
        ))
    }

    fn take_loaded(&mut self) -> NodeResult<T> {
        Err(NodeError::custom(
            "Ref link stores only id, use id() method",
        ))
    }

    fn is_loaded(&self) -> bool {
        matches!(self, Ref::Id { .. })
    }

    fn is_none(&self) -> bool {
        matches!(self, Ref::None { .. })
    }

    fn parent_id(&self) -> u64 {
        match self {
            Ref::Id { parent_id, .. } | Ref::None { parent_id } | Ref::Unknown { parent_id } => {
                *parent_id
            }
            Ref::_Phantom(_) => 0,
        }
    }

    fn set_parent_id(&mut self, new_parent_id: u64) {
        match self {
            Self::Id { parent_id, .. } | Self::None { parent_id } | Self::Unknown { parent_id } => {
                *parent_id = new_parent_id
            }
            Ref::_Phantom(..) => {}
        }
    }

    fn set_loaded(&mut self, _: T) {
        let parent_id = self.parent_id();
        *self = Ref::Id {
            parent_id,
            node_id: 0,
        };
    }

    fn set_none(&mut self) {
        let parent_id = self.parent_id();
        *self = Ref::None { parent_id };
    }

    fn id(&self) -> NodeResult<u64> {
        match self {
            Ref::Id { node_id, .. } => Ok(*node_id),
            _ => Err(NodeError::custom("Ref link not loaded")),
        }
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
            _ => Err(NodeError::custom("OwnedMultiple link not loaded")),
        }
    }

    fn get_mut(&mut self) -> NodeResult<&mut Vec<T>> {
        match self {
            OwnedMultiple::Loaded { data, .. } => Ok(data),
            OwnedMultiple::None { .. } => {
                self.set_loaded(Vec::new());
                self.get_mut()
            }
            _ => Err(NodeError::custom("OwnedMultiple link not loaded")),
        }
    }

    fn take_loaded(&mut self) -> NodeResult<Vec<T>> {
        let parent_id = self.parent_id();
        match std::mem::replace(self, OwnedMultiple::Unknown { parent_id }) {
            OwnedMultiple::Loaded { data, .. } => Ok(data),
            other => {
                *self = other;
                Err(NodeError::custom("OwnedMultiple link not loaded"))
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

    fn ids(&self) -> NodeResult<&Vec<u64>> {
        Err(NodeError::custom(
            "OwnedMultiple link stores loaded values, not ids",
        ))
    }
}

impl<T: Node> MultipleLink<T> for RefMultiple<T> {
    fn new_loaded(parent_id: u64, nodes: Vec<T>) -> Self {
        RefMultiple::Ids {
            parent_id,
            node_ids: nodes.into_iter().map(|n| n.id()).collect(),
        }
    }

    fn none(parent_id: u64) -> Self {
        RefMultiple::None { parent_id }
    }

    fn unknown(parent_id: u64) -> Self {
        RefMultiple::Unknown { parent_id }
    }

    fn get(&self) -> NodeResult<&Vec<T>> {
        Err(NodeError::custom(
            "RefMultiple link stores only ids, use ids() method",
        ))
    }

    fn get_mut(&mut self) -> NodeResult<&mut Vec<T>> {
        Err(NodeError::custom(
            "RefMultiple link stores only ids, use ids() method",
        ))
    }

    fn take_loaded(&mut self) -> NodeResult<Vec<T>> {
        Err(NodeError::custom(
            "RefMultiple link stores only ids, use ids() method",
        ))
    }

    fn is_loaded(&self) -> bool {
        matches!(self, RefMultiple::Ids { .. })
    }

    fn is_none(&self) -> bool {
        matches!(self, RefMultiple::None { .. })
    }

    fn parent_id(&self) -> u64 {
        match self {
            RefMultiple::Ids { parent_id, .. }
            | RefMultiple::None { parent_id }
            | RefMultiple::Unknown { parent_id } => *parent_id,
            RefMultiple::_Phantom(_) => 0,
        }
    }

    fn set_parent_id(&mut self, new_parent_id: u64) {
        match self {
            Self::Ids { parent_id, .. }
            | Self::None { parent_id }
            | Self::Unknown { parent_id } => *parent_id = new_parent_id,
            RefMultiple::_Phantom(..) => {}
        }
    }

    fn set_loaded(&mut self, nodes: Vec<T>) {
        let parent_id = self.parent_id();
        *self = RefMultiple::Ids {
            parent_id,
            node_ids: nodes.iter().map(|n| n.id()).collect(),
        };
    }

    fn set_none(&mut self) {
        let parent_id = self.parent_id();
        *self = RefMultiple::None { parent_id };
    }

    fn ids(&self) -> NodeResult<&Vec<u64>> {
        match self {
            RefMultiple::Ids { node_ids, .. } => Ok(node_ids),
            _ => Err(NodeError::custom("RefMultiple link not loaded")),
        }
    }
}

impl<T: Node> Ref<T> {
    pub fn set_id(&mut self, id: u64) {
        *self = Ref::Id {
            parent_id: self.parent_id(),
            node_id: id,
        };
    }

    pub fn load_id<C: ContextSource>(&mut self, ctx: &Context<C>) -> NodeResult<u64> {
        let node_id = ctx.first_child(self.parent_id(), T::kind_s())?;
        *self = Ref::Id {
            parent_id: self.parent_id(),
            node_id,
        };
        Ok(node_id)
    }
}

impl<T: Node> RefMultiple<T> {
    pub fn set_ids(&mut self, ids: Vec<u64>) {
        let parent_id = self.parent_id();
        *self = RefMultiple::Ids {
            parent_id,
            node_ids: ids,
        };
    }
    pub fn push_id(&mut self, id: u64) -> NodeResult<()> {
        match self {
            RefMultiple::Ids { node_ids, .. } => node_ids.push(id),
            RefMultiple::None { parent_id } => {
                *self = RefMultiple::Ids {
                    parent_id: *parent_id,
                    node_ids: vec![id],
                }
            }
            _ => {
                return Err(NodeError::custom("RefMultiple link not loaded"));
            }
        };
        Ok(())
    }

    pub fn load_ids<C: ContextSource>(&mut self, ctx: &Context<C>) -> NodeResult<Vec<u64>> {
        let node_ids = ctx.collect_kind_children(self.parent_id(), T::kind_s())?;
        *self = RefMultiple::Ids {
            parent_id: self.parent_id(),
            node_ids: node_ids.clone(),
        };
        Ok(node_ids)
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

impl<T> Default for Ref<T> {
    fn default() -> Self {
        Ref::Unknown { parent_id: 0 }
    }
}

impl<T> Default for OwnedMultiple<T> {
    fn default() -> Self {
        OwnedMultiple::Unknown { parent_id: 0 }
    }
}

impl<T> Default for RefMultiple<T> {
    fn default() -> Self {
        RefMultiple::Unknown { parent_id: 0 }
    }
}
