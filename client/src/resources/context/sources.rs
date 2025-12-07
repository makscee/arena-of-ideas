use super::*;
/// Static sources for mirroring stdb state
pub struct StaticSources {
    pub solid: Sources<'static>,
    pub core: Sources<'static>,
    pub incubator: Sources<'static>,
}

impl StaticSources {
    pub fn iter(&self) -> impl Iterator<Item = &Sources<'static>> {
        [&self.solid, &self.core, &self.incubator].into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Sources<'static>> {
        [&mut self.solid, &mut self.core, &mut self.incubator].into_iter()
    }
}

impl StaticSources {
    pub fn new() -> Self {
        Self {
            solid: Sources::new_solid(),
            core: Sources::new_core(),
            incubator: Sources::new_incubator(),
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

pub fn with_incubator_source<R, F>(f: F) -> NodeResult<R>
where
    F: FnOnce(&mut ClientContext) -> NodeResult<R>,
{
    with_static_sources(|sources| {
        if matches!(sources.incubator, Sources::None) {
            panic!("Double take of Incubator source");
        }
        let taken = std::mem::replace(&mut sources.incubator, Sources::None);
        let mut context = taken.as_context();
        let result = f(&mut context);
        sources.incubator = context.into_source();
        result
    })
}

pub trait ClientSource {
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
    Incubator(World),
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

    pub fn new_incubator() -> Self {
        let mut world = World::new();
        Self::init_world(&mut world);
        Sources::Incubator(world)
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
            Sources::Solid(world) | Sources::Core(world) | Sources::Incubator(world) => Ok(world),
            Sources::Battle(sim, _) => Ok(&sim.world),
            Sources::SourceRef(s) => s.world(),
            Sources::None => Err(NodeError::custom("No world in empty source")),
        }
    }

    pub fn world_mut(&mut self) -> NodeResult<&mut World> {
        match self {
            Sources::Solid(world) | Sources::Core(world) | Sources::Incubator(world) => Ok(world),
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
        let kind = self
            .get_node_kind(node_id)
            .track()?
            .with_other_components()
            .into_iter()
            .find(|k| k.var_names().contains(&var))
            .to_not_found()
            .track()?;
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
        let world = self.world()?;
        if let Some(component) = world.get::<T>(entity) {
            if component.id() == node_id {
                return Ok(component);
            }
            if let Some(extras) = world.get::<ExtraNodes<T>>(entity) {
                if let Some(extra) = extras.get(node_id) {
                    return Ok(extra);
                }
                return Err(NodeError::not_found(node_id));
            }
            return Ok(component);
        }

        world
            .get::<ExtraNodes<T>>(entity)
            .and_then(|extras| extras.get(node_id))
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
        }
        Ok(())
    }

    fn should_process_node(&self, node: &TNode) -> bool {
        match self {
            Sources::Solid(_) => true,
            Sources::Core(_) => node.owner == ID_CORE,
            Sources::Incubator(_) => node.owner == ID_CORE || node.owner == ID_INCUBATOR,
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
            Sources::Solid(_) => true,
            Sources::Core(_) => check_owner(link, vec![ID_CORE]),
            Sources::Incubator(_) => check_owner(link, vec![ID_INCUBATOR, ID_CORE]),
            Sources::Battle(..) | Sources::None | Sources::SourceRef(..) => false,
        }
    }

    fn handle_link_insert(&mut self, link: &TNodeLink) -> NodeResult<()> {
        let is_incubator = matches!(self, Sources::Incubator(..));
        let world = self.world_mut()?;
        if let Some(mut links_map) = world.get_resource_mut::<LinksMapResource>() {
            links_map.insert(link.clone());
        }
        if is_incubator {
            debug!("link_insert {link:?}");
        }
        // Handle component links specially
        if Self::is_component_link(link) {
            let parent_kind = link.parent_kind.to_kind();
            let child_kind = link.child_kind.to_kind();
            let is_many_to_one = NodeKind::get_relation(parent_kind, child_kind)
                .is_some_and(|rel| rel == schema::NodeRelation::ManyToOne);

            if is_many_to_one {
                // For many-to-one components, clone the component tree to the parent's entity
                if self.entity(link.parent).is_ok() && self.entity(link.child).is_ok() {
                    let parent_entity = self.entity(link.parent)?;
                    self.clone_component_tree_for_link(link.child, parent_entity)?;
                }
            } else {
                if is_incubator {
                    debug!(
                        "entities {:?} {:?}",
                        self.entity(link.parent),
                        self.entity(link.child)
                    );
                }
                let parent_entity = self.entity(link.parent)?;
                let _ = self.entity(link.child)?;
                self.merge_component_with_rating_check(link.child, parent_entity, child_kind)?;
            }
        }

        match self {
            Sources::Solid(_) | Sources::Core(_) | Sources::Incubator(_) => {
                self.get_links_data_mut()?.add_link(
                    link.parent,
                    link.child,
                    link.parent_kind.to_kind(),
                    link.child_kind.to_kind(),
                );
            }
            Sources::Battle(..) | Sources::SourceRef(..) | Sources::None => {}
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
            Sources::Solid(_) | Sources::Core(_) | Sources::Incubator(_) => {
                self.remove_link(link.parent, link.child)?;
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
            Sources::Solid(_) | Sources::Core(_) | Sources::Incubator(_) => {
                self.add_link(new.parent, new.child)?;
            }
            Sources::Battle(..) | Sources::SourceRef(..) | Sources::None => {}
        }

        // Handle component links when they are added
        if Self::is_component_link(new) {
            // Handle this as a new link insertion for component merging
            if self.entity(new.parent).is_ok() && self.entity(new.child).is_ok() {
                let parent_kind = new.parent_kind.to_kind();
                let child_kind = new.child_kind.to_kind();
                let is_many_to_one = NodeKind::get_relation(parent_kind, child_kind)
                    .is_some_and(|rel| rel == schema::NodeRelation::ManyToOne);

                if is_many_to_one {
                    // For many-to-one components, clone the component tree to the parent's entity
                    if self.entity(new.parent).is_ok() && self.entity(new.child).is_ok() {
                        let parent_entity = self.entity(new.parent)?;
                        self.clone_component_tree_for_link(new.child, parent_entity)?;
                    }
                }
            } else {
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

    fn spawn_node_with_entity_merging(
        &mut self,
        node_id: u64,
        kind: NodeKind,
        node_data: &TNode,
    ) -> NodeResult<()> {
        let target_entity = self.find_or_create_component_entity(node_id, kind)?;

        if let Some(parent_kind) = kind.component_parent() {
            if let Ok(parents) = self.get_parents_of_kind(node_id, parent_kind) {
                for parent_id in parents {
                    if let Ok(parent_entity) = self.entity(parent_id) {
                        let _ = self.check_and_handle_promotion_from_extras(
                            node_id,
                            kind,
                            parent_entity,
                            node_data,
                        );
                    }
                }
            }
        }

        let world = self.world_mut()?;
        node_kind_match!(kind, {
            let node = node_data.to_node::<NodeType>()?;

            let is_in_extras = world
                .get::<ExtraNodes<NodeType>>(target_entity)
                .map(|extras| extras.contains(node_id))
                .unwrap_or(false);

            if is_in_extras {
                if let Some(mut extras_container) = world
                    .entity_mut(target_entity)
                    .get_mut::<ExtraNodes<NodeType>>()
                {
                    if let Some(extra_comp) = extras_container.get_mut(node_id) {
                        *extra_comp = node;
                    }
                }
            } else {
                world.entity_mut(target_entity).insert(node);
                if let Some(mut node_map) = world.get_resource_mut::<NodesMapResource>() {
                    node_map.insert(node_id, kind, target_entity);
                }
            }
        });
        debug!("spawn {self} {kind} {node_id} {target_entity}");

        Ok(())
    }

    fn check_and_handle_promotion_from_extras(
        &mut self,
        node_id: u64,
        kind: NodeKind,
        parent_entity: Entity,
        node_data: &TNode,
    ) -> NodeResult<()> {
        node_kind_match!(kind, {
            let new_node = node_data.to_node::<NodeType>()?;
            let new_rating = new_node.rating();

            let world = self.world()?;

            let is_in_extras = world
                .get::<ExtraNodes<NodeType>>(parent_entity)
                .map(|extras| extras.contains(node_id))
                .unwrap_or(false);

            if !is_in_extras {
                return Ok(());
            }

            if let Some(existing_main) = world.get::<NodeType>(parent_entity) {
                let existing_rating = existing_main.rating();
                let existing_id = existing_main.id();

                if new_rating > existing_rating
                    || (new_rating == existing_rating && node_id < existing_id)
                {
                    let world = self.world_mut()?;
                    let existing_comp = world.entity_mut(parent_entity).take::<NodeType>().unwrap();
                    let existing_id = existing_comp.id();

                    if !world
                        .entity(parent_entity)
                        .contains::<ExtraNodes<NodeType>>()
                    {
                        world
                            .entity_mut(parent_entity)
                            .insert(ExtraNodes::<NodeType>::new());
                    }

                    world
                        .entity_mut(parent_entity)
                        .get_mut::<ExtraNodes<NodeType>>()
                        .unwrap()
                        .add(existing_id, existing_comp);

                    world
                        .entity_mut(parent_entity)
                        .get_mut::<ExtraNodes<NodeType>>()
                        .unwrap()
                        .remove(node_id);

                    if let Some(mut node_map) = world.get_resource_mut::<NodesMapResource>() {
                        node_map.insert(node_id, kind, parent_entity);
                        node_map.insert(existing_id, kind, parent_entity);
                    }
                }
            }
        });

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
            match NodeKind::get_relation(kind, child_kind).unwrap() {
                NodeRelation::ManyToOne => continue,
                NodeRelation::OneToOne | NodeRelation::OneToMany => {}
            }
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

    fn merge_component_with_rating_check(
        &mut self,
        child_id: u64,
        parent_entity: Entity,
        child_kind: NodeKind,
    ) -> NodeResult<()> {
        node_kind_match!(child_kind, {
            self.merge_component_with_rating_check_impl::<NodeType>(
                child_id,
                parent_entity,
                child_kind,
            )
        })
    }

    fn merge_component_with_rating_check_impl<T: ClientNode + BevyComponent>(
        &mut self,
        child_id: u64,
        parent_entity: Entity,
        child_kind: NodeKind,
    ) -> NodeResult<()> {
        let new_rating = {
            let world = self.world()?;
            world.get::<T>(self.entity(child_id)?).map(|n| n.rating())
        };

        let existing_main_node_id = {
            let world = self.world()?;
            let node_map = world
                .get_resource::<NodesMapResource>()
                .ok_or_else(|| NodeError::custom("NodesMapResource not found"))?;

            node_map
                .entity_to_nodes
                .get(&parent_entity)
                .and_then(|nodes| {
                    nodes
                        .iter()
                        .find(|&&nid| node_map.get_kind(nid).map_or(false, |k| k == child_kind))
                        .copied()
                })
        };

        let existing_main_rating = {
            let world = self.world()?;
            world.get::<T>(parent_entity).map(|n| n.rating())
        };

        debug!(
            "merge_component_with_rating_check: child_id={}, new_rating={:?}, existing_main_rating={:?}",
            child_id, new_rating, existing_main_rating
        );

        let should_replace_existing = match (new_rating, existing_main_rating) {
            (Some(new_r), Some(existing_r)) => {
                if new_r > existing_r {
                    true
                } else if new_r == existing_r {
                    if let Some(existing_id) = existing_main_node_id {
                        child_id < existing_id
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            _ => false,
        };

        match (new_rating, existing_main_rating) {
            (Some(new_r), Some(existing_r)) if should_replace_existing => {
                debug!(
                    "  Case 1: new ({}) > existing ({}) or same rating with lower id, moving existing to extras",
                    new_r, existing_r
                );
                let world = self.world_mut()?;

                let existing_comp = world.entity_mut(parent_entity).take::<T>().unwrap();
                if let Some(existing_id) = existing_main_node_id {
                    if !world.entity(parent_entity).contains::<ExtraNodes<T>>() {
                        world
                            .entity_mut(parent_entity)
                            .insert(ExtraNodes::<T>::new());
                    }
                    world
                        .entity_mut(parent_entity)
                        .get_mut::<ExtraNodes<T>>()
                        .unwrap()
                        .add(existing_id, existing_comp);
                }

                self.merge_component_entities_for_link(child_id, parent_entity)?;
            }
            (Some(_new_r), Some(_existing_r)) => {
                debug!(
                    "  Case 2: new ({}) <= existing ({}), moving new to extras",
                    _new_r, _existing_r
                );
                let child_entity = self.entity(child_id)?;
                let world = self.world_mut()?;

                if let Some(child_comp) = world.entity_mut(child_entity).take::<T>() {
                    if !world.entity(parent_entity).contains::<ExtraNodes<T>>() {
                        world
                            .entity_mut(parent_entity)
                            .insert(ExtraNodes::<T>::new());
                    }
                    world
                        .entity_mut(parent_entity)
                        .get_mut::<ExtraNodes<T>>()
                        .unwrap()
                        .add(child_id, child_comp);
                }

                if let Some(mut node_map) = world.get_resource_mut::<NodesMapResource>() {
                    node_map.insert(child_id, child_kind, parent_entity);
                }

                if child_entity != parent_entity {
                    let should_despawn = {
                        let node_map = world
                            .get_resource::<NodesMapResource>()
                            .ok_or_else(|| NodeError::custom("NodesMapResource not found"))?;
                        node_map
                            .entity_to_nodes
                            .get(&child_entity)
                            .map_or(true, |nodes| nodes.is_empty())
                    };
                    if should_despawn {
                        world.despawn(child_entity);
                    }
                }
            }
            _ => {
                debug!("  Case 3: no existing main component, normal merge");
                self.merge_component_entities_for_link(child_id, parent_entity)?;
            }
        }

        Ok(())
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

    fn clone_component_tree_for_link(
        &mut self,
        child_id: u64,
        target_entity: Entity,
    ) -> NodeResult<()> {
        let nodes_to_clone = self.collect_component_tree_with_recursive_children(child_id)?;
        if nodes_to_clone.is_empty() {
            return Ok(());
        }
        let world = self.world_mut()?;
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
            return Ok(());
        }
        for (_node_id, node_kind) in &nodes_to_clone {
            Self::clone_node_component(world, source_entity, target_entity, *node_kind)?;
        }
        if let Some(mut node_map) = world.get_resource_mut::<NodesMapResource>() {
            for (node_id, node_kind) in nodes_to_clone {
                node_map.insert(node_id, node_kind, target_entity);
            }
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

    fn clone_node_component(
        world: &mut World,
        from_entity: Entity,
        to_entity: Entity,
        kind: NodeKind,
    ) -> NodeResult<()> {
        node_kind_match!(kind, {
            let component = world.entity(from_entity).get::<NodeType>().cloned();
            if let Some(component) = component {
                world.entity_mut(to_entity).insert(component);
            }
        });
        Ok(())
    }
}

impl ClientSource for Sources<'_> {
    fn world(&self) -> NodeResult<&World> {
        match self {
            Sources::Solid(w) | Sources::Core(w) | Sources::Incubator(w) => Ok(w),
            Sources::Battle(sim, _) => Ok(&sim.world),
            Sources::SourceRef(source) => source.world(),
            Sources::None => Err(NodeError::custom("No world available")),
        }
    }

    fn world_mut(&mut self) -> NodeResult<&mut World> {
        match self {
            Sources::Solid(w) | Sources::Core(w) | Sources::Incubator(w) => Ok(w),
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
    NodeUpdate { old: TNode, new: TNode },
    NodeDelete(TNode),
    LinkInsert(TNodeLink),
    LinkUpdate { old: TNodeLink, new: TNodeLink },
    LinkDelete(TNodeLink),
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
        for kind in kind.with_other_components() {
            node_kind_match!(kind, {
                if self
                    .load_mut::<NodeType>(node_id)
                    .is_ok_and(|mut n| n.set_var(var, value.clone()).is_ok())
                {
                    break;
                }
            });
        }
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
        let parent_kind = self.get_node_kind(parent_id).track()?;
        let child_kind = self.get_node_kind(child_id).track()?;
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

    fn insert_node(
        &mut self,
        id: u64,
        owner: u64,
        data: String,
        node_kind: NodeKind,
    ) -> NodeResult<()> {
        node_kind_match!(node_kind, {
            let mut node = NodeType::default();
            node.inject_data(&data)?;
            node.set_id(id);
            node.set_owner(owner);

            let entity = self.find_or_create_component_entity(id, node_kind)?;
            let world = self.world_mut()?;

            world.entity_mut(entity).insert(node);

            if let Some(mut node_data) = world.get_resource_mut::<NodesMapResource>() {
                node_data.insert(id, node_kind, entity);
            }
        });

        Ok(())
    }

    fn delete_node(&mut self, node_id: u64) -> NodeResult<()> {
        // Clear all links
        self.clear_links(node_id)?;

        // Get node kind before removing from map
        let kind = self.get_node_kind(node_id);

        // Remove from NodesMapResource and get entity
        if let Some(entity) = self.get_nodes_map_mut()?.remove(node_id) {
            // Check if there are other nodes on this entity
            let other_nodes_exist = {
                let node_map = self.world()?.get_resource::<NodesMapResource>();
                node_map.map_or(false, |nm| !nm.get_node_ids(entity).is_empty())
            };

            if other_nodes_exist {
                // Only remove the component for this specific node, don't despawn
                let world = self.world_mut()?;
                let kind = kind?;
                node_kind_match!(kind, {
                    world.entity_mut(entity).remove::<NodeType>();
                });
            } else {
                // No other nodes on this entity, despawn it
                self.world_mut()?.despawn(entity);
            }
        }

        Ok(())
    }
}
