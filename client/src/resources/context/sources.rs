use super::*;

/// Static sources for mirroring stdb state
pub struct StaticSources {
    pub top: Sources<'static>,
    pub selected: Sources<'static>,
    pub solid: Sources<'static>,
    pub core: Sources<'static>,
}

impl StaticSources {
    pub fn new() -> Self {
        Self {
            top: Sources::new_top(),
            selected: Sources::new_selected(),
            solid: Sources::new_solid(),
            core: Sources::new_core(),
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
            panic!("Static sources not initialized")
        }
    }
}

pub fn with_solid_source<R, F>(f: F) -> NodeResult<R>
where
    F: FnOnce(&mut ClientContext) -> NodeResult<R>,
{
    with_static_sources(|sources| {
        let taken = std::mem::replace(&mut sources.solid, Sources::None);
        let mut context = taken.as_context();
        let result = f(&mut context);
        sources.solid = context.into_source();
        result
    })
}

pub fn with_top_source<R, F>(f: F) -> NodeResult<R>
where
    F: FnOnce(&mut ClientContext) -> NodeResult<R>,
{
    with_static_sources(|sources| {
        let taken = std::mem::replace(&mut sources.top, Sources::None);
        let mut context = taken.as_context();
        let result = f(&mut context);
        sources.top = context.into_source();
        result
    })
}

pub fn with_selected_source<R, F>(f: F) -> NodeResult<R>
where
    F: FnOnce(&mut ClientContext) -> NodeResult<R>,
{
    with_static_sources(|sources| {
        let taken = std::mem::replace(&mut sources.selected, Sources::None);
        let mut context = taken.as_context();
        let result = f(&mut context);
        sources.selected = context.into_source();
        result
    })
}

pub fn with_core_source<R, F>(f: F) -> NodeResult<R>
where
    F: FnOnce(&mut ClientContext) -> NodeResult<R>,
{
    with_static_sources(|sources| {
        let taken = std::mem::replace(&mut sources.core, Sources::None);
        let mut context = taken.as_context();
        let result = f(&mut context);
        sources.core = context.into_source();
        result
    })
}

pub trait ClientSource {
    fn insert_node<T: ClientNode + BevyComponent>(&mut self, node: &T) -> NodeResult<()>;
    fn world(&self) -> NodeResult<&World>;
    fn world_mut(&mut self) -> NodeResult<&mut World>;
    fn entity(&self, node_id: u64) -> NodeResult<Entity>;
    fn load_ref<T: ClientNode>(&self, node_id: u64) -> NodeResult<&T>;
    fn load<T: ClientNode + Clone>(&self, node_id: u64) -> NodeResult<T>;
    fn battle(&self) -> NodeResult<&BattleSimulation>;
    fn battle_mut(&mut self) -> NodeResult<&mut BattleSimulation>;
    fn rng(&mut self) -> NodeResult<&mut ChaCha8Rng>;
    fn t(&self) -> Option<f32>;
    fn t_mut(&mut self) -> Option<&mut f32>;
}

/// Sources enum for different node data sources
#[derive(Debug, Default, Display)]
pub enum Sources<'a> {
    Solid(World),
    Core(World),
    Top(World),
    Selected(World),
    Battle(BattleSimulation, f32),
    SourceRef(&'a Sources<'a>),
    #[default]
    None,
}

impl<'a> Sources<'a> {
    pub fn new_solid() -> Self {
        let mut world = World::new();
        Self::init_world(&mut world);
        Sources::Solid(world)
    }

    pub fn new_core() -> Self {
        let mut world = World::new();
        Self::init_world(&mut world);
        Sources::Core(world)
    }

    pub fn new_top() -> Self {
        let mut world = World::new();
        Self::init_world(&mut world);
        world.init_resource::<LinkRatings>();
        world.init_resource::<LinksMapResource>();
        Sources::Top(world)
    }

    pub fn new_selected() -> Self {
        let mut world = World::new();
        Self::init_world(&mut world);
        world.init_resource::<LinksMapResource>();
        Sources::Selected(world)
    }

