use std::collections::{HashMap, HashSet};

use itertools::Itertools;

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
    fn rating(&self) -> i32;
    fn set_rating(&mut self, rating: i32);
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

    fn pack(&self) -> PackedNodes {
        let mut packed = PackedNodes::default();
        let mut visited = HashSet::new();
        self.pack_recursive(&mut packed, &mut visited);
        packed.root = self.id();
        packed.add_node(
            Self::kind_s().to_string(),
            self.get_data(),
            self.id(),
            self.owner(),
        );
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
        packed.add_node(self.kind().to_string(), self.get_data(), id, self.owner());

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

    fn collect_owned_ids(&self) -> Vec<u64>;
    fn collect_owned_links(&self) -> Vec<(u64, u64)>;
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
    fn insert_node(
        &mut self,
        id: u64,
        owner: u64,
        data: String,
        node_kind: NodeKind,
    ) -> NodeResult<()>;

    fn commit(&mut self, node: impl Node) -> NodeResult<()> {
        self.commit_vec(vec![node])
    }

    fn commit_vec(&mut self, nodes: Vec<impl Node>) -> NodeResult<()> {
        if nodes.is_empty() {
            return Ok(());
        }
        // 1. Get flat structure of nodes and links
        let mut all_nodes = HashMap::new();
        let mut all_links = HashSet::new();
        let mut root_ids = Vec::new();

        for node in nodes {
            root_ids.push(node.id());
            let packed = node.pack();

            // Collect all nodes
            for (id, node_data) in packed.nodes {
                all_nodes.insert(id, node_data);
            }

            // Collect all links
            for link in packed.links {
                all_links.insert(link);
            }
        }

        // 2. For any parent+child_kind in links, remove existing links from source that are not listed
        let mut links_by_parent_kind: HashMap<(u64, String), HashSet<u64>> = HashMap::new();
        for link in &all_links {
            links_by_parent_kind
                .entry((link.parent, link.child_kind.clone()))
                .or_default()
                .insert(link.child);
        }

        for ((parent_id, child_kind), new_children) in &links_by_parent_kind {
            let child_kind_enum = NodeKind::from_str(child_kind)
                .map_err(|_| NodeError::custom(format!("Invalid node kind: {}", child_kind)))?;
            let existing_children = self
                .get_children_of_kind(*parent_id, child_kind_enum)
                .unwrap_or_default();

            for existing_child in existing_children {
                if !new_children.contains(&existing_child) {
                    self.remove_link(*parent_id, existing_child)?;
                }
            }
        }

        // 3. Check for orphaned nodes and collect them
        let mut orphans_to_delete = Vec::new();
        let mut checked_nodes = HashSet::new();

        // Get all affected children from removed links
        for ((parent_id, child_kind), new_children) in &links_by_parent_kind {
            let child_kind_enum = NodeKind::from_str(child_kind)
                .map_err(|_| NodeError::custom(format!("Invalid node kind: {}", child_kind)))?;
            let existing_children = self
                .get_children_of_kind(*parent_id, child_kind_enum)
                .unwrap_or_default();

            for existing_child in existing_children {
                if !new_children.contains(&existing_child)
                    && !checked_nodes.contains(&existing_child)
                {
                    checked_nodes.insert(existing_child);

                    // Check if this child has become an orphan
                    if !root_ids.contains(&existing_child) {
                        let child_kind = self.get_node_kind(existing_child)?;
                        let owning_parents = child_kind.owning_parents();

                        let mut has_owner = false;
                        for parent_kind in owning_parents {
                            let parents = self.get_parents_of_kind(existing_child, parent_kind)?;

                            // Also check if any parent exists in our new links
                            for parent in &parents {
                                if all_links
                                    .iter()
                                    .any(|l| l.parent == *parent && l.child == existing_child)
                                {
                                    has_owner = true;
                                    break;
                                }
                            }

                            if has_owner || !parents.is_empty() {
                                has_owner = true;
                                break;
                            }
                        }

                        if !has_owner {
                            orphans_to_delete.push(existing_child);
                        }
                    }
                }
            }
        }

        // Delete orphaned nodes
        for orphan_id in orphans_to_delete {
            self.delete_node(orphan_id)?;
        }

        // 4. Insert/update all nodes to source
        for (id, node_data) in all_nodes {
            let kind = NodeKind::from_str(&node_data.kind)
                .map_err(|_| NodeError::custom(format!("Invalid node kind: {}", node_data.kind)))?;
            self.insert_node(id, node_data.owner, node_data.data, kind)?;
        }

        // 5. Insert all links to source
        for link in all_links {
            self.add_link(link.parent, link.child)?;
        }

        Ok(())
    }
}

