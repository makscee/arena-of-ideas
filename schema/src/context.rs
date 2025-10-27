use crate::*;

/// Node trait for all node types
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
    fn reassign_ids(&mut self, next_id: &mut u64, id_map: &mut std::collections::HashMap<u64, u64>);
    fn kind_s() -> NodeKind
    where
        Self: Sized;

    fn var_names() -> Vec<VarName>
    where
        Self: Sized;
    fn set_var(&mut self, var: VarName, value: VarValue) -> NodeResult<()>;
    fn get_var(&self, var: VarName) -> NodeResult<VarValue>;
    fn get_vars(&self) -> Vec<(VarName, VarValue)>;

    fn set_dirty(&mut self, value: bool);
    fn is_dirty(&self) -> bool;

    fn pack(&self) -> PackedNodes {
        let mut packed = PackedNodes::default();
        packed.root = self.id();
        packed.add_node(Self::kind_s().to_string(), self.get_data(), self.id());
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

/// Trait for converting between NodeKind and concrete types
pub trait NodeKindConvert {
    fn to_node_kind(&self) -> NodeKind;
    fn from_node_kind(_kind: NodeKind) -> Option<Self>
    where
        Self: Sized;
}

/// Core context source trait for data access
pub trait ContextSource {
    fn get_var(&self, node_id: u64, var: VarName) -> NodeResult<VarValue>;
    fn set_var(&mut self, node_id: u64, var: VarName, value: VarValue) -> NodeResult<()>;
    fn var_updated(&mut self, node_id: u64, var: VarName, value: VarValue);

    fn get_node_kind(&self, node_id: u64) -> NodeResult<NodeKind>;
    fn get_children(&self, node_id: u64) -> NodeResult<Vec<u64>>;
    fn get_children_of_kind(&self, node_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>>;
    fn get_parents(&self, node_id: u64) -> NodeResult<Vec<u64>>;
    fn get_parents_of_kind(&self, node_id: u64, kind: NodeKind) -> NodeResult<Vec<u64>>;

    fn add_link(&mut self, parent_id: u64, child_id: u64) -> NodeResult<()>;
    fn remove_link(&mut self, parent_id: u64, child_id: u64) -> NodeResult<()>;
    fn clear_links(&mut self, node_id: u64) -> NodeResult<()>;
    fn is_linked(&self, parent_id: u64, child_id: u64) -> NodeResult<bool>;
    fn delete_node(&mut self, node_id: u64) -> NodeResult<()>;
}

/// Context layer for scoped operations
#[derive(Debug, Clone, PartialEq)]
pub enum ContextLayer {
    Owner(u64),
    Target(u64),
    Caster(u64),
    Status(u64),
    Var(VarName, VarValue),
    Time(f32),
}

/// Main context struct
#[derive(Debug)]
pub struct Context<S> {
    source: S,
    layers: Vec<ContextLayer>,
}

impl<S: ContextSource> Context<S> {
    pub const fn new(source: S) -> Self {
        Self {
            source,
            layers: Vec::new(),
        }
    }

    pub fn new_with_layers(source: S, layers: Vec<ContextLayer>) -> Self {
        Self { source, layers }
    }

    pub fn source(&self) -> &S {
        &self.source
    }

    pub fn source_mut(&mut self) -> &mut S {
        &mut self.source
    }

    pub fn into_inner(self) -> S {
        self.source
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
        let original_len = self.layers.len();
        self.layers.extend(layers.into());
        let result = f(self);
        self.layers.truncate(original_len);
        result
    }

    pub fn with_owner<F, R>(&mut self, owner: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>,
    {
        self.with_layer(ContextLayer::Owner(owner), f)
    }

    pub fn with_target<F, R>(&mut self, target: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>,
    {
        self.with_layer(ContextLayer::Target(target), f)
    }

    pub fn with_caster<F, R>(&mut self, caster: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>,
    {
        self.with_layer(ContextLayer::Caster(caster), f)
    }

    pub fn with_time<F, R>(&mut self, time: f32, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>,
    {
        self.with_layer(ContextLayer::Time(time), f)
    }

    pub fn with_status<F, R>(&mut self, status: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>,
    {
        self.with_layer(ContextLayer::Status(status), f)
    }

    pub fn owner(&self) -> Option<u64> {
        self.layers.iter().rev().find_map(|l| match l {
            ContextLayer::Owner(id) => Some(*id),
            _ => None,
        })
    }

    pub fn target(&self) -> Option<u64> {
        self.layers.iter().rev().find_map(|l| match l {
            ContextLayer::Target(id) => Some(*id),
            _ => None,
        })
    }

    pub fn caster(&self) -> Option<u64> {
        self.layers.iter().rev().find_map(|l| match l {
            ContextLayer::Caster(id) => Some(*id),
            _ => None,
        })
    }

    pub fn status(&self) -> Option<u64> {
        self.layers.iter().rev().find_map(|l| match l {
            ContextLayer::Status(id) => Some(*id),
            _ => None,
        })
    }

    pub fn time(&self) -> Option<f32> {
        self.layers.iter().rev().find_map(|l| match l {
            ContextLayer::Time(t) => Some(*t),
            _ => None,
        })
    }

    pub fn owner_var(&self, var: VarName) -> NodeResult<VarValue> {
        if let Some(owner) = self.owner() {
            self.source.get_var(owner, var)
        } else {
            Err(NodeError::custom("No owner in context"))
        }
    }

    pub fn target_var(&self, var: VarName) -> NodeResult<VarValue> {
        if let Some(target) = self.target() {
            self.source.get_var(target, var)
        } else {
            Err(NodeError::custom("No target in context"))
        }
    }

    pub fn caster_var(&self, var: VarName) -> NodeResult<VarValue> {
        if let Some(caster) = self.caster() {
            self.source.get_var(caster, var)
        } else {
            Err(NodeError::custom("No caster in context"))
        }
    }

    pub fn get_var(&self, var: VarName) -> NodeResult<VarValue> {
        // Check context layers first
        for layer in self.layers.iter().rev() {
            if let ContextLayer::Var(v, value) = layer {
                if *v == var {
                    return Ok(value.clone());
                }
            }
        }

        // Try to get from owner
        if let Some(owner) = self.owner() {
            self.source.get_var(owner, var)
        } else {
            Err(NodeError::custom("Cannot get var without owner"))
        }
    }

    pub fn add_target(&mut self, target: u64) {
        self.layers.push(ContextLayer::Target(target));
    }

    pub fn collect_targets(&self) -> Vec<u64> {
        self.layers
            .iter()
            .filter_map(|l| match l {
                ContextLayer::Target(id) => Some(*id),
                _ => None,
            })
            .collect()
    }

    pub fn set_owner(&mut self, owner: u64) {
        self.layers.push(ContextLayer::Owner(owner));
    }

    pub fn set_caster(&mut self, caster: u64) {
        self.layers.push(ContextLayer::Caster(caster));
    }

    pub fn set_var_layer(&mut self, var: VarName, value: VarValue) {
        self.layers.push(ContextLayer::Var(var, value));
    }

    pub fn set_time_layer(&mut self, time: f32) {
        self.layers.push(ContextLayer::Time(time));
    }

    pub fn get_kind(&self, id: u64) -> NodeResult<NodeKind> {
        self.source.get_node_kind(id)
    }

    pub fn get_children(&self, id: u64) -> NodeResult<Vec<u64>> {
        self.source.get_children(id)
    }

    pub fn get_children_of_kind(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        self.source.get_children_of_kind(id, kind)
    }

    pub fn get_parents(&self, id: u64) -> NodeResult<Vec<u64>> {
        self.source.get_parents(id)
    }

    pub fn get_parents_of_kind(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        self.source.get_parents_of_kind(id, kind)
    }

    pub fn add_link(&mut self, parent: u64, child: u64) -> NodeResult<()> {
        self.source.add_link(parent, child)
    }

    pub fn remove_link(&mut self, parent: u64, child: u64) -> NodeResult<()> {
        self.source.remove_link(parent, child)
    }

    pub fn is_linked(&self, parent: u64, child: u64) -> NodeResult<bool> {
        self.source.is_linked(parent, child)
    }

    pub fn clear_layers(&mut self) {
        self.layers.clear();
    }

    pub fn layer_depth(&self) -> usize {
        self.layers.len()
    }

    pub fn delete_recursive(&mut self, id: u64) -> NodeResult<()> {
        let children = self.get_children(id).unwrap_or_default();
        for child_id in children {
            self.delete_recursive(child_id)?;
        }
        self.source.delete_node(id)
    }

    pub fn first_parent(&self, id: u64, kind: NodeKind) -> NodeResult<u64> {
        self.get_parents_of_kind(id, kind)?
            .into_iter()
            .next()
            .ok_or_else(|| NodeError::custom(format!("No parent of kind {:?}", kind)))
    }

    pub fn first_parent_recursive(&self, id: u64, kind: NodeKind) -> NodeResult<u64> {
        if let Ok(parent) = self.first_parent(id, kind) {
            return Ok(parent);
        }

        let parents = self.get_parents(id)?;
        for parent_id in parents {
            if let Ok(found) = self.first_parent_recursive(parent_id, kind) {
                return Ok(found);
            }
        }

        Err(NodeError::custom(format!(
            "No parent of kind {:?} found recursively",
            kind
        )))
    }

    pub fn first_child(&self, id: u64, kind: NodeKind) -> NodeResult<u64> {
        self.get_children_of_kind(id, kind)?
            .into_iter()
            .next()
            .ok_or_else(|| NodeError::custom(format!("No child of kind {:?}", kind)))
    }

    pub fn first_child_recursive(&self, id: u64, kind: NodeKind) -> NodeResult<u64> {
        if let Ok(child) = self.first_child(id, kind) {
            return Ok(child);
        }

        let children = self.get_children(id)?;
        for child_id in children {
            if let Ok(found) = self.first_child_recursive(child_id, kind) {
                return Ok(found);
            }
        }

        Err(NodeError::custom(format!(
            "No child of kind {:?} found recursively",
            kind
        )))
    }

    pub fn parents_recursive(&self, id: u64) -> NodeResult<Vec<u64>> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.collect_parents_recursive(id, &mut result, &mut visited)?;
        Ok(result)
    }

    fn collect_parents_recursive(
        &self,
        id: u64,
        result: &mut Vec<u64>,
        visited: &mut std::collections::HashSet<u64>,
    ) -> NodeResult<()> {
        if !visited.insert(id) {
            return Ok(());
        }
        for parent_id in self.get_parents(id)? {
            result.push(parent_id);
            self.collect_parents_recursive(parent_id, result, visited)?;
        }
        Ok(())
    }

    pub fn children_recursive(&self, id: u64) -> NodeResult<Vec<u64>> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.collect_children_recursive(id, &mut result, &mut visited)?;
        Ok(result)
    }

    fn collect_children_recursive(
        &self,
        id: u64,
        result: &mut Vec<u64>,
        visited: &mut std::collections::HashSet<u64>,
    ) -> NodeResult<()> {
        if !visited.insert(id) {
            return Ok(());
        }
        for child_id in self.get_children(id)? {
            result.push(child_id);
            self.collect_children_recursive(child_id, result, visited)?;
        }
        Ok(())
    }

    pub fn collect_kind_children(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        self.get_children_of_kind(id, kind)
    }

    pub fn collect_kind_children_recursive(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.collect_kind_children_inner(id, kind, &mut result, &mut visited)?;
        Ok(result)
    }

    fn collect_kind_children_inner(
        &self,
        id: u64,
        kind: NodeKind,
        result: &mut Vec<u64>,
        visited: &mut std::collections::HashSet<u64>,
    ) -> NodeResult<()> {
        if !visited.insert(id) {
            return Ok(());
        }
        result.extend(self.get_children_of_kind(id, kind)?);
        for child_id in self.get_children(id)? {
            self.collect_kind_children_inner(child_id, kind, result, visited)?;
        }
        Ok(())
    }

    pub fn collect_kind_parents(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        self.get_parents_of_kind(id, kind)
    }

    pub fn collect_kind_parents_recursive(&self, id: u64, kind: NodeKind) -> NodeResult<Vec<u64>> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.collect_kind_parents_inner(id, kind, &mut result, &mut visited)?;
        Ok(result)
    }

    fn collect_kind_parents_inner(
        &self,
        id: u64,
        kind: NodeKind,
        result: &mut Vec<u64>,
        visited: &mut std::collections::HashSet<u64>,
    ) -> NodeResult<()> {
        if !visited.insert(id) {
            return Ok(());
        }
        result.extend(self.get_parents_of_kind(id, kind)?.into_iter());
        for parent_id in self.get_parents(id)? {
            self.collect_kind_parents_inner(parent_id, kind, result, visited)?;
        }
        Ok(())
    }

    pub fn get_vars_layers(&self) -> Vec<(VarName, VarValue)> {
        self.layers
            .iter()
            .filter_map(|l| match l {
                ContextLayer::Var(var, value) => Some((*var, value.clone())),
                _ => None,
            })
            .collect()
    }

    pub fn layers(&self) -> &Vec<ContextLayer> {
        &self.layers
    }

    pub fn debug_layers(&self) {
        dbg!(&self.layers);
    }
}
