use super::*;

/// Extension trait for Ui to manage inspected node state
pub trait InspectedNodeExt {
    /// Set the currently inspected node ID
    fn set_inspected_node(&mut self, node_id: u64);

    /// Get the currently inspected node ID
    fn inspected_node(&self) -> Option<u64>;

    /// Clear the inspected node
    fn clear_inspected_node(&mut self);

    /// Check if a specific node is currently inspected
    fn is_node_inspected(&self, node_id: u64) -> bool;

    /// Set the inspected node for a specific parent context
    fn set_inspected_node_for_parent(&mut self, parent_id: u64, node_id: u64);

    /// Get the inspected node for a specific parent context
    fn inspected_node_for_parent(&self, parent_id: u64) -> Option<u64>;

    /// Clear the inspected node for a specific parent context
    fn clear_inspected_node_for_parent(&mut self, parent_id: u64);

    /// Set the current edit context (parent node ID)
    fn set_edit_context(&mut self, parent_id: u64);

    /// Get the current edit context (parent node ID)
    fn edit_context(&self) -> Option<u64>;

    /// Clear the current edit context
    fn clear_edit_context(&mut self);

    /// Helper function to set selection for a parent-child pair
    fn set_selection_for_pair(&mut self, parent_id: u64, child_id: u64);
}

impl InspectedNodeExt for Ui {
    fn set_inspected_node(&mut self, node_id: u64) {
        if let Some(parent_id) = self.edit_context() {
            self.set_inspected_node_for_parent(parent_id, node_id);
        } else {
            self.ctx().data_mut(|data| {
                data.insert_temp(Id::new("inspected_node"), node_id);
            });
        }
    }

    fn inspected_node(&self) -> Option<u64> {
        if let Some(parent_id) = self.edit_context() {
            self.inspected_node_for_parent(parent_id)
        } else {
            self.ctx()
                .data(|data| data.get_temp::<u64>(Id::new("inspected_node")))
        }
    }

    fn clear_inspected_node(&mut self) {
        if let Some(parent_id) = self.edit_context() {
            self.clear_inspected_node_for_parent(parent_id);
        } else {
            self.ctx().data_mut(|data| {
                data.remove::<u64>(Id::new("inspected_node"));
            });
        }
    }

    fn is_node_inspected(&self, node_id: u64) -> bool {
        self.inspected_node() == Some(node_id)
    }

    fn set_inspected_node_for_parent(&mut self, parent_id: u64, node_id: u64) {
        self.ctx().data_mut(|data| {
            data.insert_temp(Id::new(&format!("inspected_node_{}", parent_id)), node_id);
        });
    }

    fn inspected_node_for_parent(&self, parent_id: u64) -> Option<u64> {
        self.ctx()
            .data(|data| data.get_temp::<u64>(Id::new(&format!("inspected_node_{}", parent_id))))
    }

    fn clear_inspected_node_for_parent(&mut self, parent_id: u64) {
        self.ctx().data_mut(|data| {
            data.remove::<u64>(Id::new(&format!("inspected_node_{}", parent_id)));
        });
    }

    fn set_edit_context(&mut self, parent_id: u64) {
        self.ctx().data_mut(|data| {
            data.insert_temp(Id::new("edit_context"), parent_id);
        });
    }

    fn edit_context(&self) -> Option<u64> {
        self.ctx()
            .data(|data| data.get_temp::<u64>(Id::new("edit_context")))
    }

    fn clear_edit_context(&mut self) {
        self.ctx().data_mut(|data| {
            data.remove::<u64>(Id::new("edit_context"));
        });
    }

    fn set_selection_for_pair(&mut self, parent_id: u64, child_id: u64) {
        self.set_edit_context(parent_id);
        self.set_inspected_node_for_parent(parent_id, child_id);
    }
}

/// Structure to hold breadcrumb information
#[derive(Debug, Clone)]
pub struct NodeBreadcrumb {
    pub id: u64,
    pub kind: NodeKind,
    pub field_name: Option<String>,
}