    pub fn as_context(self) -> ClientContext<'a> {
        ClientContext::new(self)
    }

    pub fn take(&mut self) -> Sources<'a> {
        std::mem::replace(self, Sources::None)
    }

    pub fn exec_context<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut ClientContext) -> R,
    {
        let taken_source = self.take();
        let mut ctx = taken_source.as_context();
        let result = f(&mut ctx);
        *self = ctx.source_mut().take();
        result
    }

    pub fn exec_context_ref<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut ClientContext) -> R,
    {
        let mut ctx = ClientContext::new(Sources::SourceRef(self));
        f(&mut ctx)
    }

    pub fn to_solid_source() -> Sources<'static> {
        let mut world = World::new();
        Self::init_world(&mut world);
        Sources::Solid(world)
    }

    pub fn get_battle_simulation(&self) -> NodeResult<&BattleSimulation> {
        match self {
            Sources::Battle(sim, _) => Ok(sim),
            _ => Err(NodeError::custom("Not a battle source")),
        }
    }

    /// Unified initialization function for all World sources
    fn init_world(world: &mut World) {
        world.init_resource::<NodesMapResource>();
        world.init_resource::<NodesLinkResource>();
        world.init_resource::<LinksMapResource>();
    }

    pub fn world(&self) -> NodeResult<&World> {
        match self {
            Sources::Solid(world)
            | Sources::Core(world)
            | Sources::Top(world)
            | Sources::Selected(world) => Ok(world),
            Sources::Battle(sim, _) => Ok(&sim.world),
            Sources::SourceRef(s) => s.world(),
            Sources::None => Err(NodeError::custom("No world in empty source")),
        }
    }

    pub fn world_mut(&mut self) -> NodeResult<&mut World> {
        match self {
            Sources::Solid(world)
            | Sources::Core(world)
            | Sources::Top(world)
            | Sources::Selected(world) => Ok(world),
            Sources::Battle(sim, _) => Ok(&mut sim.world),
            Sources::SourceRef(_) => Err(NodeError::custom(
                "Cannot get mutable world from source ref",
            )),
            Sources::None => Err(NodeError::custom("No world in empty source")),
        }
    }

    pub fn get_nodes_map(&self) -> NodeResult<&NodesMapResource> {
        self.world()?
            .get_resource::<NodesMapResource>()
            .ok_or_else(|| NodeError::custom("NodesMapResource resource not found"))
    }

    fn get_nodes_map_mut(&mut self) -> NodeResult<Mut<'_, NodesMapResource>> {
        self.world_mut()?
            .get_resource_mut::<NodesMapResource>()
            .to_not_found()
    }

    fn get_links_data(&self) -> NodeResult<&NodesLinkResource> {
        self.world()?
            .get_resource::<NodesLinkResource>()
            .ok_or_else(|| NodeError::custom("NodeLinksData resource not found"))
    }

    fn get_links_data_mut(&mut self) -> NodeResult<Mut<'_, NodesLinkResource>> {
        self.world_mut()?
            .get_resource_mut::<NodesLinkResource>()
            .ok_or_else(|| NodeError::custom("NodeLinksData resource not found"))
    }

    fn get_var_from_node(&self, node_id: u64, var: VarName) -> NodeResult<VarValue> {
        let entity = self.entity(node_id).track()?;
        let map = self.get_nodes_map()?;
        let kind = map
            .get_node_ids(entity)
            .into_iter()
            .filter_map(|id| map.get_kind(id))
            .find(|kind| kind.var_names().contains(&var));
        let Some(kind) = kind else {
            return Err(NodeError::var_not_found(var)).track();
        };
        let world = self.world().track()?;

        node_kind_match!(kind, {
            world
                .get::<NodeType>(entity)
                .ok_or_else(|| NodeError::not_found(node_id))
                .track()?
                .get_var(var)
        })
    }

    pub fn entity(&self, node_id: u64) -> NodeResult<Entity> {
        self.get_nodes_map()?
            .get_entity(node_id)
            .ok_or_else(|| NodeError::entity_not_found(node_id))
            .track()
    }

    pub fn load_ref<T: ClientNode>(&self, node_id: u64) -> NodeResult<&T> {
        let entity = self.entity(node_id).track()?;
        self.world()?
            .get::<T>(entity)
            .ok_or_else(|| NodeError::not_found(node_id))
    }

    pub fn load_mut<T: ClientNode + BevyComponent<Mutability = Mutable>>(
        &mut self,
        node_id: u64,
    ) -> NodeResult<Mut<'_, T>> {
        let entity = self.entity(node_id).track()?;
        self.world_mut()?
            .get_mut::<T>(entity)
            .ok_or_else(|| NodeError::not_found(node_id))
    }

    pub fn load<T: ClientNode + Clone>(&self, node_id: u64) -> NodeResult<T> {
        self.load_ref::<T>(node_id).track().map(|node| node.clone())
    }

    // Handle SpacetimeDB updates - unified for all sources
    pub fn handle_stdb_update(&mut self, update: &StdbUpdate) -> NodeResult<()> {
        self.handle_update_with_filtering(update)
    }

    fn handle_update_with_filtering(&mut self, update: &StdbUpdate) -> NodeResult<()> {
        match update {
            StdbUpdate::NodeInsert(node) | StdbUpdate::NodeUpdate { new: node, .. } => {
                if self.should_process_node(node) {
                    let kind = node.kind.parse().map_err(|_| {
                        NodeError::custom(format!("Invalid node kind: {}", node.kind))
                    })?;
                    self.spawn_node_with_entity_merging(node.id, kind, node)?;
                }
            }
            StdbUpdate::NodeDelete(node) => {
                self.delete_node(node.id)?;
            }
            StdbUpdate::LinkInsert(link) => {
                if self.should_process_link(link) {
                    self.handle_link_insert(link)?;
                }
            }
            StdbUpdate::LinkUpdate { old, new } => {
                if self.should_process_link(new) {
                    self.handle_link_update(old, new)?;
                }
            }
            StdbUpdate::LinkDelete(link) => {
                if self.should_process_link(link) {
                    self.handle_link_delete(link)?;
                }
            }
            StdbUpdate::PlayerLinkSelectionInsert(selection) => {
                self.handle_player_selection_insert(selection)?;
            }
            StdbUpdate::PlayerLinkSelectionUpdate { old, new } => {
                self.handle_player_selection_update(old, new)?;
            }
            StdbUpdate::PlayerLinkSelectionDelete(selection) => {
                self.handle_player_selection_delete(selection)?;
            }
        }
        Ok(())
    }

    fn should_process_node(&self, node: &TNode) -> bool {
        match self {
            Sources::Solid(_) => true,
            Sources::Core(_) => node.owner == ID_CORE,
            Sources::Top(_) | Sources::Selected(_) => node.owner == ID_CORE || node.owner == 0,
            Sources::Battle(..) | Sources::None | Sources::SourceRef(..) => true,
        }
    }

    fn should_process_link(&self, link: &TNodeLink) -> bool {
        fn check_owner(link: &TNodeLink, owners: Vec<u64>) -> bool {
            if cfg!(test) {
                return true;
            }
            let nodes = cn().db.nodes_world().id();
            let Some(parent_owner) = nodes.find(&link.parent).map(|n| n.owner) else {
                return false;
            };
            let Some(child_owner) = nodes.find(&link.child).map(|n| n.owner) else {
                return false;
            };
            owners.contains(&parent_owner) && owners.contains(&child_owner)
        }
        match self {
            Sources::Solid(_) => link.solid,
            Sources::Core(_) => {
                if !link.solid {
                    return false;
                }
                check_owner(link, vec![ID_CORE])
            }
            Sources::Top(_) | Sources::Selected(_) => check_owner(link, vec![ID_CORE, 0]),
            Sources::Battle(..) | Sources::None | Sources::SourceRef(..) => false,
        }
    }

    fn handle_link_insert(&mut self, link: &TNodeLink) -> NodeResult<()> {
        let world = self.world_mut()?;
        if let Some(mut links_map) = world.get_resource_mut::<LinksMapResource>() {
            links_map.insert(link.clone());
        }
        // Handle component links specially
        if Self::is_component_link(link) {
            // First, check if parent already has a child of this type
            let existing_child_of_same_kind = {
                let world = self.world()?;
                let node_map = world
                    .get_resource::<NodesMapResource>()
                    .ok_or_else(|| NodeError::custom("NodesMapResource not found"))?;

                // Get parent entity
                if let Some(&parent_entity) = node_map.entities.get(&link.parent) {
                    // Find existing component of the same kind on parent entity
                    node_map
                        .entity_to_nodes
                        .get(&parent_entity)
                        .and_then(|nodes| {
                            nodes
                                .iter()
                                .find(|&&node_id| {
                                    node_map.get_kind(node_id).map_or(false, |kind| {
                                        kind == link.child_kind.to_kind() && node_id != link.parent
                                    })
                                })
                                .copied()
                        })
                } else {
                    None
                }
            };

            // If there's an existing child of the same kind, unlink it first
            if let Some(existing_child_id) = existing_child_of_same_kind {
                self.unlink_from_component_parent(existing_child_id)?;
            }
            // Now unlink the new child from any existing component parent it might have
            self.unlink_from_component_parent(link.child)?;
            // Finally, merge the child onto the parent's entity
            if self.entity(link.parent).is_ok() && self.entity(link.child).is_ok() {
                let parent_entity = self.entity(link.parent)?;
                self.merge_component_entities_for_link(link.child, parent_entity)?;
            }
        }

        match self {
            Sources::Solid(_) | Sources::Core(_) => {
                self.get_links_data_mut()?.add_link(
                    link.parent,
                    link.child,
                    link.parent_kind.to_kind(),
                    link.child_kind.to_kind(),
                );
            }
            Sources::Top(_) => {
                self.handle_top_link_insert(link)?;
            }
            Sources::Selected(..)
            | Sources::Battle(..)
            | Sources::SourceRef(..)
            | Sources::None => {}
        }

        Ok(())
    }

    fn handle_link_delete(&mut self, link: &TNodeLink) -> NodeResult<()> {
        let world = self.world_mut()?;
        if let Some(mut links_map) = world.get_resource_mut::<LinksMapResource>() {
            links_map.remove(link.id);
        }
        // Handle component links specially - unlink the child from its parent
        if Self::is_component_link(link) {
            // Unlink the child and all its recursive component children
            self.unlink_from_component_parent(link.child)?;
        }

        match self {
            Sources::Solid(_) | Sources::Core(_) => {
                self.remove_link(link.parent, link.child)?;
            }
            Sources::Top(_) => {
                self.handle_top_link_delete(link)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_link_update(&mut self, old: &TNodeLink, new: &TNodeLink) -> NodeResult<()> {
        let world = self.world_mut()?;
        if let Some(mut links_map) = world.get_resource_mut::<LinksMapResource>() {
            links_map.remove(old.id);
            links_map.insert(new.clone());
        }
        match self {
            Sources::Solid(_) | Sources::Core(_) => {
                if old.solid && !new.solid {
                    self.remove_link(old.parent, old.child)?;
                } else if !old.solid && new.solid {
                    self.add_link(new.parent, new.child)?;
                }
            }
            Sources::Top(_) => {
                self.handle_top_link_update(old, new)?;
            }
            Sources::Selected(..)
            | Sources::Battle(..)
            | Sources::SourceRef(..)
            | Sources::None => {}
        }

        // Handle component links when they become solid
        if !old.solid && new.solid && Self::is_component_link(new) {
            // Handle this as a new link insertion for component merging
            if self.entity(new.parent).is_ok() && self.entity(new.child).is_ok() {
                // Check if parent already has a child of this type
                let existing_child_of_same_kind = {
                    let world = self.world()?;
                    let node_map = world
                        .get_resource::<NodesMapResource>()
                        .ok_or_else(|| NodeError::custom("NodesMapResource not found"))?;

                    // Get parent entity
                    if let Some(&parent_entity) = node_map.entities.get(&new.parent) {
                        // Find existing component of the same kind on parent entity
                        node_map
                            .entity_to_nodes
                            .get(&parent_entity)
                            .and_then(|nodes| {
                                nodes
                                    .iter()
                                    .find(|&&node_id| {
                                        node_map.get_kind(node_id).map_or(false, |kind| {
                                            kind == new.child_kind.to_kind()
                                                && node_id != new.parent
                                        })
                                    })
                                    .copied()
                            })
                    } else {
                        None
                    }
                };
                // If there's an existing child of the same kind, unlink it first
                if let Some(existing_child_id) = existing_child_of_same_kind {
                    self.unlink_from_component_parent(existing_child_id)?;
                }
                // Now unlink the new child from any existing component parent it might have
                self.unlink_from_component_parent(new.child)?;
                // Finally, merge the child onto the parent's entity
                let parent_entity = self.entity(new.parent)?;
                self.merge_component_entities_for_link(new.child, parent_entity)?;
            }
        }

        Ok(())
    }

    fn handle_player_selection_insert(
        &mut self,
        selection: &TPlayerLinkSelection,
    ) -> NodeResult<()> {
        match self {
            Sources::Selected(_) => {
                if selection.player_id == player_id() {
                    let world = self.world()?;
                    let link = world
                        .get_resource::<LinksMapResource>()
                        .and_then(|links| links.get(selection.link_id))
                        .cloned()
                        .ok_or_else(|| {
                            NodeError::custom(format!("Link#{} not found", selection.link_id))
                        })?;
                    self.add_link(link.parent, link.child)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_player_selection_update(
        &mut self,
        old: &TPlayerLinkSelection,
        new: &TPlayerLinkSelection,
    ) -> NodeResult<()> {
        match self {
            Sources::Selected(_) => {
                if new.player_id == player_id() {
                    let world = self.world()?;
                    let links_map = world
                        .get_resource::<LinksMapResource>()
                        .ok_or_else(|| NodeError::custom("LinksMapResource not found"))?;
                    let old_link = links_map
                        .get(old.link_id)
                        .cloned()
                        .ok_or_else(|| NodeError::custom("Old link not found"))?;
                    let new_link = links_map
                        .get(new.link_id)
                        .cloned()
                        .ok_or_else(|| NodeError::custom("New link not found"))?;
                    self.remove_link(old_link.parent, old_link.child)?;
                    self.add_link(new_link.parent, new_link.child)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_player_selection_delete(
        &mut self,
        selection: &TPlayerLinkSelection,
    ) -> NodeResult<()> {
        match self {
            Sources::Selected(_) => {
                if selection.player_id == player_id() {
                    let world = self.world()?;
                    let link = world
                        .get_resource::<LinksMapResource>()
                        .and_then(|links| links.get(selection.link_id))
                        .cloned()
                        .ok_or_else(|| NodeError::custom("Link not found"))?;
                    self.remove_link(link.parent, link.child)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn spawn_node_with_entity_merging(
        &mut self,
        node_id: u64,
        kind: NodeKind,
        node_data: &TNode,
    ) -> NodeResult<()> {
        let target_entity = self.find_or_create_component_entity(node_id, kind)?;
        let world = self.world_mut()?;
        node_kind_match!(kind, {
            let node = node_data.to_node::<NodeType>()?;
            world.entity_mut(target_entity).insert(node);
            if let Some(mut node_map) = world.get_resource_mut::<NodesMapResource>() {
                node_map.insert(node_id, kind, target_entity);
            }
        });
        debug!("spawn {self} {kind} {node_id} {target_entity}");
        Ok(())
    }

    fn find_or_create_component_entity(
        &mut self,
        node_id: u64,
        kind: NodeKind,
    ) -> NodeResult<Entity> {
        if let Ok(existing_entity) = self.entity(node_id) {
            return Ok(existing_entity);
        }
        // Check if we have a component parent that already exists
        if let Some(parent_kind) = kind.component_parent() {
            if let Ok(parents) = self.get_parents_of_kind(node_id, parent_kind) {
                for parent_id in parents {
                    if let Ok(entity) = self.entity(parent_id) {
                        return Ok(entity);
                    }
                }
            }
        }
        // Check if we have component children that already exist
        for child_kind in kind.component_children() {
            if let Ok(children) = self.get_children_of_kind(node_id, child_kind) {
                for child_id in children {
                    if let Ok(entity) = self.entity(child_id) {
                        return Ok(entity);
                    }
                }
            }
        }
        // No related components exist, create a new entity
        let world = self.world_mut()?;
        let target_entity = world.spawn_empty().id();
        Ok(target_entity)
    }

    fn merge_component_entities_for_link(
        &mut self,
        child_id: u64,
        target_entity: Entity,
    ) -> NodeResult<()> {
        // Collect all nodes to migrate (child and its recursive component children)
        let nodes_to_migrate = self.collect_component_tree_with_recursive_children(child_id)?;
        if nodes_to_migrate.is_empty() {
            return Ok(());
        }
        let world = self.world_mut()?;
        // Get the source entity
        let source_entity = {
            let node_map = world
                .get_resource::<NodesMapResource>()
                .ok_or_else(|| NodeError::custom("NodesMapResource not found"))?;
            node_map
                .entities
                .get(&child_id)
                .copied()
                .ok_or_else(|| NodeError::custom("Child entity not found"))?
        };
        if source_entity == target_entity {
            return Ok(()); // Already on the same entity
        }
        // Migrate all components
        for (_node_id, node_kind) in &nodes_to_migrate {
            Self::migrate_node_component(world, source_entity, target_entity, *node_kind)?;
        }
        // Update node mappings
        if let Some(mut node_map) = world.get_resource_mut::<NodesMapResource>() {
            for (node_id, node_kind) in nodes_to_migrate {
                node_map.insert(node_id, node_kind, target_entity);
            }
        }
        // Despawn the source entity if it's now empty
        let should_despawn = {
            let node_map = world
                .get_resource::<NodesMapResource>()
                .ok_or_else(|| NodeError::custom("NodesMapResource not found"))?;
            node_map
                .entity_to_nodes
                .get(&source_entity)
                .map_or(true, |nodes| nodes.is_empty())
        };
        if should_despawn {
            world.despawn(source_entity);
        }
        Ok(())
    }

    fn unlink_from_component_parent(&mut self, node_id: u64) -> NodeResult<()> {
        let node_kind = {
            let world = self.world()?;
            let node_map = world
                .get_resource::<NodesMapResource>()
                .ok_or_else(|| NodeError::custom("NodesMapResource not found"))?;
            node_map.get_kind(node_id)
        };
        let Some(kind) = node_kind else {
            return Ok(()); // Node doesn't exist
        };
        // Check if this node has a component parent type
        if kind.component_parent().is_none() {
            return Ok(()); // Not a component node
        }
        // Collect all nodes to move (this node and its recursive component children)
        let nodes_to_move = self.collect_component_tree_with_recursive_children(node_id)?;
        if nodes_to_move.is_empty() {
            return Ok(());
        }
        // Create a new entity for these components
        let world = self.world_mut()?;
        let new_entity = world.spawn_empty().id();
        // Get the current entity
        let current_entity = {
            let node_map = world
                .get_resource::<NodesMapResource>()
                .ok_or_else(|| NodeError::custom("NodesMapResource not found"))?;
            node_map
                .entities
                .get(&node_id)
                .copied()
                .ok_or_else(|| NodeError::custom("Node entity not found"))?
        };
        // Migrate all components to the new entity
        for (_node_id, node_kind) in &nodes_to_move {
            Self::migrate_node_component(world, current_entity, new_entity, *node_kind)?;
        }
        // Update node mappings
        if let Some(mut node_map) = world.get_resource_mut::<NodesMapResource>() {
            for (move_id, move_kind) in nodes_to_move {
                node_map.insert(move_id, move_kind, new_entity);
            }
        }
        Ok(())
    }

    fn is_component_link(link: &TNodeLink) -> bool {
        let parent_kind = link.parent_kind.to_kind();
        let child_kind = link.child_kind.to_kind();
        child_kind
            .component_parent()
            .is_some_and(|kind| kind == parent_kind)
    }

    fn collect_component_tree_with_recursive_children(
        &self,
        root_id: u64,
    ) -> NodeResult<Vec<(u64, NodeKind)>> {
        let world = self.world()?;
        let node_map = world
            .get_resource::<NodesMapResource>()
            .ok_or_else(|| NodeError::custom("NodesMapResource not found"))?;

        let root_kind = node_map
            .get_kind(root_id)
            .ok_or_else(|| NodeError::custom("Root node kind not found"))?;

        let mut result = Vec::new();
        result.push((root_id, root_kind));

        // Get all recursive component children kinds
        let recursive_children_kinds = root_kind.component_children_recursive();

        if !recursive_children_kinds.is_empty() {
            // Find the entity this node is on
            if let Some(&entity) = node_map.entities.get(&root_id) {
                // Find all nodes on this entity that match the recursive children kinds
                if let Some(entity_nodes) = node_map.entity_to_nodes.get(&entity) {
                    for &node_id in entity_nodes {
                        if node_id != root_id {
                            if let Some(node_kind) = node_map.get_kind(node_id) {
                                if recursive_children_kinds.contains(&node_kind) {
                                    result.push((node_id, node_kind));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    fn migrate_node_component(
        world: &mut World,
        from_entity: Entity,
        to_entity: Entity,
        kind: NodeKind,
    ) -> NodeResult<()> {
        node_kind_match!(kind, {
            if let Some(component) = world.entity_mut(from_entity).take::<NodeType>() {
                world.entity_mut(to_entity).insert(component);
            }
        });
        Ok(())
    }

    fn handle_top_link_insert(&mut self, link: &TNodeLink) -> NodeResult<()> {
        use schema::NodeKind;

        let parent_kind = link.parent_kind.to_kind();
        let child_kind = link.child_kind.to_kind();
        let world = self.world_mut()?;
        let mut ratings = world.resource_mut::<LinkRatings>();

        ratings.add_rating(
            link.parent,
            link.child,
            parent_kind,
            child_kind,
            link.rating,
        );

        if let Some(relation) = NodeKind::get_relation(parent_kind, child_kind) {
            match relation {
                schema::NodeRelation::OneToMany => {
                    // For one-to-many (e.g., NHouse -> NUnit), we need to update the top parent for this child
                    if let Some(top_parent) = ratings.get_top_parent(link.child, parent_kind) {
                        // Remove old parent links for this child
                        if let Ok(old_parents) = self.get_parents_of_kind(link.child, parent_kind) {
                            for old_parent in old_parents {
                                self.remove_link(old_parent, link.child)?;
                            }
                        }
                        // Add link to top parent
                        self.add_link(top_parent, link.child)?;
                    }
                }
                schema::NodeRelation::ManyToOne => {
                    // For many-to-one (e.g., NUnit -> NUnitDescription), we need to update the top child for this parent
                    if let Some(top_child) = ratings.get_top_child(link.parent, child_kind) {
                        if let Ok(old_children) = self.get_children_of_kind(link.parent, child_kind)
                        {
                            for old_child in old_children {
                                self.remove_link(link.parent, old_child)?;
                            }
                        }
                        self.add_link(link.parent, top_child)?;
                    }
                }
                schema::NodeRelation::OneToOne => {
                    // For one-to-one, update top child for parent
                    if let Some(top_child) = ratings.get_top_child(link.parent, child_kind) {
                        if let Ok(old_children) = self.get_children_of_kind(link.parent, child_kind)
                        {
                            for old_child in old_children {
                                self.remove_link(link.parent, old_child)?;
                            }
                        }
                        self.add_link(link.parent, top_child)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_top_link_update(&mut self, old: &TNodeLink, new: &TNodeLink) -> NodeResult<()> {
        use schema::NodeKind;

        let parent_kind = new.parent_kind.to_kind();
        let child_kind = new.child_kind.to_kind();

        let world = self.world_mut()?;
        let mut links_map = world.resource_mut::<LinksMapResource>();
        if old.id != new.id {
            links_map.remove(old.id);
        }
        links_map.insert(new.clone());
        let mut ratings = world.resource_mut::<LinkRatings>();

        let old_parent_kind = old.parent_kind.to_kind();
        let old_child_kind = old.child_kind.to_kind();
        ratings.remove_rating(old.parent, old.child, old_parent_kind, old_child_kind);
        ratings.add_rating(new.parent, new.child, parent_kind, child_kind, new.rating);

        if let Some(relation) = NodeKind::get_relation(parent_kind, child_kind) {
            match relation {
                schema::NodeRelation::OneToMany => {
                    // For one-to-many (e.g., NHouse -> NUnit), we need to update the top parent for this child
                    if let Some(top_parent) = ratings.get_top_parent(new.child, parent_kind) {
                        // Remove old parent links for this child
                        if let Ok(old_parents) = self.get_parents_of_kind(new.child, parent_kind) {
                            for old_parent in old_parents {
                                self.remove_link(old_parent, new.child)?;
                            }
                        }
                        // Add link to top parent
                        self.add_link(top_parent, new.child)?;
                    }
                }
                schema::NodeRelation::ManyToOne => {
                    // For many-to-one (e.g., NUnit -> NUnitDescription), we need to update the top child for this parent
                    if let Some(top_child) = ratings.get_top_child(new.parent, child_kind) {
                        if let Ok(old_children) = self.get_children_of_kind(new.parent, child_kind)
                        {
                            for old_child in old_children {
                                self.remove_link(new.parent, old_child)?;
                            }
                        }
                        self.add_link(new.parent, top_child)?;
                    }
                }
                schema::NodeRelation::OneToOne => {
                    // For one-to-one, update top child for parent
                    if let Some(top_child) = ratings.get_top_child(new.parent, child_kind) {
                        if let Ok(old_children) = self.get_children_of_kind(new.parent, child_kind)
                        {
                            for old_child in old_children {
                                self.remove_link(new.parent, old_child)?;
                            }
                        }
                        self.add_link(new.parent, top_child)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_top_link_delete(&mut self, link: &TNodeLink) -> NodeResult<()> {
        use schema::NodeKind;

        let parent_kind = link.parent_kind.to_kind();
        let child_kind = link.child_kind.to_kind();

        // Update ratings and get new top after removal
        let (top_child_after_removal, top_parent_after_removal) = {
            let world = self.world_mut()?;
            let mut links_map = world.resource_mut::<LinksMapResource>();
            links_map.remove(link.id);
            let mut ratings = world.resource_mut::<LinkRatings>();
            ratings.remove_rating(link.parent, link.child, parent_kind, child_kind);
            (
                ratings.get_top_child(link.parent, child_kind),
                ratings.get_top_parent(link.child, parent_kind),
            )
        };

        if let Some(relation) = NodeKind::get_relation(parent_kind, child_kind) {
            match relation {
                schema::NodeRelation::OneToMany => {
                    // For one-to-many (e.g., NHouse -> NUnit), update the top parent for this child
                    if let Ok(old_parents) = self.get_parents_of_kind(link.child, parent_kind) {
                        for old_parent in old_parents {
                            self.remove_link(old_parent, link.child)?;
                        }
                    }
                    // Add link to new top parent if any
                    if let Some(top_parent) = top_parent_after_removal {
                        self.add_link(top_parent, link.child)?;
                    }
                }
                schema::NodeRelation::ManyToOne => {
                    // For many-to-one (e.g., NUnit -> NUnitDescription), update the top child for this parent
                    if let Ok(old_children) = self.get_children_of_kind(link.parent, child_kind) {
                        for old_child in old_children {
                            self.remove_link(link.parent, old_child)?;
                        }
                    }
                    // Add link to new top child if any
                    if let Some(top_child) = top_child_after_removal {
                        self.add_link(link.parent, top_child)?;
                    }
                }
                schema::NodeRelation::OneToOne => {
                    // For one-to-one, update top child for parent
                    if let Ok(old_children) = self.get_children_of_kind(link.parent, child_kind) {
                        for old_child in old_children {
                            self.remove_link(link.parent, old_child)?;
                        }
                    }
                    // Add link to new top child if any
                    if let Some(top_child) = top_child_after_removal {
                        self.add_link(link.parent, top_child)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl ClientSource for Sources<'_> {
    fn insert_node<T: ClientNode + BevyComponent>(&mut self, node: &T) -> NodeResult<()> {
        let kind = node.kind();
        let entity = self.find_or_create_component_entity(node.id(), kind)?;
        let world = self.world_mut()?;
        let id = node.id();

        world.entity_mut(entity).insert(node.clone());

        if let Some(mut node_data) = world.get_resource_mut::<NodesMapResource>() {
            node_data.insert(id, kind, entity);
        }

        Ok(())
    }

    fn world(&self) -> NodeResult<&World> {
        match self {
            Sources::Solid(w) | Sources::Core(w) | Sources::Top(w) | Sources::Selected(w) => Ok(w),
            Sources::Battle(sim, _) => Ok(&sim.world),
            Sources::SourceRef(source) => source.world(),
            Sources::None => Err(NodeError::custom("No world available")),
        }
    }

    fn world_mut(&mut self) -> NodeResult<&mut World> {
        match self {
            Sources::Solid(w) | Sources::Core(w) | Sources::Top(w) | Sources::Selected(w) => Ok(w),
            Sources::Battle(sim, _) => Ok(&mut sim.world),
            Sources::SourceRef(_) => Err(NodeError::custom("Can't mutate World of SourceRef")),
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
        let entity = self.entity(node_id).track()?;
        self.world()?
            .get::<T>(entity)
            .ok_or_else(|| NodeError::not_found(node_id))
    }

    fn load<T: ClientNode + Clone>(&self, node_id: u64) -> NodeResult<T> {
        self.load_ref::<T>(node_id).track().map(|node| node.clone())
    }

    fn battle(&self) -> NodeResult<&BattleSimulation> {
        match self {
            Sources::Battle(sim, _) => Ok(sim),
            Sources::SourceRef(source) => source.battle(),
            _ => Err(NodeError::custom("Not a battle source")),
        }
    }

    fn battle_mut(&mut self) -> NodeResult<&mut BattleSimulation> {
        match self {
            Sources::Battle(sim, _) => Ok(sim),
            _ => Err(NodeError::custom("Not a battle source")),
        }
    }

    fn t(&self) -> Option<f32> {
        match self {
            Sources::Battle(_, time) => Some(*time),
            Sources::SourceRef(source) => source.t(),
            _ => None,
        }
    }

    fn t_mut(&mut self) -> Option<&mut f32> {
        match self {
            Sources::Battle(_, time) => Some(time),
            _ => None,
        }
    }

    fn rng(&mut self) -> NodeResult<&mut ChaCha8Rng> {
        match self {
            Sources::Battle(sim, _) => Ok(&mut sim.rng),
            Sources::SourceRef(_) => {
                Err(NodeError::custom("Cannot get mutable RNG from source ref"))
            }
            _ => Err(NodeError::custom("RNG only available in battle context")),
        }
    }
}

#[derive(Debug)]
pub enum StdbUpdate {
    NodeInsert(TNode),
    NodeUpdate {
        old: TNode,
        new: TNode,
    },
    NodeDelete(TNode),
    LinkInsert(TNodeLink),
    LinkUpdate {
        old: TNodeLink,
        new: TNodeLink,
    },
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
        if let Ok(sim) = self.battle() {
            let world = &sim.world;
            let time = self.t().unwrap_or(sim.duration);
            // Check NodeStateHistory first for battle contexts
            if let Some(node_data) = world.get_resource::<NodesMapResource>() {
                if let Some(entity) = node_data.get_entity(node_id) {
                    if let Some(state) = world.get::<NodeStateHistory>(entity) {
                        if let Ok(value) = state.get_at(time, var) {
                            return Ok(value);
                        } else if let Some(value) = state.get(var) {
                            return Ok(value);
                        }
                    }
                }
            }

            // Fall back to node's own var
            self.get_var_from_node(node_id, var)
        } else {
            self.get_var_from_node(node_id, var)
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
        if let Ok(sim) = self.battle_mut() {
            // Save to NodeStateHistory in battle contexts
            if let Some(node_data) = sim.world.get_resource::<NodesMapResource>() {
                if let Some(entity) = node_data.get_entity(node_id) {
                    let t = sim.duration;
                    if let Some(mut state) = sim.world.get_mut::<NodeStateHistory>(entity) {
                        if state.insert(t, 0.0, var, value) {
                            // sim.duration += 0.01;
                        }
                    } else {
                        panic!("NodeStateHistory not found for {node_id}");
                    }
                }
            }
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
