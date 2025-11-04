use super::*;
use crate::stdb::TNodeLink;
mod sources;

pub use sources::*;

pub type ClientContext<'a> = Context<Sources<'a>>;

pub const EMPTY_CONTEXT: ClientContext = Context::new(Sources::None);

/// Resources for storing node data in World
#[derive(Resource, Default)]
pub struct NodesLinkResource {
    /// HashMap<parent_id, HashMap<NodeKind, Vec<child_id>>>
    pub children: HashMap<u64, HashMap<NodeKind, Vec<u64>>>,
    /// HashMap<child_id, HashMap<NodeKind, Vec<parent_id>>>
    pub parents: HashMap<u64, HashMap<NodeKind, Vec<u64>>>,
}

impl NodesLinkResource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_link(
        &mut self,
        parent_id: u64,
        child_id: u64,
        parent_kind: NodeKind,
        child_kind: NodeKind,
    ) {
        let children_vec = self
            .children
            .entry(parent_id)
            .or_default()
            .entry(child_kind)
            .or_default();
        if !children_vec.contains(&child_id) {
            children_vec.push(child_id);
        }

        let parents_vec = self
            .parents
            .entry(child_id)
            .or_default()
            .entry(parent_kind)
            .or_default();
        if !parents_vec.contains(&parent_id) {
            parents_vec.push(parent_id);
        }
    }

    pub fn remove_link(&mut self, parent_id: u64, child_id: u64) {
        if let Some(children_map) = self.children.get_mut(&parent_id) {
            for children in children_map.values_mut() {
                children.retain(|&id| id != child_id);
            }
        }

        if let Some(parents_map) = self.parents.get_mut(&child_id) {
            for parents in parents_map.values_mut() {
                parents.retain(|&id| id != parent_id);
            }
        }
    }

    pub fn get_children(&self, parent_id: u64) -> Vec<u64> {
        self.children
            .get(&parent_id)
            .map(|m| m.values().flatten().copied().collect())
            .unwrap_or_default()
    }

    pub fn get_children_of_kind(&self, parent_id: u64, kind: NodeKind) -> Vec<u64> {
        self.children
            .get(&parent_id)
            .and_then(|m| m.get(&kind))
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_parents(&self, child_id: u64) -> Vec<u64> {
        self.parents
            .get(&child_id)
            .map(|m| m.values().flatten().copied().collect())
            .unwrap_or_default()
    }

    pub fn get_parents_of_kind(&self, child_id: u64, kind: NodeKind) -> Vec<u64> {
        self.parents
            .get(&child_id)
            .and_then(|m| m.get(&kind))
            .cloned()
            .unwrap_or_default()
    }

    pub fn is_linked(&self, parent_id: u64, child_id: u64) -> bool {
        self.children
            .get(&parent_id)
            .map(|m| m.values().any(|children| children.contains(&child_id)))
            .unwrap_or(false)
    }

    pub fn clear_node(&mut self, node_id: u64) {
        // Remove as parent
        if let Some(children_map) = self.children.remove(&node_id) {
            for child_id in children_map.values().flatten() {
                if let Some(parents_map) = self.parents.get_mut(child_id) {
                    for parents in parents_map.values_mut() {
                        parents.retain(|&id| id != node_id);
                    }
                }
            }
        }

        // Remove as child
        if let Some(parents_map) = self.parents.remove(&node_id) {
            for parent_id in parents_map.values().flatten() {
                if let Some(children_map) = self.children.get_mut(parent_id) {
                    for children in children_map.values_mut() {
                        children.retain(|&id| id != node_id);
                    }
                }
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct NodesMapResource {
    /// Maps node ID to its kind
    pub kinds: HashMap<u64, NodeKind>,
    /// Maps node ID to its entity
    pub entities: HashMap<u64, Entity>,
    /// Maps entity to node IDs (one entity can have multiple nodes)
    pub entity_to_nodes: HashMap<Entity, Vec<u64>>,
}

impl NodesMapResource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, id: u64, kind: NodeKind, entity: Entity) {
        self.kinds.insert(id, kind);
        if let Some(entity) = self.entities.get(&id) {
            let ids = self.entity_to_nodes.get_mut(entity).unwrap();
            let pos = ids.iter().position(|i| *i == id).unwrap();
            ids.remove(pos);
        }
        self.entities.insert(id, entity);
        self.entity_to_nodes.entry(entity).or_default().push(id);
    }

    pub fn remove(&mut self, id: u64) -> Option<Entity> {
        self.kinds.remove(&id);
        if let Some(entity) = self.entities.remove(&id) {
            if let Some(nodes) = self.entity_to_nodes.get_mut(&entity) {
                nodes.retain(|&node_id| node_id != id);
                if nodes.is_empty() {
                    self.entity_to_nodes.remove(&entity);
                }
            }
            Some(entity)
        } else {
            None
        }
    }

    pub fn get_kind(&self, id: u64) -> Option<NodeKind> {
        self.kinds.get(&id).copied()
    }

    pub fn get_entity(&self, id: u64) -> Option<Entity> {
        self.entities.get(&id).copied()
    }

    pub fn get_node_ids(&self, entity: Entity) -> Vec<u64> {
        self.entity_to_nodes
            .get(&entity)
            .cloned()
            .unwrap_or_default()
    }
}

/// Resource for storing link ratings in Top source
#[derive(Resource, Default, Debug)]
pub struct LinkRatings {
    /// Maps (parent_id, child_kind) to Vec<(child_id, rating)>
    pub ratings: HashMap<(u64, NodeKind), Vec<(u64, i32)>>,
}

#[derive(Resource, Default, Debug)]
pub struct LinksMapResource {
    /// Maps link ID to TNodeLink
    pub links: HashMap<u64, TNodeLink>,
}

impl LinksMapResource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, link: TNodeLink) {
        self.links.insert(link.id, link);
    }

    pub fn remove(&mut self, link_id: u64) {
        self.links.remove(&link_id);
    }

    pub fn get(&self, link_id: u64) -> Option<&TNodeLink> {
        self.links.get(&link_id)
    }
}

