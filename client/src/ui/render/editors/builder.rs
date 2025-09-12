use super::super::*;
use crate::nodes::*;

/// Configurable builder for node editors
#[derive(Debug, Clone)]
pub struct NodeEditorBuilder {
    /// Whether to show nested parts inline or collapsed
    pub show_nested_inline: bool,
    /// Whether to allow creating new nested parts
    pub allow_create_nested: bool,
    /// Whether to allow deleting nested parts
    pub allow_delete_nested: bool,
    /// Whether to show collapsible sections for nested parts
    pub collapsible_sections: bool,
    /// Whether to show field labels
    pub show_field_labels: bool,
    /// Whether to show add/remove buttons for lists
    pub show_list_controls: bool,
    /// Maximum nesting depth to show inline
    pub max_inline_depth: usize,
    /// Current nesting depth (internal use)
    current_depth: usize,
}

impl Default for NodeEditorBuilder {
    fn default() -> Self {
        Self {
            show_nested_inline: false,
            allow_create_nested: true,
            allow_delete_nested: true,
            collapsible_sections: true,
            show_field_labels: true,
            show_list_controls: true,
            max_inline_depth: 2,
            current_depth: 0,
        }
    }
}

impl NodeEditorBuilder {
    /// Create a new node editor builder with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure whether to show nested parts inline or collapsed
    pub fn with_nested_inline(mut self, inline: bool) -> Self {
        self.show_nested_inline = inline;
        self
    }

    /// Configure whether to allow creating new nested parts
    pub fn with_create_nested(mut self, allow: bool) -> Self {
        self.allow_create_nested = allow;
        self
    }

    /// Configure whether to allow deleting nested parts
    pub fn with_delete_nested(mut self, allow: bool) -> Self {
        self.allow_delete_nested = allow;
        self
    }

