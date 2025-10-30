use super::*;
use std::collections::HashSet;

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

/// Trait for client-side node data sources
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
            Sources::Solid(w) | Sources::Top(w) | Sources::Selected(w) => Ok(w),
            Sources::Battle(sim, _) => Ok(&sim.world),
            Sources::SourceRef(source) => source.world(),
            Sources::None => Err(NodeError::custom("No world in None source")),
        }
    }

    pub fn world_mut(&mut self) -> NodeResult<&mut World> {
        match self {
            Sources::Solid(world) | Sources::Top(world) | Sources::Selected(world) => Ok(world),
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
    ) -> NodeResult<Mut<'_, T>> {
        let entity = self.entity(node_id)?;
        self.world_mut()?
            .get_mut::<T>(entity)
            .ok_or_else(|| NodeError::not_found(node_id))
    }

    pub fn load<T: ClientNode + Clone>(&self, node_id: u64) -> NodeResult<T> {
        self.load_ref::<T>(node_id).map(|node| node.clone())
    }

    // Handle SpacetimeDB updates - unified for all sources
    pub fn handle_stdb_update(&mut self, update: &StdbUpdate) -> NodeResult<()> {
        self.handle_update_with_filtering(update)
    }

    fn handle_update_with_filtering(&mut self, update: &StdbUpdate) -> NodeResult<()> {
        match update {
            StdbUpdate::NodeInsert(node) | StdbUpdate::NodeUpdate { new: node, .. } => {
                let kind = node
                    .kind
                    .parse()
                    .map_err(|_| NodeError::custom(format!("Invalid node kind: {}", node.kind)))?;
                self.spawn_node_with_entity_merging(node.id, kind, node)?;
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
                self.handle_link_update(old, new)?;
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

    fn should_process_link(&self, link: &TNodeLink) -> bool {
        match self {
            Sources::Solid(_) => link.solid,
            Sources::Top(_) | Sources::Selected(_) => true,
            Sources::Battle(..) | Sources::None | Sources::SourceRef(..) => false,
        }
    }

    fn handle_link_insert(&mut self, link: &TNodeLink) -> NodeResult<()> {
        {
            let world = self.world_mut()?;
            if let Some(mut links_map) = world.get_resource_mut::<LinksMapResource>() {
                links_map.insert(link.clone());
            }
        }

        if Self::is_component_link(link) {
            self.handle_component_replacement(link.parent, link.child_kind.to_kind())?;
        }

        match self {
            Sources::Solid(_) => {
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
            _ => {}
        }

        // Universal component merging for all sources that process links
        if Self::is_component_link(link)
            && self.entity(link.parent).is_ok()
            && self.entity(link.child).is_ok()
        {
            if matches!(self, Sources::Top(..)) {
                debug!("try merge {link:?}");
            }
            self.try_merge_component_entities(link.parent, link.child)?;
        }

        Ok(())
    }

    fn handle_link_delete(&mut self, link: &TNodeLink) -> NodeResult<()> {
        {
            let world = self.world_mut()?;
            if let Some(mut links_map) = world.get_resource_mut::<LinksMapResource>() {
                links_map.remove(link.id);
            }
        }
        match self {
            Sources::Solid(_) => {
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
        {
            let world = self.world_mut()?;
            if let Some(mut links_map) = world.get_resource_mut::<LinksMapResource>() {
                links_map.remove(old.id);
                links_map.insert(new.clone());
            }
        }

        match self {
            Sources::Solid(_) => {
                if old.solid && !new.solid {
                    self.remove_link(old.parent, old.child)?;
                } else if !old.solid && new.solid {
                    self.add_link(new.parent, new.child)?;
                }
            }
            Sources::Top(_) => {
                self.handle_top_link_update(old, new)?;
            }
            _ => {}
        }

        // Universal component merging when link becomes solid and is component type
        if !old.solid
            && new.solid
            && Self::is_component_link(new)
            && self.entity(new.parent).is_ok()
            && self.entity(new.child).is_ok()
        {
            self.try_merge_component_entities(new.parent, new.child)?;
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
                        .and_then(|links| links.get(selection.selected_link_id))
                        .cloned()
                        .ok_or_else(|| {
                            NodeError::custom(format!(
                                "Link#{} not found",
                                selection.selected_link_id
                            ))
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
                        .get(old.selected_link_id)
                        .cloned()
                        .ok_or_else(|| NodeError::custom("Old link not found"))?;
                    let new_link = links_map
                        .get(new.selected_link_id)
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
                        .and_then(|links| links.get(selection.selected_link_id))
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
        if matches!(self, Sources::Top(..)) {
            debug!("spawn {self} {kind} {node_id} {target_entity}");
        }
        self.check_pending_merges_for_node(node_id)?;
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
        let mut candidate_entities = Vec::new();
        for child_kind in kind.component_children() {
            if let Ok(children) = self.get_children_of_kind(node_id, child_kind) {
                for child_id in children {
                    if let Ok(entity) = self.entity(child_id) {
                        candidate_entities.push(entity);
                    }
                }
            }
        }
        if let Some(parent_kind) = kind.component_parent() {
            if let Ok(parents) = self.get_parents_of_kind(node_id, parent_kind) {
                for parent_id in parents {
                    if let Ok(entity) = self.entity(parent_id) {
                        candidate_entities.push(entity);
                    }
                }
            }
        }
        let target_entity = if let Some(&first_entity) = candidate_entities.first() {
            self.merge_component_entities(node_id, kind, &candidate_entities)?;
            first_entity
        } else {
            let world = self.world_mut()?;
            world.spawn_empty().id()
        };
        Ok(target_entity)
    }

    fn merge_component_entities(
        &mut self,
        _node_id: u64,
        _kind: NodeKind,
        entities: &[Entity],
    ) -> NodeResult<()> {
        if entities.len() <= 1 {
            return Ok(());
        }

        let world = self.world_mut()?;
        let primary_entity = entities[0];

        let nodes_to_migrate: Vec<(Entity, Vec<(u64, NodeKind)>)> = {
            let node_map = world
                .get_resource::<NodesMapResource>()
                .ok_or_else(|| NodeError::custom("NodesMapResource not found"))?;

            entities[1..]
                .iter()
                .map(|&entity| {
                    let nodes = node_map
                        .entity_to_nodes
                        .get(&entity)
                        .unwrap_or(&Vec::new())
                        .iter()
                        .filter_map(|&node_id| {
                            node_map.get_kind(node_id).map(|kind| (node_id, kind))
                        })
                        .collect();
                    (entity, nodes)
                })
                .collect()
        };

        for (entity, nodes) in nodes_to_migrate.clone() {
            for (_migrate_id, migrate_kind) in &nodes {
                Self::migrate_node_component(world, entity, primary_entity, *migrate_kind)?;
            }

            if let Some(mut node_map) = world.get_resource_mut::<NodesMapResource>() {
                for (migrate_id, migrate_kind) in nodes {
                    node_map.insert(migrate_id, migrate_kind, primary_entity);
                }
            }
            world.despawn(entity);
        }
        if matches!(self, Sources::Top(..)) {
            debug!("despawn {nodes_to_migrate:?} {_node_id}");
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

    fn handle_component_replacement(
        &mut self,
        parent_id: u64,
        child_kind: NodeKind,
    ) -> NodeResult<()> {
        let is_top_source = matches!(self, Sources::Top(..));

        if is_top_source {
            debug!(
                "handle_component_replacement: parent_id={}, child_kind={:?}",
                parent_id, child_kind
            );
        }

        // Check if parent entity already has a component of this kind
        let parent_entity = self.entity(parent_id)?;

        // Find existing component of the same kind on parent entity
        let existing_component_id = {
            let world = self.world_mut()?;
            let node_map = world
                .get_resource::<NodesMapResource>()
                .ok_or_else(|| NodeError::custom("NodesMapResource not found"))?;

            if is_top_source {
                debug!(
                    "Parent entity {:?} has nodes: {:?}",
                    parent_entity,
                    node_map.entity_to_nodes.get(&parent_entity)
                );
            }

            node_map
                .entity_to_nodes
                .get(&parent_entity)
                .and_then(|nodes| {
                    nodes
                        .iter()
                        .find(|&&node_id| {
                            let matches_kind = node_map
                                .get_kind(node_id)
                                .map_or(false, |kind| kind == child_kind && node_id != parent_id);
                            if is_top_source && matches_kind {
                                debug!(
                                    "Found existing component node_id={} of kind {:?}",
                                    node_id, child_kind
                                );
                            }
                            matches_kind
                        })
                        .copied()
                })
        };

        if let Some(old_component_id) = existing_component_id {
            if is_top_source {
                debug!(
                    "Extracting old component {} to new entity",
                    old_component_id
                );
            }
            // Extract the old component and its children to a new entity
            self.extract_component_to_new_entity(old_component_id)?;
        } else if is_top_source {
            debug!("No existing component of kind {:?} found", child_kind);
        }

        Ok(())
    }

    fn extract_component_to_new_entity(&mut self, component_id: u64) -> NodeResult<Entity> {
        let world = self.world_mut()?;
        let current_entity = {
            let node_map = world
                .get_resource::<NodesMapResource>()
                .ok_or_else(|| NodeError::custom("NodesMapResource not found"))?;
            node_map
                .entities
                .get(&component_id)
                .copied()
                .ok_or_else(|| NodeError::custom("Component entity not found"))?
        };

        // Create a new entity for the extracted component
        let new_entity = world.spawn_empty().id();

        // Collect all component nodes to extract (the component and its children)
        let nodes_to_extract = self.collect_component_tree(component_id)?;

        let world = self.world_mut()?;

        // Migrate each node to the new entity
        for (node_id, node_kind) in &nodes_to_extract {
            Self::migrate_node_component(world, current_entity, new_entity, *node_kind)?;
        }

        // Update node mappings
        if let Some(mut node_map) = world.get_resource_mut::<NodesMapResource>() {
            for (node_id, node_kind) in nodes_to_extract {
                node_map.insert(node_id, node_kind, new_entity);
            }
        }

        Ok(new_entity)
    }

    fn collect_component_tree(&self, root_id: u64) -> NodeResult<Vec<(u64, NodeKind)>> {
        let world = self.world()?;
        let node_map = world
            .get_resource::<NodesMapResource>()
            .ok_or_else(|| NodeError::custom("NodesMapResource not found"))?;

        let mut result = Vec::new();
        let mut to_visit = vec![root_id];
        let mut visited = HashSet::new();

        while let Some(node_id) = to_visit.pop() {
            if !visited.insert(node_id) {
                continue;
            }

            if let Some(kind) = node_map.get_kind(node_id) {
                result.push((node_id, kind));

                // Get component children of this node
                let children_kinds = kind.component_children();
                if !children_kinds.is_empty() {
                    // Find actual child nodes on the same entity
                    let entity = node_map.entities.get(&node_id).copied();
                    if let Some(entity) = entity {
                        if let Some(entity_nodes) = node_map.entity_to_nodes.get(&entity) {
                            for &child_id in entity_nodes {
                                if let Some(child_kind) = node_map.get_kind(child_id) {
                                    if children_kinds.contains(&child_kind) {
                                        to_visit.push(child_id);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    fn try_merge_component_entities(&mut self, parent_id: u64, child_id: u64) -> NodeResult<()> {
        let parent_entity = self.entity(parent_id)?;
        let child_entity = self.entity(child_id)?;

        if parent_entity != child_entity {
            if matches!(self, Sources::Top(..)) {
                debug!("merge {parent_entity} {child_entity}");
            }
            let world = self.world_mut()?;

            // Get all node components from child entity to migrate
            let nodes_to_migrate: Vec<(u64, NodeKind)> = {
                let node_map = world
                    .get_resource::<NodesMapResource>()
                    .ok_or_else(|| NodeError::custom("NodesMapResource not found"))?;

                node_map
                    .entity_to_nodes
                    .get(&child_entity)
                    .unwrap_or(&Vec::new())
                    .iter()
                    .filter_map(|&node_id| node_map.get_kind(node_id).map(|kind| (node_id, kind)))
                    .collect()
            };

            // Migrate each component from child to parent entity
            for (_migrate_id, migrate_kind) in &nodes_to_migrate {
                Self::migrate_node_component(world, child_entity, parent_entity, *migrate_kind)?;
            }

            // Update node mappings to point to parent entity
            if let Some(mut node_map) = world.get_resource_mut::<NodesMapResource>() {
                for (migrate_id, migrate_kind) in &nodes_to_migrate {
                    node_map.insert(*migrate_id, *migrate_kind, parent_entity);
                }
            }

            // Check if child entity has any remaining nodes after migration
            let remaining_nodes = {
                let node_map = world.get_resource::<NodesMapResource>().unwrap();
                node_map
                    .entity_to_nodes
                    .get(&child_entity)
                    .map(|nodes| nodes.len())
                    .unwrap_or(0)
            };

            // Only despawn child entity if it has no remaining nodes
            if remaining_nodes == 0 {
                world.despawn(child_entity);
                if matches!(self, Sources::Top(..)) {
                    debug!("despawn empty entity {child_entity}");
                }
            } else if matches!(self, Sources::Top(..)) {
                debug!(
                    "keeping entity {child_entity} with {} remaining nodes",
                    remaining_nodes
                );
            }
        }
        Ok(())
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

    fn check_pending_merges_for_node(&mut self, node_id: u64) -> NodeResult<()> {
        // Collect all potential merges first to avoid borrow checker issues
        let mut merges_to_process = Vec::new();

        // Check all links where this node is a parent
        if let Ok(children_data) = self.get_links_data() {
            let all_children: Vec<u64> = children_data
                .children
                .get(&node_id)
                .map(|child_map| child_map.values().flatten().cloned().collect())
                .unwrap_or_default();

            for child_id in all_children {
                if self.entity(child_id).is_ok() {
                    merges_to_process.push((node_id, child_id));
                }
            }
        }

        // Check all links where this node is a child
        if let Ok(links_data) = self.get_links_data() {
            for (parent_id, child_map) in &links_data.children {
                for child_ids in child_map.values() {
                    if child_ids.contains(&node_id) && self.entity(*parent_id).is_ok() {
                        merges_to_process.push((*parent_id, node_id));
                    }
                }
            }
        }

        // Process all merges
        for (parent_id, child_id) in merges_to_process {
            self.try_merge_component_entities(parent_id, child_id)?;
        }

        Ok(())
    }

    fn handle_top_link_insert(&mut self, link: &TNodeLink) -> NodeResult<()> {
        let child_kind = link.child_kind.to_kind();
        let world = self.world_mut()?;
        let mut ratings = world.resource_mut::<LinkRatings>();
        ratings.add_rating(link.parent, link.child, child_kind, link.rating);
        if let Some(top_child) = ratings.get_top(link.parent, child_kind) {
            if let Ok(old_children) = self.get_children_of_kind(link.parent, child_kind) {
                for old_child in old_children {
                    self.remove_link(link.parent, old_child)?;
                }
            }
            self.add_link(link.parent, top_child)?;
        }
        Ok(())
    }

    fn handle_top_link_update(&mut self, old: &TNodeLink, new: &TNodeLink) -> NodeResult<()> {
        let child_kind = new
            .child_kind
            .parse::<NodeKind>()
            .map_err(|_| NodeError::custom(format!("Invalid child kind: {}", new.child_kind)))?;

        let world = self.world_mut()?;
        let mut links_map = world.resource_mut::<LinksMapResource>();
        if old.id != new.id {
            links_map.remove(old.id);
        }
        links_map.insert(new.clone());
        let mut ratings = world.resource_mut::<LinkRatings>();
        ratings.remove_rating(old.parent, old.child, child_kind);
        ratings.add_rating(new.parent, new.child, child_kind, new.rating);
        if let Some(top_child) = ratings.get_top(new.parent, child_kind) {
            if let Ok(old_children) = self.get_children_of_kind(new.parent, child_kind) {
                for old_child in old_children {
                    self.remove_link(new.parent, old_child)?;
                }
            }
            self.add_link(new.parent, top_child)?;
        }
        Ok(())
    }

    fn handle_top_link_delete(&mut self, link: &TNodeLink) -> NodeResult<()> {
        let child_kind = link
            .child_kind
            .parse::<NodeKind>()
            .map_err(|_| NodeError::custom(format!("Invalid child kind: {}", link.child_kind)))?;

        // First collect old children before modifying ratings
        let old_children = self
            .get_children_of_kind(link.parent, child_kind)
            .unwrap_or_default();

        // Then update ratings and get new top child
        let top_child_after_removal = {
            let world = self.world_mut()?;

            let mut links_map = world.resource_mut::<LinksMapResource>();
            links_map.remove(link.id);

            let mut ratings = world.resource_mut::<LinkRatings>();
            ratings.remove_rating(link.parent, link.child, child_kind);
            ratings.get_top(link.parent, child_kind)
        };

        // Remove all existing links of this kind first
        for old_child in old_children {
            self.remove_link(link.parent, old_child)?;
        }

        // Update links to reflect new top-rated child
        if let Some(top_child) = top_child_after_removal {
            self.add_link(link.parent, top_child)?;
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
            Sources::Solid(w) | Sources::Top(w) | Sources::Selected(w) => Ok(w),
            Sources::Battle(sim, _) => Ok(&sim.world),
            Sources::SourceRef(source) => source.world(),
            Sources::None => Err(NodeError::custom("No world available")),
        }
    }

    fn world_mut(&mut self) -> NodeResult<&mut World> {
        match self {
            Sources::Solid(w) | Sources::Top(w) | Sources::Selected(w) => Ok(w),
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