impl LinkRatings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_rating(&mut self, parent_id: u64, child_id: u64, child_kind: NodeKind, rating: i32) {
        let ratings = self.ratings.entry((parent_id, child_kind)).or_default();

        // Keep sorted by rating (highest first)
        match ratings.binary_search_by(|&(_, r)| rating.cmp(&r)) {
            Ok(pos) | Err(pos) => ratings.insert(pos, (child_id, rating)),
        }
    }

    pub fn remove_rating(&mut self, parent_id: u64, child_id: u64, child_kind: NodeKind) {
        if let Some(ratings) = self.ratings.get_mut(&(parent_id, child_kind)) {
            ratings.retain(|&(id, _)| id != child_id);
        }
    }

    pub fn get_top(&self, parent_id: u64, child_kind: NodeKind) -> Option<u64> {
        self.ratings
            .get(&(parent_id, child_kind))
            .and_then(|ratings| ratings.first())
            .map(|&(id, _)| id)
    }
}

impl ClientSource for ClientContext<'_> {
    fn insert_node<T: ClientNode + BevyComponent>(&mut self, node: &T) -> NodeResult<()> {
        self.source_mut().insert_node(node)
    }

    fn world(&self) -> NodeResult<&World> {
        self.source().world()
    }

    fn world_mut(&mut self) -> NodeResult<&mut World> {
        self.source_mut().world_mut()
    }

    fn entity(&self, node_id: u64) -> NodeResult<Entity> {
        self.source().entity(node_id)
    }

    fn load_ref<T: ClientNode>(&self, node_id: u64) -> NodeResult<&T> {
        self.source().load_ref(node_id).track()
    }

    fn load<T: ClientNode + Clone>(&self, node_id: u64) -> NodeResult<T> {
        self.source().load(node_id).track()
    }

    fn battle(&self) -> NodeResult<&BattleSimulation> {
        self.source().battle()
    }

    fn battle_mut(&mut self) -> NodeResult<&mut BattleSimulation> {
        self.source_mut().battle_mut()
    }

    fn rng(&mut self) -> NodeResult<&mut ChaCha8Rng> {
        self.source_mut().rng()
    }

    fn t(&self) -> Option<f32> {
        self.source().t()
    }

    fn t_mut(&mut self) -> Option<&mut f32> {
        self.source_mut().t_mut()
    }
}

