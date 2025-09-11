use super::super::*;
use crate::nodes::*;

/// Extension trait for editing nodes with the render system
pub trait NodeEditor: Node + Clone {
    /// Render an editable node with title and controls
    fn render_editor(&self, context: &Context, ui: &mut Ui) -> NodeEditorResponse
    where
        Self: FTitle,
    {
        let response = NodeEditorResponse::default();

        ui.group(|ui| {
            // Title without menu for now - can be extended with with_menu() if needed
            self.render(context).title(ui);
        });

        response
    }

    /// Render an inline editor for the node
    fn render_inline_editor(&self, context: &Context, ui: &mut Ui) -> Response
    where
        Self: FTag,
    {
        ui.horizontal(|ui| {
            self.render(context).tag(ui);
            if ui.small_button("✏").clicked() {
                // Open editor dialog
            }
        })
        .response
    }
}

/// Response from node editor operations
#[derive(Default)]
pub struct NodeEditorResponse {
    pub changed: bool,
    pub deleted: bool,
    pub replaced: Option<Box<dyn std::any::Any>>,
    pub action: Option<NodeEditorAction>,
}

pub enum NodeEditorAction {
    Navigate(u64),
    OpenDialog(u64),
    Custom(String),
}

/// Composer for editing nodes
pub struct NodeEditorComposer {
    show_info: bool,
    show_stats: bool,
    collapsible: bool,
}

impl Default for NodeEditorComposer {
    fn default() -> Self {
        Self {
            show_info: true,
            show_stats: true,
            collapsible: false,
        }
    }
}

impl NodeEditorComposer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_info(mut self, show: bool) -> Self {
        self.show_info = show;
        self
    }

    pub fn with_stats(mut self, show: bool) -> Self {
        self.show_stats = show;
        self
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }
}

impl<T> Composer<T> for NodeEditorComposer
where
    T: Node + FTitle + Clone + 'static,
{
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        let mut response = ui.label("");

        if self.collapsible {
            let header_response = ui
                .collapsing(data.title(context).get_text(), |ui| {
                    self.render_editor_content(data, context, ui)
                })
                .header_response;
            response = response.union(header_response);
        } else {
            response = self.render_editor_content(data, context, ui);
        }

        response
    }
}

impl NodeEditorComposer {
    fn render_editor_content<T>(&self, data: &T, context: &Context, ui: &mut Ui) -> Response
    where
        T: Node + FTitle + Clone + 'static,
    {
        ui.group(|ui| {
            // Header with title
            ui.horizontal(|ui| {
                data.render(context).title_button(ui);

                // Optional info - skip for now since FInfo isn't implemented for all nodes
                // if self.show_info {
                //     if let Some(info_provider) =
                //         (data as &dyn std::any::Any).downcast_ref::<&dyn FInfo>()
                //     {
                //         ui.separator();
                //         info_provider.info(context).label(ui);
                //     }
                // }
            });

            // Stats if available - skip for now since downcasting is complex
            // if self.show_stats {
            //     if let Some(stats_provider) =
            //         (data as &dyn std::any::Any).downcast_ref::<&dyn FStats>()
            //     {
            //         ui.separator();
            //         ui.horizontal(|ui| {
            //             for (var_name, var_value) in stats_provider.stats(context) {
            //                 TagWidget::new_var_value(var_name, var_value).ui(ui);
            //             }
            //         });
            //     }
            // }
        })
        .response
    }
}

