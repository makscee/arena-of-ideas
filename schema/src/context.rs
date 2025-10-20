use super::*;
use crate::node_error::NodeResult;
use std::collections::{HashMap, HashSet, VecDeque};

/// This system provides a single, consistent way to access node state through:
/// 1. Load-Edit-Save pattern for node modifications
/// 2. Generated field accessors for type-safe var operations
/// 3. Context-aware var access with history support
///
/// USAGE PATTERNS:
///
/// 1. Load node using node_kind_match!:
///    ```rust
///    let kind = ctx.get_node_kind(id)?;
///    node_kind_match!(kind, {
///        let mut node = ctx.load_and_get::<NodeType>(id)?;
///        // ... edit node
///        node.save(ctx)?;
///    });
///    ```
///
/// 2. Edit vars via generated field setters:
///    - node.hp_set(value) - direct field modification + dirty marking
///    - node.set_var(var, value) - maps to appropriate field setter
///    - node.set_ctx_var(ctx, var, value) - updates field + pushes to history
///
/// 3. Read vars via generated field getters:
///    - node.hp_get() - direct field access
///    - node.get_var(var) - maps to appropriate field getter
///    - node.get_ctx_var(ctx, var) - checks history first, then field
///
/// 4. Context var access with recursive parent lookup:
///    - ctx.owner_var(var) - gets from owner, then parents recursively
///    - ctx.target_var(var) - gets from target, then parents recursively
///    - ctx.caster_var(var) - gets from caster, then parents recursively
///    - ctx.get_var(var) - compatibility method, tries all contexts
///
/// 5. Save pattern handles everything:
///    - Checks is_dirty flag
///    - Saves child nodes recursively
///    - Updates NodeStateHistory for battle contexts
///    - Persists to storage via ContextSource
///    - Updates links between nodes
///    - Clears dirty flag
pub trait Node: Send + Sync + Default + StringData {
    fn with_owner(mut self, owner: u64) -> Self {
        self.set_owner(owner);
        self
    }
    fn with_id(mut self, id: u64) -> Self {
        self.set_id(id);
        self
    }
    fn id(&self) -> u64;
    fn set_id(&mut self, id: u64);
    fn owner(&self) -> u64;
    fn set_owner(&mut self, owner: u64);
    fn kind(&self) -> NodeKind {
        Self::kind_s()
    }
    fn reassign_ids(&mut self, next_id: &mut u64);
    fn kind_s() -> NodeKind
    where
        Self: Sized;

    fn var_names() -> HashSet<VarName>
    where
        Self: Sized;

    // Direct var access - just maps to node fields
    fn set_var(&mut self, var: VarName, value: VarValue) -> NodeResult<()>;
    fn get_var(&self, var: VarName) -> NodeResult<VarValue>;
    fn get_vars(&self) -> HashMap<VarName, VarValue>;

    // Context-aware var access - goes through history for battle context
    fn set_ctx_var<S: ContextSource>(
        &mut self,
        ctx: &mut Context<S>,
        var: VarName,
        value: VarValue,
    ) -> NodeResult<()> {
        // First set on the node itself
        self.set_var(var, value.clone())?;
        // Then push to context (e.g., NodeStateHistory for battles)
        ctx.source_mut().set_var(self.id(), var, value)
    }

    fn get_ctx_var<S: ContextSource>(
        &self,
        ctx: &Context<S>,
        var: VarName,
    ) -> NodeResult<VarValue> {
        // Try to get from context first (e.g., NodeStateHistory)
        ctx.source()
            .get_var_direct(self.id(), var)
            .or_else(|_| self.get_var(var))
    }

    fn save<S: ContextSource>(&mut self, ctx: &mut Context<S>) -> NodeResult<()>;
    fn set_dirty(&mut self, value: bool);
    fn is_dirty(&self) -> bool;

    fn pack(&self) -> PackedNodes {
        let mut packed = PackedNodes::default();
        let mut visited = std::collections::HashSet::new();
        self.pack_recursive(&mut packed, &mut visited);
        packed.root = self.id();
        packed
    }

    fn pack_recursive(
        &self,
        packed: &mut PackedNodes,
        visited: &mut std::collections::HashSet<u64>,
    ) {
        let id = self.id();
        if visited.contains(&id) {
            return;
        }
        visited.insert(id);

        // Add this node's data
        packed.add_node(self.kind().to_string(), self.get_data(), id);

        // This will be implemented by the generated code for each node type
        // to handle their specific linked fields
        self.pack_links(packed, visited);
    }

