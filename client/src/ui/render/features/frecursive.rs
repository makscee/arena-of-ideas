use super::*;

/// Feature for types that can be rendered recursively in a tree view with selection
pub trait FRecursiveNodeEdit: FEdit + FTitle + ClientNode + Clone + Debug {
    /// Compatibility wrapper for old render_recursive_edit API
    fn render_recursive_edit(&mut self, ui: &mut Ui, ctx: &ClientContext) -> bool {
        // Simple edit mode without tree structure
        self.edit(ui, ctx).changed()
    }

    /// Render the node recursively as a tree, returning true if any changes were made
    fn render_recursive_tree<H, B>(
        &mut self,
        ui: &mut Ui,
        ctx: &ClientContext,
        render_header: &H,
        render_body: &B,
    ) -> bool
    where
        H: Fn(u64, NodeKind, &ClientContext) -> String,
        B: Fn(u64, NodeKind, &ClientContext, &mut Ui),
    {
        let node_id = self.id();
        let node_kind = self.kind();
        render_body(node_id, node_kind, ctx, ui);
        self.render_linked_fields_tree(ui, ctx, render_header, render_body)
    }

    /// Render linked fields recursively with the given closures
    fn render_linked_fields_tree<H, B>(
        &mut self,
        ui: &mut Ui,
        ctx: &ClientContext,
        render_header: &H,
        render_body: &B,
    ) -> bool
    where
        H: Fn(u64, NodeKind, &ClientContext) -> String,
        B: Fn(u64, NodeKind, &ClientContext, &mut Ui);
}