/// Composer for parent node editing
pub struct ParentNodeEditorComposer<T> {
    child_id: u64,
    owner_id: u64,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ParentNodeEditorComposer<T> {
    pub fn new(child_id: u64, owner_id: u64) -> Self {
        Self {
            child_id,
            owner_id,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Composer<Option<T>> for ParentNodeEditorComposer<T>
where
    T: Node + FTitle + Clone + Default + 'static,
{
    fn compose(&self, data: &Option<T>, context: &Context, ui: &mut Ui) -> Response {
        let mut response = ui.label("");

        if let Some(parent) = data {
            ui.collapsing(format!("{}", T::kind_s().cstr()), |ui| {
                ui.horizontal(|ui| {
                    // Use render system for display
                    response = parent.render(context).title_button(ui);
                    // Info display would go here if FInfo was implemented
                });

                ui.group(|ui| {
                    // For editing, we still need the old system
                    // This would need to be migrated to FEdit when available
                    ui.label("Edit mode would go here");
                });
            });
        } else {
            ui.label(format!("{} not set", T::kind_s().cstr()));
            if ui
                .button(format!("➕ Add Default {}", T::kind_s().cstr()))
                .clicked()
            {
                // Create default node
                response = response.union(ui.label("Would create default node"));
            }
        }

        response
    }
}

/// Composer for children node list editing
pub struct ChildrenNodeEditorComposer<T> {
    parent_id: u64,
    owner_id: u64,
    allow_add: bool,
    allow_delete: bool,
    allow_reorder: bool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ChildrenNodeEditorComposer<T> {
    pub fn new(parent_id: u64, owner_id: u64) -> Self {
        Self {
            parent_id,
            owner_id,
            allow_add: true,
            allow_delete: true,
            allow_reorder: false,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_add(mut self, allow: bool) -> Self {
        self.allow_add = allow;
        self
    }

    pub fn with_delete(mut self, allow: bool) -> Self {
        self.allow_delete = allow;
        self
    }

    pub fn with_reorder(mut self, allow: bool) -> Self {
        self.allow_reorder = allow;
        self
    }
}

impl<T> Composer<Vec<T>> for ChildrenNodeEditorComposer<T>
where
    T: Node + FTitle + Clone + 'static,
{
    fn compose(&self, data: &Vec<T>, context: &Context, ui: &mut Ui) -> Response {
        let mut response = ui.label("");

        for (i, node) in data.iter().enumerate() {
            ui.horizontal(|ui| {
                if self.allow_reorder {
                    ui.vertical(|ui| {
                        if i > 0 && ui.small_button("↑").clicked() {
                            // Handle move up
                        }
                        if i < data.len() - 1 && ui.small_button("↓").clicked() {
                            // Handle move down
                        }
                    });
                }

                // Use render system - context menu requires specific traits
                response = response.union(node.render(context).title(ui));

                if self.allow_delete && ui.small_button("🗑").clicked() {
                    // Handle delete
                }
            });
        }

        if self.allow_add {
            if ui
                .button(format!("➕ Add {}", T::kind_s().cstr()))
                .clicked()
            {
                // Handle add
                response = response.union(ui.label(""));
            }
        }

        response
    }
}

/// Extension methods for RenderBuilder for node editing
impl<'a, T> RenderBuilder<'a, T>
where
    T: Node + FTitle + Clone + 'static,
{
    /// Render as an editable node
    pub fn node_editor(self, ui: &mut Ui) -> Response {
        NodeEditorComposer::new().compose(self.data(), self.context(), ui)
    }

    /// Render as a collapsible node editor
    pub fn node_editor_collapsible(self, ui: &mut Ui) -> Response {
        NodeEditorComposer::new()
            .collapsible(true)
            .compose(self.data(), self.context(), ui)
    }
}

/// Helper trait for nodes that can be edited in place
pub trait InPlaceEdit: Node {
    fn edit_in_place(&mut self, context: &Context, ui: &mut Ui) -> bool;
}

// Implement InPlaceEdit for common node types
impl InPlaceEdit for NUnit {
    fn edit_in_place(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Name:");
            changed |= ui.text_edit_singleline(&mut self.unit_name).changed();
        });

        changed
    }
}

impl InPlaceEdit for NHouse {
    fn edit_in_place(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("House Name:");
            changed |= ui.text_edit_singleline(&mut self.house_name).changed();
        });

        changed
    }
}

impl InPlaceEdit for NAbilityMagic {
    fn edit_in_place(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Ability:");
            changed |= ui.text_edit_singleline(&mut self.ability_name).changed();
        });

        changed
    }
}

impl InPlaceEdit for NStatusMagic {
    fn edit_in_place(&mut self, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Status:");
            changed |= ui.text_edit_singleline(&mut self.status_name).changed();
        });

        changed
    }
}

/// Composer for in-place editing
pub struct InPlaceEditComposer;

impl<T> ComposerMut<T> for InPlaceEditComposer
where
    T: InPlaceEdit + FTitle,
{
    fn compose_mut(&self, data: &mut T, context: &Context, ui: &mut Ui) -> bool {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                data.render_mut(context).title_label(ui);
                ui.separator();
            });
            data.edit_in_place(context, ui)
        })
        .inner
    }
}

/// Extension for mutable render builder
impl<'a, T> RenderBuilder<'a, T>
where
    T: InPlaceEdit + FTitle,
{
    /// Edit the node in place
    pub fn edit_in_place(self, ui: &mut Ui) -> bool {
        let RenderBuilder { data, ctx, .. } = self;
        match data {
            RenderDataRef::Mutable(data) => InPlaceEditComposer.compose_mut(data, ctx, ui),
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}