    fn unpack(packed: &PackedNodes) -> NodeResult<Self> {
        let root_data = packed
            .get(packed.root)
            .ok_or_else(|| NodeError::custom("Root node not found in packed data"))?;

        let mut node = Self::default();
        node.inject_data(&root_data.data)?;
        node.set_id(packed.root);

        node.unpack_links(packed);
        Ok(node)
    }

    fn pack_links(&self, packed: &mut PackedNodes, visited: &mut std::collections::HashSet<u64>);

    fn unpack_links(&mut self, packed: &PackedNodes);
}

// Helper trait for converting between node types
pub trait NodeKindConvert {
    fn to_node_kind(&self) -> NodeKind;
    fn from_node_kind(kind: NodeKind) -> Option<Self>
    where
        Self: Sized;
}

// Context system

/// Source for Context to retrieve node data and manage links
/// Trait for loading specific node types and calling their methods

pub trait ContextSource {
    /// Get the node kind for a given ID
    fn get_node_kind(&self, node_id: u64) -> NodeResult<NodeKind>;

    /// Get all child nodes
    fn get_children(&self, from_id: u64) -> NodeResult<Vec<u64>>;

    /// Get all child nodes of a specific kind
    fn get_children_of_kind(&self, from_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>>;

    /// Get all parent nodes
    fn get_parents(&self, to_id: u64) -> NodeResult<Vec<u64>>;

    /// Get all parent nodes of a specific kind
    fn get_parents_of_kind(&self, to_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>>;

    /// Add a link between nodes
    fn add_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()>;

    /// Remove a link between nodes
    fn remove_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()>;

    /// Check if two nodes are linked
    fn is_linked(&self, from_id: u64, to_id: u64) -> NodeResult<bool>;

    /// Insert a new node
    fn insert_node(&mut self, id: u64, owner: u64, kind: NodeKind, data: String) -> NodeResult<()>;

    /// Delete a node
    fn delete_node(&mut self, id: u64) -> NodeResult<()>;

    /// Get a variable value directly from a node (no recursion, may check history)
    fn get_var_direct(&self, node_id: u64, var: VarName) -> NodeResult<VarValue>;

    /// Set a variable value on a node (may update history for battle context)
    fn set_var(&mut self, node_id: u64, var: VarName, value: VarValue) -> NodeResult<()>;
}

/// Context layer for scoping operations
#[derive(Debug, Clone)]
pub enum ContextLayer {
    Owner(u64),
    Target(u64),
    Caster(u64),
    Status(u64),
    Var(VarName, VarValue),
}

/// Generic context that wraps a ContextSource
pub struct Context<S> {
    source: S,
    layers: Vec<ContextLayer>,
}

impl<S> Context<S>
where
    S: ContextSource,
{
    /// Create a new context from a source
    pub const fn new(source: S) -> Self {
        Self {
            source,
            layers: Vec::new(),
        }
    }

    /// Create a new context with initial layers
    pub fn new_with_layers(source: S, layers: Vec<ContextLayer>) -> Self {
        Self { source, layers }
    }

    /// Get the underlying source
    pub fn source(&self) -> &S {
        &self.source
    }

    /// Get the underlying source mutably
    pub fn source_mut(&mut self) -> &mut S {
        &mut self.source
    }

    /// Execute with a new context
    pub fn exec<R, F>(source: S, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let mut ctx = Self::new(source);
        f(&mut ctx)
    }

    /// Execute with a new context with initial layers
    pub fn exec_with_layers<R, F>(source: S, layers: Vec<ContextLayer>, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let mut ctx = Self::new_with_layers(source, layers);
        f(&mut ctx)
    }

    /// Execute a closure with a new context layer
    pub fn with_layer<R, F>(&mut self, layer: ContextLayer, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>,
    {
        self.layers.push(layer);
        let result = f(self);
        self.layers.pop();
        result
    }

    /// Execute with multiple context layers
    pub fn with_layers<R, F>(&mut self, layers: impl Into<Vec<ContextLayer>>, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>,
    {
        let layers: Vec<ContextLayer> = layers.into();
        let len = layers.len();
        for layer in layers {
            self.layers.push(layer);
        }
        let result = f(self);
        for _ in 0..len {
            self.layers.pop();
        }
        result
    }

    /// Execute with owner context
    pub fn with_owner<R, F>(&mut self, owner_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>,
    {
        self.with_layer(ContextLayer::Owner(owner_id), f)
    }

    /// Execute with target context
    pub fn with_target<R, F>(&mut self, target_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>,
    {
        self.with_layer(ContextLayer::Target(target_id), f)
    }

    /// Execute with caster context
    pub fn with_caster<R, F>(&mut self, caster_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>,
    {
        self.with_layer(ContextLayer::Caster(caster_id), f)
    }

    /// Execute with caster context
    pub fn with_status<R, F>(&mut self, status_id: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>,
    {
        self.with_layer(ContextLayer::Status(status_id), f)
    }

    /// Get current owner ID from context layers
    pub fn owner(&self) -> Option<u64> {
        for layer in self.layers.iter().rev() {
            if let ContextLayer::Owner(id) = layer {
                return Some(*id);
            }
        }
        None
    }

    /// Get current target ID from context layers
    pub fn target(&self) -> Option<u64> {
        for layer in self.layers.iter().rev() {
            if let ContextLayer::Target(id) = layer {
                return Some(*id);
            }
        }
        None
    }

    /// Get current caster ID from context layers
    pub fn caster(&self) -> Option<u64> {
        for layer in self.layers.iter().rev() {
            if let ContextLayer::Caster(id) = layer {
                return Some(*id);
            }
        }
        None
    }

    /// Get current status ID from context layers
    pub fn status(&self) -> Option<u64> {
        for layer in self.layers.iter().rev() {
            if let ContextLayer::Status(id) = layer {
                return Some(*id);
            }
        }
        None
    }

    /// Get variable from owner node with recursive parent lookup
    pub fn owner_var(&self, var: VarName) -> NodeResult<VarValue> {
        if let Some(owner_id) = self.owner() {
            // First try direct access
            self.source.get_var_direct(owner_id, var).or_else(|_| {
                // Then check parents recursively
                let mut parents = VecDeque::from(self.source.get_parents(owner_id)?);
                while let Some(parent) = parents.pop_front() {
                    if let Ok(value) = self.source.get_var_direct(parent, var) {
                        return Ok(value);
                    }
                    if let Ok(more_parents) = self.source.get_parents(parent) {
                        parents.extend(more_parents);
                    }
                }
                Err(NodeError::var_not_found(var))
            })
        } else {
            Err(NodeError::custom("No owner in context"))
        }
    }

    /// Get variable from target node with recursive parent lookup
    pub fn target_var(&self, var: VarName) -> NodeResult<VarValue> {
        if let Some(target_id) = self.target() {
            // First try direct access
            self.source.get_var_direct(target_id, var).or_else(|_| {
                // Then check parents recursively
                let mut parents = VecDeque::from(self.source.get_parents(target_id)?);
                while let Some(parent) = parents.pop_front() {
                    if let Ok(value) = self.source.get_var_direct(parent, var) {
                        return Ok(value);
                    }
                    if let Ok(more_parents) = self.source.get_parents(parent) {
                        parents.extend(more_parents);
                    }
                }
                Err(NodeError::var_not_found(var))
            })
        } else {
            Err(NodeError::custom("No target in context"))
        }
    }

    /// Get variable from caster node with recursive parent lookup
    pub fn caster_var(&self, var: VarName) -> NodeResult<VarValue> {
        if let Some(caster_id) = self.caster() {
            // First try direct access
            self.source.get_var_direct(caster_id, var).or_else(|_| {
                // Then check parents recursively
                let mut parents = VecDeque::from(self.source.get_parents(caster_id)?);
                while let Some(parent) = parents.pop_front() {
                    if let Ok(value) = self.source.get_var_direct(parent, var) {
                        return Ok(value);
                    }
                    if let Ok(more_parents) = self.source.get_parents(parent) {
                        parents.extend(more_parents);
                    }
                }
                Err(NodeError::var_not_found(var))
            })
        } else {
            Err(NodeError::custom("No caster in context"))
        }
    }

    /// Get variable from context layers, checking layers then trying owner/target/caster
    pub fn get_var(&self, var: VarName) -> NodeResult<VarValue> {
        // Check context layers first
        for layer in self.layers.iter().rev() {
            if let ContextLayer::Var(var_name, value) = layer {
                if *var_name == var {
                    return Ok(value.clone());
                }
            }
        }

        // Try owner, then target, then caster
        self.owner_var(var)
            .or_else(|_| self.target_var(var))
            .or_else(|_| self.caster_var(var))
    }

    /// Add a target to the targets list
    pub fn add_target(&mut self, target_id: u64) {
        self.layers.push(ContextLayer::Target(target_id));
    }

    /// Collect all targets
    pub fn collect_targets(&self) -> Vec<u64> {
        self.layers
            .iter()
            .filter_map(|l| {
                if let ContextLayer::Target(target) = l {
                    Some(*target)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Set owner by adding owner layer
    pub fn set_owner(&mut self, owner_id: u64) {
        self.layers.push(ContextLayer::Owner(owner_id));
    }

    /// Set caster by adding caster layer
    pub fn set_caster(&mut self, caster_id: u64) {
        self.layers.push(ContextLayer::Caster(caster_id));
    }

    /// Set a variable in the context
    pub fn set_var_layer(&mut self, name: VarName, value: VarValue) {
        self.layers.push(ContextLayer::Var(name, value));
    }

    /// Get node kind by ID
    pub fn get_kind(&self, id: u64) -> NodeResult<NodeKind> {
        self.source.get_node_kind(id)
    }

    /// Get all child node IDs
    pub fn get_children(&self, from_id: u64) -> NodeResult<Vec<u64>> {
        self.source.get_children(from_id)
    }

    /// Get children of specific kind
    pub fn get_children_of_kind(&self, from_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        self.source.get_children_of_kind(from_id, kind)
    }

    /// Get all parent node IDs
    pub fn get_parents(&self, id: u64) -> NodeResult<Vec<u64>> {
        self.source.get_parents(id)
    }

    /// Get parents of specific kind
    pub fn get_parents_of_kind(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        self.source.get_parents_of_kind(id, kind)
    }

    /// Add a link between two nodes
    pub fn add_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        self.source.add_link(from_id, to_id)
    }

    /// Remove a link between two nodes
    pub fn remove_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()> {
        self.source.remove_link(from_id, to_id)
    }

    /// Check if two nodes are linked
    pub fn is_linked(&self, from_id: u64, to_id: u64) -> NodeResult<bool> {
        self.source.is_linked(from_id, to_id)
    }

    /// Clear all layers
    pub fn clear_layers(&mut self) {
        self.layers.clear();
    }

    pub fn layer_depth(&self) -> usize {
        self.layers.len()
    }

    /// Recursively delete a node and all its owned/component children
    pub fn delete_recursive(&mut self, id: u64) -> NodeResult<()> {
        // Get the node kind first
        let kind = self.get_kind(id)?;

        // Delete all owned and component child nodes recursively
        let owned_children = kind.owned_children();
        let component_children = kind.component_children();

        let children = self.get_children(id)?;
        for child_id in children {
            if let Ok(child_kind) = self.get_kind(child_id) {
                if owned_children.contains(&child_kind) || component_children.contains(&child_kind)
                {
                    self.delete_recursive(child_id)?;
                } else {
                    // Just remove the link for Ref children
                    self.remove_link(id, child_id)?;
                }
            }
        }

        // Remove all links to this node from parents
        let parents = self.get_parents(id)?;
        for parent_id in parents {
            self.remove_link(parent_id, id)?;
        }

        // Finally delete the node itself
        self.source.delete_node(id)?;

        Ok(())
    }

    /// Find first parent of specified kind
    pub fn first_parent(&self, id: u64, kind: NodeKind) -> NodeResult<u64> {
        let parents = self.get_parents(id)?;
        for parent_id in parents {
            if self.get_kind(parent_id)? == kind {
                return Ok(parent_id);
            }
        }
        Err(NodeError::custom(format!(
            "No parent of kind {:?} found for node {}",
            kind, id
        )))
    }

    /// Find first parent of specified kind recursively (BFS)
    pub fn first_parent_recursive(&self, id: u64, kind: NodeKind) -> NodeResult<u64> {
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();

        queue.push_back(id);
        visited.insert(id);

        while let Some(current_id) = queue.pop_front() {
            let parents = self.get_parents(current_id)?;
            for parent_id in parents {
                if !visited.insert(parent_id) {
                    continue;
                }

                if self.get_kind(parent_id)? == kind {
                    return Ok(parent_id);
                }

                queue.push_back(parent_id);
            }
        }

        Err(NodeError::custom(format!(
            "No parent of kind {:?} found recursively for node {}",
            kind, id
        )))
    }

    /// Find first child of specified kind
    pub fn first_child(&self, id: u64, kind: NodeKind) -> NodeResult<u64> {
        let children = self.get_children(id)?;
        for child_id in children {
            if self.get_kind(child_id)? == kind {
                return Ok(child_id);
            }
        }
        Err(NodeError::custom(format!(
            "No child of kind {:?} found for node {}",
            kind, id
        )))
    }

    /// Find first child of specified kind recursively (BFS)
    pub fn first_child_recursive(&self, id: u64, kind: NodeKind) -> NodeResult<u64> {
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();

        queue.push_back(id);
        visited.insert(id);

        while let Some(current_id) = queue.pop_front() {
            let children = self.get_children(current_id)?;
            for child_id in children {
                if !visited.insert(child_id) {
                    continue;
                }

                if self.get_kind(child_id)? == kind {
                    return Ok(child_id);
                }

                queue.push_back(child_id);
            }
        }

        Err(NodeError::custom(format!(
            "No child of kind {:?} found recursively for node {}",
            kind, id
        )))
    }

    /// Get all parents recursively
    pub fn parents_recursive(&self, id: u64) -> NodeResult<Vec<u64>> {
        let mut result = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();

        queue.push_back(id);
        visited.insert(id);

        while let Some(current_id) = queue.pop_front() {
            let parents = self.get_parents(current_id)?;
            for parent_id in parents {
                if visited.insert(parent_id) {
                    result.push(parent_id);
                    queue.push_back(parent_id);
                }
            }
        }

        Ok(result)
    }

    /// Get all children recursively
    pub fn children_recursive(&self, id: u64) -> NodeResult<Vec<u64>> {
        let mut result = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();

        queue.push_back(id);
        visited.insert(id);

        while let Some(current_id) = queue.pop_front() {
            let children = self.get_children(current_id)?;
            for child_id in children {
                if visited.insert(child_id) {
                    result.push(child_id);
                    queue.push_back(child_id);
                }
            }
        }

        Ok(result)
    }

    /// Collect children of specific kind
    pub fn collect_kind_children(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        let children = self.get_children(id)?;
        let mut result = Vec::new();

        for child_id in children {
            if self.get_kind(child_id)? == kind {
                result.push(child_id);
            }
        }

        Ok(result)
    }

    /// Collect children of specific kind recursively
    pub fn collect_kind_children_recursive(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        let mut result = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();

        queue.push_back(id);
        visited.insert(id);

        while let Some(current_id) = queue.pop_front() {
            let children = self.get_children(current_id)?;
            for child_id in children {
                if !visited.insert(child_id) {
                    continue;
                }

                if self.get_kind(child_id)? == kind {
                    result.push(child_id);
                }

                queue.push_back(child_id);
            }
        }

        Ok(result)
    }

    /// Collect parents of specific kind
    pub fn collect_kind_parents(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        let parents = self.get_parents(id)?;
        let mut result = Vec::new();

        for parent_id in parents {
            if self.get_kind(parent_id)? == kind {
                result.push(parent_id);
            }
        }

        Ok(result)
    }

    /// Collect parents of specific kind recursively
    pub fn collect_kind_parents_recursive(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        let mut result = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();

        queue.push_back(id);
        visited.insert(id);

        while let Some(current_id) = queue.pop_front() {
            let parents = self.get_parents(current_id)?;
            for parent_id in parents {
                if !visited.insert(parent_id) {
                    continue;
                }

                if self.get_kind(parent_id)? == kind {
                    result.push(parent_id);
                }

                queue.push_back(parent_id);
            }
        }

        Ok(result)
    }

    pub fn get_vars_layers(&self) -> HashMap<VarName, VarValue> {
        let mut result = HashMap::new();

        for l in &self.layers {
            match l {
                ContextLayer::Var(var, var_value) => {
                    result.insert(*var, var_value.clone());
                }
                _ => {}
            }
        }

        result
    }

    pub fn layers(&self) -> &Vec<ContextLayer> {
        &self.layers
    }

    pub fn debug_layers(&self) {
        dbg!(&self.layers);
    }
}
