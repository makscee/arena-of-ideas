use super::*;
use crate::node_error::NodeResult;

// Common traits that both client and server will implement
pub trait Node: Send + Sync + Default + StringData {
    fn id(&self) -> u64;
    fn set_id(&mut self, id: u64);
    fn owner(&self) -> u64;
    fn set_owner(&mut self, owner: u64);
    fn kind(&self) -> NodeKind;
    fn reassign_ids(&mut self, next_id: &mut u64);
    fn kind_s() -> NodeKind
    where
        Self: Sized;

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
            .ok_or_else(|| NodeError::Custom("Root node not found in packed data".into()))?;

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
pub trait ContextSource {
    /// Get node kind by ID
    fn get_node_kind(&self, id: u64) -> NodeResult<NodeKind>;

    /// Get all linked child IDs
    fn get_children(&self, from_id: u64) -> NodeResult<Vec<u64>>;

    /// Get children of specific kind
    fn get_children_of_kind(&self, from_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>>;

    /// Get all parent IDs
    fn get_parents(&self, id: u64) -> NodeResult<Vec<u64>>;

    /// Get parents of specific kind
    fn get_parents_of_kind(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>>;

    /// Add a link between two nodes
    fn add_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()>;

    /// Remove a link between two nodes
    fn remove_link(&mut self, from_id: u64, to_id: u64) -> NodeResult<()>;

    /// Check if two nodes are linked
    fn is_linked(&self, from_id: u64, to_id: u64) -> NodeResult<bool>;
}

/// Context layer for scoping operations
#[derive(Debug, Clone)]
pub enum ContextLayer {
    Owner(u64),
    Target(u64),
    Caster(u64),
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
    pub fn with_layers(source: S, layers: Vec<ContextLayer>) -> Self {
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
        let mut ctx = Self::with_layers(source, layers);
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
    pub fn with_layers_temp<R, F>(&mut self, layers: Vec<ContextLayer>, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>,
    {
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

    /// Get variable value from context layers
    pub fn get_var(&self, var: VarName) -> NodeResult<VarValue> {
        for layer in self.layers.iter().rev() {
            if let ContextLayer::Var(var_name, value) = layer {
                if *var_name == var {
                    return Ok(value.clone());
                }
            }
        }
        Err(NodeError::VarNotFound(var))
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

    pub fn get_color(&self, var: VarName) -> NodeResult<Color32> {
        self.get_var(var)?.get_color()
    }

    pub fn get_i32(&self, var: VarName) -> NodeResult<i32> {
        self.get_var(var)?.get_i32()
    }

    pub fn get_f32(&self, var: VarName) -> NodeResult<f32> {
        self.get_var(var)?.get_f32()
    }

    pub fn get_string(&self, var: VarName) -> NodeResult<String> {
        self.get_var(var)?.get_string()
    }

    pub fn get_bool(&self, var: VarName) -> NodeResult<bool> {
        self.get_var(var)?.get_bool()
    }

    pub fn get_vec2(&self, var: VarName) -> NodeResult<Vec2> {
        self.get_var(var)?.get_vec2()
    }

    /// Set a variable in the context
    pub fn set_var(&mut self, name: VarName, value: VarValue) {
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

    /// Get the current layer stack depth
    pub fn layer_depth(&self) -> usize {
        self.layers.len()
    }

    /// Find first parent of specified kind
    pub fn first_parent(&self, id: u64, kind: NodeKind) -> NodeResult<u64> {
        let parents = self.get_parents(id)?;
        for parent_id in parents {
            if self.get_kind(parent_id)? == kind {
                return Ok(parent_id);
            }
        }
        Err(NodeError::Custom(format!(
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

        Err(NodeError::Custom(format!(
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
        Err(NodeError::Custom(format!(
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

        Err(NodeError::Custom(format!(
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
}