/// Extension methods for ClientContext
pub trait ClientContextExt<'a> {
    fn load_mut<T: ClientNode + BevyComponent<Mutability = Mutable>>(
        &mut self,
        node_id: u64,
    ) -> NodeResult<Mut<'_, T>>;
    fn load_many_ref<T: ClientNode>(&self, ids: &[u64]) -> NodeResult<Vec<&T>>;
    fn load_children_ref<T: ClientNode>(&self, id: u64) -> NodeResult<Vec<&T>>;
    fn load_first_parent_recursive_ref<T: ClientNode>(&self, id: u64) -> NodeResult<&T>;
    fn load_first_parent_ref<T: ClientNode>(&self, id: u64) -> NodeResult<&T>;
    fn add_id_entity_link(
        &mut self,
        kind: NodeKind,
        node_id: u64,
        entity: Entity,
    ) -> NodeResult<()>;
    fn despawn(&mut self, node_id: u64) -> NodeResult<()>;

    fn exec_ref<F, R>(&'a self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>;
    fn color(&self) -> Color32;

    fn into_source(self) -> Sources<'a>;
    fn exec_mut<F, R>(&mut self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>;
}

impl<'a> ClientContextExt<'a> for ClientContext<'a> {
    fn load_mut<T: ClientNode + BevyComponent<Mutability = Mutable>>(
        &mut self,
        node_id: u64,
    ) -> NodeResult<Mut<'_, T>> {
        let entity = self.entity(node_id)?;
        self.world_mut()?
            .get_mut::<T>(entity)
            .ok_or_else(|| NodeError::not_found(node_id))
    }

    fn load_many_ref<T: ClientNode>(&self, ids: &[u64]) -> NodeResult<Vec<&T>> {
        ids.iter().map(|&id| self.load_ref::<T>(id)).collect()
    }

    fn load_children_ref<T: ClientNode>(&self, id: u64) -> NodeResult<Vec<&T>> {
        let ids = self.get_children_of_kind(id, T::kind_s())?;
        self.load_many_ref(&ids)
    }

    fn load_first_parent_ref<T: ClientNode>(&self, id: u64) -> NodeResult<&T> {
        let id = self.first_parent(id, T::kind_s())?;
        self.load_ref(id)
    }

    fn load_first_parent_recursive_ref<T: ClientNode>(&self, id: u64) -> NodeResult<&T> {
        let id = self.first_parent_recursive(id, T::kind_s())?;
        self.load_ref(id)
    }

    fn add_id_entity_link(
        &mut self,
        kind: NodeKind,
        node_id: u64,
        entity: Entity,
    ) -> NodeResult<()> {
        if let Ok(world) = self.world_mut() {
            if let Some(mut node_data) = world.get_resource_mut::<NodesMapResource>() {
                node_data.insert(node_id, kind, entity);
            }
        }
        Ok(())
    }

    fn despawn(&mut self, node_id: u64) -> NodeResult<()> {
        self.source_mut().delete_node(node_id)
    }

    fn exec_ref<F, R>(&'a self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>,
    {
        let layers = self.layers();
        let mut ctx = ClientContext::new(Sources::SourceRef(self.source()));
        ctx.with_layers(layers.clone(), f)
    }

    fn color(&self) -> Color32 {
        if let Ok(c) = self.get_var(VarName::color).and_then(|c| c.get_color()) {
            c
        } else {
            colorix().low_contrast_text()
        }
    }

    fn into_source(self) -> Sources<'a> {
        self.into_inner()
    }

    fn exec_mut<F, R>(&mut self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>,
    {
        f(self)
    }
}

/// Extension for creating Sources from World
pub trait WorldExt {
    fn to_solid_source(self) -> Sources<'static>;
}

impl WorldExt for World {
    fn to_solid_source(self) -> Sources<'static> {
        Sources::Solid(self)
    }
}

/// Component for tracking nodes attached to an entity
#[derive(BevyComponent, Debug, Default)]
pub struct NodeEntityComponent {
    pub nodes: HashMap<NodeKind, u64>,
}

impl NodeEntityComponent {
    pub fn new(id: u64, kind: NodeKind) -> Self {
        Self {
            nodes: HashMap::from_iter(vec![(kind, id)]),
        }
    }

    pub fn add_node(&mut self, id: u64, kind: NodeKind) {
        self.nodes.insert(kind, id);
    }

    pub fn ids(&self) -> HashSet<u64> {
        self.nodes.values().copied().collect()
    }

    pub fn get_node_ids(&self) -> Vec<u64> {
        self.nodes.values().copied().collect()
    }
}
