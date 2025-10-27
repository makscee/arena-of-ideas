use super::*;

/// Static sources for mirroring stdb state
pub struct StaticSources {
    pub top: Sources<'static>,
    pub selected: Sources<'static>,
    pub solid: Sources<'static>,
}

impl StaticSources {
    pub fn new() -> Self {
        Self {
            top: Sources::new_top(),
            selected: Sources::new_selected(),
            solid: Sources::new_solid(),
        }
    }
}

static mut STATIC_SOURCES: Option<StaticSources> = None;

pub fn init_static_sources() {
    unsafe {
        STATIC_SOURCES = Some(StaticSources::new());
    }
}

pub fn with_static_sources<R, F>(f: F) -> R
where
    F: FnOnce(&mut StaticSources) -> R,
{
    unsafe {
        if let Some(ref mut sources) = STATIC_SOURCES {
            f(sources)
        } else {
            panic!("Static sources not initialized");
        }
    }
}

/// Trait for client-side node data sources
pub trait ClientSource {
    fn insert_node<T: ClientNode + BevyComponent>(&mut self, node: T) -> NodeResult<()>;
    fn world(&self) -> NodeResult<&World>;
    fn world_mut(&mut self) -> NodeResult<&mut World>;
    fn entity(&self, node_id: u64) -> NodeResult<Entity>;
    fn load_ref<T: ClientNode>(&self, node_id: u64) -> NodeResult<&T>;
    fn load<T: ClientNode + Clone>(&self, node_id: u64) -> NodeResult<T>;
    fn battle(&self) -> NodeResult<&BattleSimulation>;
    fn rng(&mut self) -> NodeResult<&mut ChaCha8Rng>;
}

/// Sources enum for different node data sources
pub enum Sources<'a> {
    Solid(World),
    Top(World),
    Selected(World),
    Battle(World),
    ContextRef(&'a ClientContext<'a>),
    None,
}