    /// Configure whether to use collapsible sections
    pub fn with_collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible_sections = collapsible;
        self
    }

    /// Configure whether to show field labels
    pub fn with_field_labels(mut self, show: bool) -> Self {
        self.show_field_labels = show;
        self
    }

    /// Configure whether to show list controls (add/remove buttons)
    pub fn with_list_controls(mut self, show: bool) -> Self {
        self.show_list_controls = show;
        self
    }

    /// Set maximum depth for inline display
    pub fn with_max_inline_depth(mut self, depth: usize) -> Self {
        self.max_inline_depth = depth;
        self
    }

    /// Create a nested builder with increased depth
    fn nested(&self) -> Self {
        Self {
            current_depth: self.current_depth + 1,
            ..*self
        }
    }

    /// Check if we should show this level inline
    pub fn should_show_inline(&self) -> bool {
        self.show_nested_inline || self.current_depth < self.max_inline_depth
    }

    /// Render a node with the configured settings
    pub fn render_node<T>(&self, node: &mut T, context: &Context, ui: &mut Ui) -> bool
    where
        T: FEdit + FTitle,
    {
        if self.should_show_inline() {
            node.edit(context, ui)
        } else {
            let mut changed = false;
            if ui.collapsing(node.title(context).get_text(), |ui| {
                changed |= self.nested().render_node(node, context, ui);
            }).header_response.clicked() {
                // Handle header click if needed
            }
            changed
        }
    }

    /// Render a NodePart with the configured settings
    pub fn render_node_part<T>(&self, node_part: &mut NodePart<Parent, T>, context: &Context, ui: &mut Ui, field_name: &str) -> bool
    where
        T: Node + FEdit + FTitle + Default + Clone + 'static,
    {
        let mut changed = false;
        let mut should_delete = false;

        if self.show_field_labels {
            ui.label(format!("{}:", field_name));
        }

        if let Some(node) = node_part.get_data_mut() {
            if self.should_show_inline() {
                ui.group(|ui| {
                    if self.allow_delete_nested && ui.small_button("🗑").on_hover_text("Remove").clicked() {
                        // Mark for deletion
                        should_delete = true;
                        changed = true;
                    }
                    changed |= self.nested().render_node(node, context, ui);
                });
            } else {
                let node_title = node.title(context).get_text();
                if ui.collapsing(format!("{}: {}", field_name, node_title), |ui| {
                    ui.horizontal(|ui| {
                        if self.allow_delete_nested && ui.small_button("🗑").on_hover_text("Remove").clicked() {
                            // Mark for deletion
                            should_delete = true;
                            changed = true;
                        }
                    });
                    changed |= self.nested().render_node(node, context, ui);
                }).header_response.clicked() {
                    // Handle header click
                }
            }
        } else {
            ui.horizontal(|ui| {
                if self.show_field_labels {
                    ui.label(format!("{}: not set", field_name));
                } else {
                    ui.label("Not set");
                }
                
                if self.allow_create_nested && ui.button("➕ Add").clicked() {
                    // Create default node and set it
                    let default_node = T::default();
                    node_part.set_data(default_node);
                    changed = true;
                }
            });
        }

        // Handle deletion outside the closure to avoid borrowing conflicts
        if should_delete {
            node_part.set_none();
        }

        changed
    }

    /// Render a list of nodes with the configured settings
    pub fn render_node_list<T>(&self, node_parts: &mut NodeParts<Parent, T>, context: &Context, ui: &mut Ui, field_name: &str) -> bool
    where
        T: Node + FEdit + FTitle + Default + Clone + 'static,
    {
        let mut changed = false;
        let mut to_delete = Vec::new();

        if self.show_field_labels {
            ui.label(format!("{}:", field_name));
        }

        let parents = if let Some(nodes) = node_parts.get_data_mut() { nodes } else { 
            // If no data exists yet, create an empty list for potential additions
            if self.show_list_controls && ui.button("➕ Add").clicked() {
                let new_node = T::default();
                node_parts.set_data(vec![new_node]);
                changed = true;
            }
            return changed; 
        };
        
        if parents.is_empty() {
            ui.horizontal(|ui| {
                ui.label("No items");
                if self.show_list_controls && ui.button("➕ Add").clicked() {
                    let new_node = T::default();
                    parents.push(new_node);
                    changed = true;
                }
            });
        } else {
            for (index, parent) in parents.iter_mut().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        parent.title(context).button(ui);
                        
                        if self.show_list_controls && self.allow_delete_nested {
                            if ui.small_button("🗑").on_hover_text("Delete").clicked() {
                                to_delete.push(index);
                                changed = true;
                            }
                        }
                    });
                    
                    ui.separator();
                    
                    if self.should_show_inline() {
                        changed |= self.nested().render_node(parent, context, ui);
                    } else {
                        if ui.collapsing("Edit", |ui| {
                            changed |= self.nested().render_node(parent, context, ui);
                        }).header_response.clicked() {
                            // Handle header click
                        }
                    }
                });
                
                ui.separator();
            }
            
            if self.show_list_controls {
                if ui.button("➕ Add").clicked() {
                    let new_node = T::default();
                    parents.push(new_node);
                    changed = true;
                }
            }
        }

        // Remove items marked for deletion (in reverse order to preserve indices)
        for &index in to_delete.iter().rev() {
            if index < parents.len() {
                parents.remove(index);
                changed = true;
            }
        }

        changed
    }
}

/// Extension methods for RenderBuilder to use the configurable builder
impl<'a, T> RenderBuilder<'a, T>
where
    T: FEdit + FTitle,
{
    /// Edit using a configurable builder
    pub fn edit_with_builder(self, builder: &NodeEditorBuilder, ui: &mut Ui) -> bool {
        let RenderBuilder { data, ctx, .. } = self;
        match data {
            RenderDataRef::Mutable(data) => builder.render_node(data, ctx, ui),
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

/// Preset builders for common use cases
impl NodeEditorBuilder {
    /// Create a builder for compact inline editing
    pub fn compact() -> Self {
        Self::new()
            .with_nested_inline(true)
            .with_collapsible(false)
            .with_field_labels(false)
            .with_max_inline_depth(3)
    }

    /// Create a builder for detailed editing with all features
    pub fn detailed() -> Self {
        Self::new()
            .with_nested_inline(false)
            .with_collapsible(true)
            .with_field_labels(true)
            .with_list_controls(true)
            .with_max_inline_depth(1)
    }

    /// Create a builder for read-only viewing
    pub fn readonly() -> Self {
        Self::new()
            .with_create_nested(false)
            .with_delete_nested(false)
            .with_list_controls(false)
            .with_nested_inline(true)
    }

    /// Create a builder for minimal editing
    pub fn minimal() -> Self {
        Self::new()
            .with_field_labels(false)
            .with_collapsible(false)
            .with_list_controls(false)
            .with_nested_inline(true)
            .with_max_inline_depth(2)
    }
}