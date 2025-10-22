use super::*;

/// Feature for types that can be rendered recursively with breadcrumb navigation
pub trait FRecursiveRender: FEdit + Node {
    /// Render the node recursively, handling breadcrumbs and inspection state
    fn render_recursive(&mut self, ui: &mut Ui) -> bool {
        let node_id = self.id();
        ui.set_edit_context(node_id);
        let result = render_node_field_recursive_with_path(ui, "root", self, &mut vec![]);
        ui.clear_edit_context();
        result
    }

    /// Render linked fields for the current node
    fn render_linked_fields(
        &mut self,
        ui: &mut Ui,
        breadcrumb_path: &mut Vec<NodeBreadcrumb>,
    ) -> bool;

    /// Search recursively through linked fields for inspected node
    fn render_recursive_search(
        &mut self,
        ui: &mut Ui,
        breadcrumb_path: &mut Vec<NodeBreadcrumb>,
    ) -> bool;
}

/// Render breadcrumbs from a path vector
pub fn render_breadcrumbs(ui: &mut Ui, breadcrumb_path: &[NodeBreadcrumb]) -> Option<u64> {
    if breadcrumb_path.is_empty() {
        return None;
    }

    let mut clicked_id = None;

    ui.horizontal(|ui| {
        for (i, crumb) in breadcrumb_path.iter().enumerate() {
            if i > 0 {
                ui.label(" > ");
            }

            let is_last = i == breadcrumb_path.len() - 1;
            let label = if let Some(ref field_name) = crumb.field_name {
                format!("{}: {:?}", field_name, crumb.kind)
            } else {
                format!("{:?}", crumb.kind)
            };

            if is_last {
                // Current node - just show as label
                ui.strong(&label);
            } else {
                // Parent node - show as clickable button
                if ui.link(&label).clicked() {
                    clicked_id = Some(crumb.id);
                }
            }
        }
    });
    ui.separator();

    clicked_id
}

/// Helper trait for rendering node link fields
pub trait NodeLinkRender {
    /// Render a single link field (Component or Owned)
    fn render_single_link<L, T>(&mut self, field_name: &str, link: &mut L, owner_id: u64) -> bool
    where
        L: SingleLink<T>,
        T: FEdit + FRecursiveRender + Node + Default;

    /// Render a multiple link field
    fn render_multiple_link<L, T>(&mut self, field_name: &str, link: &mut L, owner_id: u64) -> bool
    where
        L: MultipleLink<T>,
        T: FEdit + FRecursiveRender + Node + Default;
}

impl NodeLinkRender for Ui {
    fn render_single_link<L, T>(&mut self, field_name: &str, link: &mut L, owner_id: u64) -> bool
    where
        L: SingleLink<T>,
        T: FEdit + FRecursiveRender + Node + Default,
    {
        let mut need_remove = false;
        let changed = if let Ok(loaded) = link.get_mut() {
            self.horizontal(|ui| {
                if format!("{field_name}: [tw {}]", loaded.kind())
                    .button(ui)
                    .clicked()
                {
                    ui.set_inspected_node(loaded.id());
                }
                if "[red [b -]]"
                    .cstr()
                    .button(ui)
                    .on_hover_text("Delete Node")
                    .clicked()
                {
                    need_remove = true;
                }
            });
            false
        } else {
            if self.button(format!("➕ Add {}", field_name)).clicked() {
                let mut new_node = T::default();
                new_node.set_id(next_id());
                new_node.set_owner(owner_id);
                link.state_mut().set(new_node);
                return true;
            }
            false
        };
        if need_remove {
            *link = SingleLink::none();
            true
        } else {
            changed
        }
    }

    fn render_multiple_link<L, T>(&mut self, field_name: &str, link: &mut L, owner_id: u64) -> bool
    where
        L: MultipleLink<T>,
        T: FEdit + FRecursiveRender + Node + Default,
    {
        let mut changed = false;

        self.separator();
        self.vertical(|ui| {
            ui.label(format!("{}:", field_name));
            if let Ok(items) = link.get_mut() {
                let mut to_remove: Option<usize> = None;
                for (index, item) in items.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        if format!("{field_name} #{index}: [tw {}]", item.kind())
                            .button(ui)
                            .clicked()
                        {
                            ui.set_inspected_node(item.id());
                        }
                        if "[red [b -]]"
                            .cstr()
                            .button(ui)
                            .on_hover_text("Delete Node")
                            .clicked()
                        {
                            to_remove = Some(index);
                        }
                    });
                }
                if let Some(index) = to_remove {
                    items.remove(index);
                    changed = true;
                }
                if ui.button(format!("➕ Push to {}", field_name)).clicked() {
                    let mut new_item = T::default();
                    new_item.set_id(next_id());
                    new_item.set_owner(owner_id);
                    items.push(new_item);
                    changed = true;
                }
            } else {
                if ui
                    .button(format!("➕ Create {} list", field_name))
                    .clicked()
                {
                    link.state_mut().set(vec![]);
                    changed = true;
                }
            }
        });

        changed
    }
}

/// Main composition function that handles recursive rendering with breadcrumbs
pub fn render_node_field_recursive_with_path<T: FEdit + FRecursiveRender + Node>(
    ui: &mut Ui,
    field_name: &str,
    field_node: &mut T,
    breadcrumb_path: &mut Vec<NodeBreadcrumb>,
) -> bool {
    let node_id = field_node.id();
    let node_kind = field_node.kind();
    let inspected = ui.inspected_node();
    breadcrumb_path.push(NodeBreadcrumb {
        id: node_id,
        kind: node_kind,
        field_name: Some(field_name.to_string()),
    });

    let changed = if inspected == Some(node_id) {
        if let Some(clicked_id) = render_breadcrumbs(ui, &breadcrumb_path) {
            ui.set_inspected_node(clicked_id);
        }
        let mut changed = field_node.edit(ui).changed();
        changed |= field_node.render_linked_fields(ui, breadcrumb_path);

        changed
    } else if inspected.is_none() {
        ui.set_inspected_node(node_id);
        false
    } else {
        // Other node is inspected - recurse through linked fields to find it
        field_node.render_recursive_search(ui, breadcrumb_path)
    };
    breadcrumb_path.pop();
    changed
}
