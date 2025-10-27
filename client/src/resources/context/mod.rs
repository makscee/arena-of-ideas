use super::*;
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
        self.children
            .entry(parent_id)
            .or_default()
            .entry(child_kind)
            .or_default()
            .push(child_id);

        self.parents
            .entry(child_id)
            .or_default()
            .entry(parent_kind)
            .or_default()
            .push(parent_id);
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
#[derive(Resource, Default)]
pub struct LinkRatings {
    /// Maps (parent_id, child_kind) to Vec<(child_id, rating)>
    pub ratings: HashMap<(u64, NodeKind), Vec<(u64, i32)>>,
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
    fn insert_node<T: ClientNode + BevyComponent>(&mut self, node: T) -> NodeResult<()> {
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
        self.source().load_ref(node_id)
    }

    fn load<T: ClientNode + Clone>(&self, node_id: u64) -> NodeResult<T> {
        self.source().load(node_id)
    }

    fn battle(&self) -> NodeResult<&BattleSimulation> {
        self.source().battle()
    }

    fn rng(&mut self) -> NodeResult<&mut ChaCha8Rng> {
        self.source_mut().rng()
    }
}

/// Extension methods for ClientContext
pub trait ClientContextExt<'a> {
    fn battle_mut(&mut self) -> NodeResult<&mut BattleSimulation>;
    fn load_mut<T: ClientNode + BevyComponent<Mutability = Mutable>>(
        &mut self,
        node_id: u64,
    ) -> NodeResult<Mut<T>>;
    fn load_many_ref<T: ClientNode>(&self, ids: &[u64]) -> NodeResult<Vec<&T>>;
    fn load_children_ref<T: ClientNode>(&self, id: u64) -> NodeResult<Vec<&T>>;
    fn load_first_parent_recursive_ref<T: ClientNode>(&self, id: u64) -> NodeResult<&T>;
    fn load_first_parent_ref<T: ClientNode>(&self, id: u64) -> NodeResult<&T>;
    fn add_id_entity_link(&mut self, node_id: u64, entity: Entity) -> NodeResult<()>;
    fn t(&self) -> NodeResult<f32>;
    fn despawn(&mut self, node_id: u64) -> NodeResult<()>;

    fn exec_ref<F, R>(&'a self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>;
    fn color(&self, ui: &mut Ui) -> Color32;
}

impl<'a> ClientContextExt<'a> for ClientContext<'a> {
    fn battle_mut(&mut self) -> NodeResult<&mut BattleSimulation> {
        match self.source_mut() {
            Sources::Battle(world) => world
                .get_resource_mut::<BattleSimulation>()
                .ok_or_else(|| NodeError::custom("BattleSimulation resource not found")),
            _ => Err(NodeError::custom("Not a battle source")),
        }
    }

    fn load_mut<T: ClientNode + BevyComponent<Mutability = Mutable>>(
        &mut self,
        node_id: u64,
    ) -> NodeResult<Mut<T>> {
        let entity = self.entity(node_id)?;
        self.world_mut()?
            .get_mut::<T>(entity)
            .ok_or_else(|| NodeError::not_found(node_id))
    }

    fn t(&self) -> NodeResult<f32> {
        self.battle().map(|sim| sim.duration)
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

    fn add_id_entity_link(&mut self, node_id: u64, entity: Entity) -> NodeResult<()> {
        let kind = self.source().get_node_kind(node_id)?;
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
        let mut ctx = ClientContext::new(Sources::ContextRef(self));
        ctx.with_layers(layers.clone(), f)
    }

    fn color(&self, ui: &mut Ui) -> Color32 {
        if let Ok(c) = self.get_var(VarName::color).and_then(|c| c.get_color()) {
            c
        } else {
            ui.visuals().text_color()
        }
    }
}

/// Extension for using Context with Bevy World
pub trait WorldContextExt {
    fn with_context<R, F>(&self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>;
    fn with_context_mut<R, F>(&mut self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>;
    fn as_context_mut(&mut self) -> ClientContext;
}

impl WorldContextExt for World {
    fn with_context<R, F>(&self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        // Check if this is a battle world by looking for BattleSimulation resource
        if self.get_resource::<BattleSimulation>().is_some() {
            // Create a world copy for battle context since we need mut access
            let mut battle_world = World::new();
            battle_world.init_resource::<NodesMapResource>();
            battle_world.init_resource::<NodesLinkResource>();
            Context::exec(Sources::Battle(battle_world), f)
        } else {
            let mut solid_world = World::new();
            solid_world.init_resource::<NodesMapResource>();
            solid_world.init_resource::<NodesLinkResource>();
            Context::exec(Sources::Solid(solid_world), f)
        }
    }

    fn with_context_mut<R, F>(&mut self, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut ClientContext) -> NodeResult<R>,
    {
        // Check if this is a battle world by looking for BattleSimulation resource
        if self.get_resource::<BattleSimulation>().is_some() {
            // Move the world temporarily to create the context
            let world = std::mem::replace(self, World::new());
            let source = Sources::Battle(world);
            let result = Context::exec(source, f);

            // Move the world back
            if let Sources::Battle(world) = source {
                *self = world;
            }
            result
        } else {
            // For non-battle worlds, create a proper solid context
            let world = std::mem::replace(self, World::new());
            let mut source = Sources::Solid(world);
            let result = Context::exec(source, f);

            // Move the world back
            if let Sources::Solid(world) = source {
                *self = world;
            }
            result
        }
    }

    fn as_context_mut(&mut self) -> ClientContext {
        // Check if this is a battle world by looking for BattleSimulation resource
        if self.get_resource::<BattleSimulation>().is_some() {
            let world = std::mem::replace(self, World::new());
            Context::new(Sources::Battle(world))
        } else {
            let world = std::mem::replace(self, World::new());
            Context::new(Sources::Solid(world))
        }
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