impl Sources<'_> {
    pub fn new_solid() -> Self {
        let mut world = World::new();
        Self::init_world(&mut world);
        Sources::Solid(world)
    }

    pub fn new_top() -> Self {
        let mut world = World::new();
        Self::init_world(&mut world);
        world.init_resource::<LinkRatings>();
        Sources::Top(world)
    }

    pub fn new_selected() -> Self {
        let mut world = World::new();
        Self::init_world(&mut world);
        Sources::Selected(world)
    }

    pub fn new_battle(world: World) -> Self {
        Sources::Battle(world)
    }

    /// Unified initialization function for all World sources
    fn init_world(world: &mut World) {
        world.init_resource::<NodesMapResource>();
        world.init_resource::<NodesLinkResource>();
    }

    pub fn world(&self) -> NodeResult<&World> {
        match self {
            Sources::Solid(w) | Sources::Top(w) | Sources::Selected(w) | Sources::Battle(w) => {
                Ok(w)
            }
            Sources::ContextRef(context) => context.world(),
            Sources::None => Err(NodeError::custom("No world in None source")),
        }
    }

    pub fn world_mut(&mut self) -> NodeResult<&mut World> {
        match self {
            Sources::Solid(w) | Sources::Top(w) | Sources::Selected(w) | Sources::Battle(w) => {
                Ok(w)
            }
            Sources::ContextRef(_) => {
                Err(NodeError::custom("Can't mutate World of ContextRef source"))
            }
            Sources::None => Err(NodeError::custom("No world in None source")),
        }
    }

    pub fn get_nodes_map(&self) -> NodeResult<&NodesMapResource> {
        self.world()?
            .get_resource::<NodesMapResource>()
            .ok_or_else(|| NodeError::custom("NodesMapResource resource not found"))
    }

    fn get_nodes_map_mut(&mut self) -> NodeResult<Mut<NodesMapResource>> {
        self.world_mut()?
            .get_resource_mut::<NodesMapResource>()
            .to_not_found()
    }

    fn get_links_data(&self) -> NodeResult<&NodesLinkResource> {
        self.world()?
            .get_resource::<NodesLinkResource>()
            .ok_or_else(|| NodeError::custom("NodeLinksData resource not found"))
    }

    fn get_links_data_mut(&mut self) -> NodeResult<Mut<NodesLinkResource>> {
        self.world_mut()?
            .get_resource_mut::<NodesLinkResource>()
            .ok_or_else(|| NodeError::custom("NodeLinksData resource not found"))
    }

    fn get_var_from_node(&self, node_id: u64, var: VarName) -> NodeResult<VarValue> {
        let kind = self.get_node_kind(node_id)?;
        let entity = self.entity(node_id)?;
        let world = self.world()?;

        node_kind_match!(kind, {
            world
                .get::<NodeType>(entity)
                .ok_or_else(|| NodeError::not_found(node_id))?
                .get_var(var)
        })
    }

    pub fn entity(&self, node_id: u64) -> NodeResult<Entity> {
        self.get_nodes_map()?
            .get_entity(node_id)
            .ok_or_else(|| NodeError::entity_not_found(node_id))
    }

    pub fn load_ref<T: ClientNode>(&self, node_id: u64) -> NodeResult<&T> {
        let entity = self.entity(node_id)?;
        self.world()?
            .get::<T>(entity)
            .ok_or_else(|| NodeError::not_found(node_id))
    }

    pub fn load_mut<T: ClientNode + BevyComponent<Mutability = Mutable>>(
        &mut self,
        node_id: u64,
    ) -> NodeResult<Mut<T>> {
        let entity = self.entity(node_id)?;
        self.world_mut()?
            .get_mut::<T>(entity)
            .ok_or_else(|| NodeError::not_found(node_id))
    }

    pub fn load<T: ClientNode + Clone>(&self, node_id: u64) -> NodeResult<T> {
        self.load_ref::<T>(node_id).map(|node| node.clone())
    }

    // Handle SpacetimeDB updates - unified for all sources
    pub fn handle_stdb_update(&mut self, update: &StdbUpdate) {
        match self {
            Sources::Solid(_) => self.handle_solid_update(update),
            Sources::Top(_) => self.handle_top_update(update),
            Sources::Selected(_) => self.handle_selected_update(update),
            Sources::Battle(_) | Sources::None | Sources::ContextRef(..) => panic!(),
        }
    }

    fn handle_solid_update(&mut self, update: &StdbUpdate) {
        match update {
            StdbUpdate::NodeInsert(node) => {
                let kind = node.kind.parse().unwrap();
                node_kind_match!(kind, {
                    let node = node.to_node::<NodeType>().unwrap();
                    self.insert_node(node).unwrap();
                });
            }
            StdbUpdate::NodeUpdate { old: _, new } => {
                let kind = new.kind.parse().unwrap();
                let entity = match self.entity(new.id) {
                    Ok(e) => e,
                    Err(_) => return,
                };

                let world = match self.world_mut() {
                    Ok(w) => w,
                    Err(_) => return,
                };

                // Update the node data based on kind
                match kind {
                    NodeKind::NArena => {
                        if let Some(mut node) = world.get_mut::<NArena>(entity) {
                            let _ = node.inject_data(&new.data);
                        }
                    }
                    NodeKind::NFloorPool => {
                        if let Some(mut node) = world.get_mut::<NFloorPool>(entity) {
                            let _ = node.inject_data(&new.data);
                        }
                    }
                    _ => {}
                }
            }
            StdbUpdate::NodeDelete(node) => {
                let _ = self.delete_node(node.id);
            }
            StdbUpdate::LinkInsert(link) => {
                if link.solid {
                    let _ = self.add_link(link.parent, link.child);
                }
            }
            StdbUpdate::LinkDelete(link) => {
                if link.solid {
                    let _ = self.remove_link(link.parent, link.child);
                }
            }
            _ => {}
        }
    }

    fn handle_top_update(&mut self, update: &StdbUpdate) {
        match update {
            StdbUpdate::NodeInsert(..) => {
                self.handle_solid_update(update);
            }
            StdbUpdate::NodeUpdate { .. } => {
                self.handle_solid_update(update);
            }
            StdbUpdate::NodeDelete(node) => {
                let _ = self.delete_node(node.id);
            }
            StdbUpdate::LinkInsert(link) => {
                let child_kind = match link.child_kind.parse::<NodeKind>() {
                    Ok(k) => k,
                    Err(_) => return,
                };

                if let Ok(world) = self.world_mut() {
                    if let Some(mut ratings) = world.get_resource_mut::<LinkRatings>() {
                        ratings.add_rating(link.parent, link.child, child_kind, link.rating);

                        // Update links to only include top-rated
                        if let Some(top_child) = ratings.get_top(link.parent, child_kind) {
                            // Remove old links of this kind
                            if let Ok(old_children) =
                                self.get_children_of_kind(link.parent, child_kind)
                            {
                                for old_child in old_children {
                                    let _ = self.remove_link(link.parent, old_child);
                                }
                            }
                            // Add new top link
                            let _ = self.add_link(link.parent, top_child);
                        }
                    }
                }
            }
            StdbUpdate::LinkDelete(link) => {
                let child_kind = match link.child_kind.parse::<NodeKind>() {
                    Ok(k) => k,
                    Err(_) => return,
                };

                if let Ok(world) = self.world_mut() {
                    if let Some(mut ratings) = world.get_resource_mut::<LinkRatings>() {
                        ratings.remove_rating(link.parent, link.child, child_kind);

                        // Update to new top link if any
                        if let Some(new_top) = ratings.get_top(link.parent, child_kind) {
                            let _ = self.remove_link(link.parent, link.child);
                            let _ = self.add_link(link.parent, new_top);
                        } else {
                            let _ = self.remove_link(link.parent, link.child);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_selected_update(&mut self, update: &StdbUpdate) {
        match update {
            StdbUpdate::NodeInsert(..) => {
                self.handle_solid_update(update);
            }
            StdbUpdate::NodeUpdate { .. } => {
                self.handle_solid_update(update);
            }
            StdbUpdate::NodeDelete(node) => {
                let _ = self.delete_node(node.id);
            }
            StdbUpdate::PlayerLinkSelectionInsert(selection) => {
                if selection.player_id == player_id() {
                    let _ = self.add_link(selection.parent_id, selection.selected_link_id);
                }
            }
            StdbUpdate::PlayerLinkSelectionUpdate { old, new } => {
                if new.player_id == player_id() {
                    let _ = self.remove_link(old.parent_id, old.selected_link_id);
                    let _ = self.add_link(new.parent_id, new.selected_link_id);
                }
            }
            StdbUpdate::PlayerLinkSelectionDelete(selection) => {
                if selection.player_id == player_id() {
                    let _ = self.remove_link(selection.parent_id, selection.selected_link_id);
                }
            }
            _ => {}
        }
    }
}

impl ClientSource for Sources<'_> {
    fn insert_node<T: ClientNode + BevyComponent>(&mut self, node: T) -> NodeResult<()> {
        let kind = node.kind();
        let comp_children = kind.component_children();
        let mut entity: Option<Entity> = self.entity(node.id()).ok();
        if entity.is_none() {
            for kind in comp_children {
                if let Some(child) = self
                    .get_children_of_kind(node.id(), kind)?
                    .into_iter()
                    .filter_map(|id| self.entity(id).ok())
                    .next()
                {
                    entity = Some(child);
                    break;
                }
            }
        }
        if entity.is_none() {
            if let Some(parent) = kind.component_parent().and_then(|kind| {
                self.get_parents_of_kind(node.id(), kind)
                    .ok()?
                    .into_iter()
                    .next()
                    .and_then(|id| self.entity(id).ok())
            }) {
                entity = Some(parent);
            }
        }
        let world = self.world_mut()?;
        if entity.is_none() {
            entity = Some(world.spawn_empty().id());
        }
        let entity = entity.unwrap();

        let id = node.id();

        world.entity_mut(entity).insert(node);

        // Update NodesMapResource resource
        if let Some(mut node_data) = world.get_resource_mut::<NodesMapResource>() {
            node_data.insert(id, kind, entity);
        }

        Ok(())
    }

    fn world(&self) -> NodeResult<&World> {
        match self {
            Sources::Solid(w) | Sources::Top(w) | Sources::Selected(w) | Sources::Battle(w) => {
                Ok(w)
            }
            Sources::ContextRef(ctx) => ctx.world(),
            Sources::None => Err(NodeError::custom("No world available")),
        }
    }

    fn world_mut(&mut self) -> NodeResult<&mut World> {
        match self {
            Sources::Solid(w) | Sources::Top(w) | Sources::Selected(w) | Sources::Battle(w) => {
                Ok(w)
            }
            Sources::ContextRef(ctx) => Err(NodeError::custom("Can't mutate World of ContextRef")),
            Sources::None => Err(NodeError::custom("No world available")),
        }
    }

    fn entity(&self, node_id: u64) -> NodeResult<Entity> {
        let node_data = self.get_nodes_map()?;
        node_data
            .get_entity(node_id)
            .ok_or_else(|| NodeError::not_found(node_id))
    }

    fn load_ref<T: ClientNode>(&self, node_id: u64) -> NodeResult<&T> {
        let entity = self.entity(node_id)?;
        self.world()?
            .get::<T>(entity)
            .ok_or_else(|| NodeError::not_found(node_id))
    }

    fn load<T: ClientNode + Clone>(&self, node_id: u64) -> NodeResult<T> {
        self.load_ref::<T>(node_id).map(|node| node.clone())
    }

    fn battle(&self) -> NodeResult<&BattleSimulation> {
        match self {
            Sources::Battle(world) => world
                .get_resource::<BattleSimulation>()
                .ok_or_else(|| NodeError::custom("BattleSimulation resource not found")),
            _ => Err(NodeError::custom("Not a battle source")),
        }
    }

    fn rng(&mut self) -> NodeResult<&mut ChaCha8Rng> {
        todo!("get rng from BattleSimulation of Sources::Battle")
    }
}

pub enum StdbUpdate {
    NodeInsert(TNode),
    NodeUpdate {
        old: TNode,
        new: TNode,
    },
    NodeDelete(TNode),
    LinkInsert(TNodeLink),
    LinkDelete(TNodeLink),
    PlayerLinkSelectionInsert(TPlayerLinkSelection),
    PlayerLinkSelectionUpdate {
        old: TPlayerLinkSelection,
        new: TPlayerLinkSelection,
    },
    PlayerLinkSelectionDelete(TPlayerLinkSelection),
}

impl ContextSource for Sources<'_> {
    fn get_var(&self, node_id: u64, var: VarName) -> NodeResult<VarValue> {
        match self {
            Sources::Battle(world) => {
                // Check NodeStateHistory first for battle contexts
                if let Some(node_data) = world.get_resource::<NodesMapResource>() {
                    if let Some(entity) = node_data.get_entity(node_id) {
                        if let Some(state) = world.get::<NodeStateHistory>(entity) {
                            if let Some(sim) = world.get_resource::<BattleSimulation>() {
                                if let Ok(value) = state.get_at(sim.duration, var) {
                                    return Ok(value);
                                }
                            } else if let Some(value) = state.get(var) {
                                return Ok(value);
                            }
                        }
                    }
                }

                // Fall back to node's own var
                self.get_var_from_node(node_id, var)
            }
            _ => self.get_var_from_node(node_id, var),
        }
    }

    fn set_var(&mut self, node_id: u64, var: VarName, value: VarValue) -> NodeResult<()> {
        // Update the node itself
        let kind = self.get_node_kind(node_id)?;
        node_kind_match!(kind, {
            let _ = self
                .load_mut::<NodeType>(node_id)?
                .set_var(var, value.clone());
        });
        // Call var_updated for history tracking
        self.var_updated(node_id, var, value);
        Ok(())
    }

    fn var_updated(&mut self, node_id: u64, var: VarName, value: VarValue) {
        match self {
            Sources::Battle(world) => {
                // Save to NodeStateHistory in battle contexts
                if let Some(node_data) = world.get_resource::<NodesMapResource>() {
                    if let Some(entity) = node_data.get_entity(node_id) {
                        let t = world
                            .get_resource::<BattleSimulation>()
                            .map(|sim| sim.duration)
                            .unwrap_or(0.0);

                        if let Some(mut state) = world.get_mut::<NodeStateHistory>(entity) {
                            state.insert(t, 0.0, var, value);
                        } else {
                            let mut state = NodeStateHistory::default();
                            state.insert(t, 0.0, var, value);
                            world.entity_mut(entity).insert(state);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn get_node_kind(&self, node_id: u64) -> NodeResult<NodeKind> {
        self.get_nodes_map()?
            .get_kind(node_id)
            .ok_or_else(|| NodeError::not_found(node_id))
    }

    fn get_children(&self, node_id: u64) -> NodeResult<Vec<u64>> {
        Ok(self.get_links_data()?.get_children(node_id))
    }

    fn get_children_of_kind(&self, node_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        Ok(self.get_links_data()?.get_children_of_kind(node_id, kind))
    }

    fn get_parents(&self, node_id: u64) -> NodeResult<Vec<u64>> {
        Ok(self.get_links_data()?.get_parents(node_id))
    }

    fn get_parents_of_kind(&self, node_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        Ok(self.get_links_data()?.get_parents_of_kind(node_id, kind))
    }

    fn add_link(&mut self, parent_id: u64, child_id: u64) -> NodeResult<()> {
        let parent_kind = self.get_node_kind(parent_id)?;
        let child_kind = self.get_node_kind(child_id)?;
        self.get_links_data_mut()?
            .add_link(parent_id, child_id, parent_kind, child_kind);
        Ok(())
    }

    fn remove_link(&mut self, parent_id: u64, child_id: u64) -> NodeResult<()> {
        self.get_links_data_mut()?.remove_link(parent_id, child_id);
        Ok(())
    }

    fn clear_links(&mut self, node_id: u64) -> NodeResult<()> {
        self.get_links_data_mut()?.clear_node(node_id);
        Ok(())
    }

    fn is_linked(&self, parent_id: u64, child_id: u64) -> NodeResult<bool> {
        Ok(self.get_links_data()?.is_linked(parent_id, child_id))
    }

    fn delete_node(&mut self, node_id: u64) -> NodeResult<()> {
        // Clear all links
        self.clear_links(node_id)?;

        // Remove from NodesMapResource and get entity
        if let Some(entity) = self.get_nodes_map_mut()?.remove(node_id) {
            // Despawn entity
            self.world_mut()?.despawn(entity);
        }

        Ok(())
    }
}