/// Context layer for scoped operations
#[derive(Debug, Clone, PartialEq)]
pub enum ContextLayer {
    Owner(u64),
    Target(u64),
    Targets(Vec<u64>),
    Caster(u64),
    Status(u64),
    Attacker(u64),
    Var(VarName, VarValue),
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
        self.with_layers([layer], f)
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

    pub fn with_status<F, R>(&mut self, status: u64, f: F) -> NodeResult<R>
    where
        F: FnOnce(&mut Self) -> NodeResult<R>,
    {
        self.with_layers(
            [ContextLayer::Owner(status), ContextLayer::Status(status)],
            f,
        )
    }

    pub fn owner(&self) -> NodeResult<u64> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| match l {
                ContextLayer::Owner(id) => Some(*id),
                _ => None,
            })
            .ok_or_else(|| NodeError::custom("No owner in context"))
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

    pub fn attacker(&self) -> Option<u64> {
        self.layers.iter().rev().find_map(|l| match l {
            ContextLayer::Attacker(id) => Some(*id),
            _ => None,
        })
    }

    pub fn status(&self) -> Option<u64> {
        self.layers.iter().rev().find_map(|l| match l {
            ContextLayer::Status(id) => Some(*id),
            _ => None,
        })
    }

    pub fn owner_var(&self, var: VarName) -> NodeResult<VarValue> {
        self.get_var_inherited(self.owner()?, var).track()
    }

    pub fn target_var(&self, var: VarName) -> NodeResult<VarValue> {
        if let Some(target) = self.target() {
            self.get_var_inherited(target, var).track()
        } else {
            Err(NodeError::custom("No target in context"))
        }
    }

    pub fn caster_var(&self, var: VarName) -> NodeResult<VarValue> {
        if let Some(caster) = self.caster() {
            self.get_var_inherited(caster, var).track()
        } else {
            Err(NodeError::custom("No caster in context"))
        }
    }

    pub fn status_var(&self, var: VarName) -> NodeResult<VarValue> {
        if let Some(status) = self.status() {
            self.get_var_inherited(status, var).track()
        } else {
            Err(NodeError::custom("No status in context"))
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
        if let Ok(owner) = self.owner() {
            self.get_var_inherited(owner, var).track()
        } else {
            Err(NodeError::custom("Cannot get var without owner"))
        }
    }

    pub fn get_var_inherited(&self, id: u64, var: VarName) -> NodeResult<VarValue> {
        if let Ok(value) = self.source().get_var(id, var) {
            return Ok(value);
        }
        for child_kind in self.get_kind(id)?.component_children_recursive() {
            for child_id in self.collect_kind_children_recursive(id, child_kind)? {
                if let Ok(value) = self.source().get_var(child_id, var) {
                    return Ok(value);
                }
            }
        }
        for parent in self.get_parents(id)? {
            if let Ok(value) = self.get_var_inherited(parent, var) {
                return Ok(value);
            }
        }
        Err(NodeError::var_not_found(var))
    }

    pub fn add_targets(&mut self, targets: Vec<u64>) {
        let mut current_targets = self.get_targets();
        current_targets.extend(targets);
        self.set_targets(current_targets);
    }

    pub fn set_targets(&mut self, targets: Vec<u64>) {
        self.layers.push(ContextLayer::Targets(targets));
    }

    pub fn get_targets(&self) -> Vec<u64> {
        for l in self.layers().iter().rev() {
            match l {
                ContextLayer::Target(id) => {
                    return vec![*id];
                }
                ContextLayer::Targets(ids) => {
                    return ids.clone();
                }
                _ => {}
            }
        }
        default()
    }

    pub fn set_owner(&mut self, owner: u64) {
        self.layers.push(ContextLayer::Owner(owner));
    }

    pub fn set_caster(&mut self, caster: u64) {
        self.layers.push(ContextLayer::Caster(caster));
    }

    pub fn set_attacker(&mut self, attacker: u64) {
        self.layers.push(ContextLayer::Attacker(attacker));
    }

    pub fn set_status(&mut self, status: u64) {
        self.layers.push(ContextLayer::Owner(status));
        self.layers.push(ContextLayer::Status(status));
    }

    pub fn set_var_layer(&mut self, var: VarName, value: VarValue) {
        self.layers.push(ContextLayer::Var(var, value));
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
        self.source.delete_node(id).track()
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
            .ok_or_else(|| NodeError::custom(format!("No child of kind {:?} for {}", kind, id)))
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
            .rev()
            .unique_by(|(var, _)| *var)
            .collect()
    }

    pub fn layers(&self) -> &Vec<ContextLayer> {
        &self.layers
    }

    pub fn debug_layers(&self) {
        dbg!(&self.layers);
    }
}
