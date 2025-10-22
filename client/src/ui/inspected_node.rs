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
}

impl InspectedNodeExt for Ui {
    fn set_inspected_node(&mut self, node_id: u64) {
        self.ctx().data_mut(|data| {
            data.insert_temp(Id::new("inspected_node"), node_id);
        });
    }

    fn inspected_node(&self) -> Option<u64> {
        self.ctx()
            .data(|data| data.get_temp::<u64>(Id::new("inspected_node")))
    }

    fn clear_inspected_node(&mut self) {
        self.ctx().data_mut(|data| {
            data.remove::<u64>(Id::new("inspected_node"));
        });
    }

    fn is_node_inspected(&self, node_id: u64) -> bool {
        self.inspected_node() == Some(node_id)
    }
}

/// Structure to hold breadcrumb information
#[derive(Debug, Clone)]
pub struct NodeBreadcrumb {
    pub id: u64,
    pub kind: NodeKind,
    pub field_name: Option<String>,
}
