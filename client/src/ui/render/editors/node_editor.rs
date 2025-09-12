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
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ParentNodeEditorComposer<T> {
    pub fn new() -> Self {
        Self {
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
    allow_add: bool,
    allow_delete: bool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ChildrenNodeEditorComposer<T> {
    pub fn new() -> Self {
        Self {
            allow_add: true,
            allow_delete: true,
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
}

impl<T> Composer<Vec<T>> for ChildrenNodeEditorComposer<T>
where
    T: Node + FTitle + Clone + 'static,
{
    fn compose(&self, data: &Vec<T>, context: &Context, ui: &mut Ui) -> Response {
        let mut response = ui.label("");

        for node in data.iter() {
            ui.horizontal(|ui| {
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
        ui.horizontal(|ui| {
            data.render_mut(context).title_label(ui);
        });
        data.edit_in_place(context, ui)
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

/// Extension methods for NodePart editing (Parent relationship)
impl<'a, T> RenderBuilder<'a, NodePart<Parent, T>>
where
    T: Node + FEdit + FTitle + Default + Clone + 'static,
{
    /// Edit a nested parent node with option to create if None
    pub fn edit_nested(self, ui: &mut Ui) -> bool {
        let RenderBuilder { data, ctx, .. } = self;
        match data {
            RenderDataRef::Mutable(node_part) => {
                ParentNodeEditComposer::new().compose_mut(node_part, ctx, ui)
            }
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }

    /// Edit a nested parent node with custom callbacks for add/delete
    pub fn edit_nested_with_callbacks<OnAdd, OnDelete>(
        self,
        ui: &mut Ui,
        on_add: OnAdd,
        on_delete: OnDelete,
    ) -> bool
    where
        OnAdd: Fn(),
        OnDelete: Fn(),
    {
        let RenderBuilder { data, ctx, .. } = self;
        match data {
            RenderDataRef::Mutable(node_part) => {
                ParentNodeEditWithCallbacks::new(on_add, on_delete).compose_mut(node_part, ctx, ui)
            }
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

/// Extension methods for NodePart editing (Child relationship)
impl<'a, T> RenderBuilder<'a, NodePart<Child, T>>
where
    T: Node + FEdit + FTitle + Default + Clone + 'static,
{
    /// Edit a nested child node
    pub fn edit_nested(self, ui: &mut Ui) -> bool {
        let RenderBuilder { data, ctx, .. } = self;
        match data {
            RenderDataRef::Mutable(node_part) => {
                ChildNodeEditComposer::new().compose_mut(node_part, ctx, ui)
            }
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

/// Extension methods for NodeParts (plural) editing
impl<'a, T> RenderBuilder<'a, NodeParts<Child, T>>
where
    T: Node + FEdit + FTitle + Default + Clone + 'static,
{
    /// Edit a list of child nodes with add/remove functionality
    pub fn edit_list(self, ui: &mut Ui) -> bool {
        let RenderBuilder { data, ctx, .. } = self;
        match data {
            RenderDataRef::Mutable(node_parts) => {
                ChildrenListEditComposer::new().compose_mut(node_parts, ctx, ui)
            }
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }

    /// Edit a list of child nodes with custom callbacks for add/delete
    pub fn edit_list_with_callbacks<OnAdd, OnDelete>(
        self,
        ui: &mut Ui,
        on_add: OnAdd,
        on_delete: OnDelete,
    ) -> bool
    where
        OnAdd: Fn(),
        OnDelete: Fn(usize),
    {
        let RenderBuilder { data, ctx, .. } = self;
        match data {
            RenderDataRef::Mutable(node_parts) => {
                ChildrenListEditWithCallbacks::new(on_add, on_delete)
                    .compose_mut(node_parts, ctx, ui)
            }
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

impl<'a, T> RenderBuilder<'a, NodeParts<Parent, T>>
where
    T: Node + FEdit + FTitle + Default + Clone + 'static,
{
    /// Edit a list of parent nodes with add/remove functionality
    pub fn edit_list(self, ui: &mut Ui) -> bool {
        let RenderBuilder { data, ctx, .. } = self;
        match data {
            RenderDataRef::Mutable(node_parts) => {
                ParentListEditComposer::new().compose_mut(node_parts, ctx, ui)
            }
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

/// Composer for editing parent nodes with create/delete options
pub struct ParentNodeEditComposer;

impl ParentNodeEditComposer {
    pub fn new() -> Self {
        Self
    }
}

impl<T> ComposerMut<NodePart<Parent, T>> for ParentNodeEditComposer
where
    T: Node + FEdit + FTitle + Default + Clone + 'static,
{
    fn compose_mut(
        &self,
        node_part: &mut NodePart<Parent, T>,
        context: &Context,
        ui: &mut Ui,
    ) -> bool {
        let mut changed = false;
        let mut should_delete = false;

        // Try to get mutable reference to the node
        if let Some(node) = node_part.get_data_mut() {
            ui.horizontal(|ui| {
                node.title(context).button(ui);
                if ui.small_button("🗑").on_hover_text("Delete").clicked() {
                    should_delete = true;
                    changed = true;
                }
            });

            changed |= node.edit(context, ui);
        } else {
            ui.horizontal(|ui| {
                ui.label(format!("{} not set", T::kind_s().cstr()));
                if ui
                    .button(format!("➕ Add {}", T::kind_s().cstr()))
                    .clicked()
                {
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
}

/// Composer for editing child nodes
pub struct ChildNodeEditComposer;

impl ChildNodeEditComposer {
    pub fn new() -> Self {
        Self
    }
}

impl<T> ComposerMut<NodePart<Child, T>> for ChildNodeEditComposer
where
    T: Node + FEdit + FTitle + Default + Clone + 'static,
{
    fn compose_mut(
        &self,
        node_part: &mut NodePart<Child, T>,
        context: &Context,
        ui: &mut Ui,
    ) -> bool {
        let mut changed = false;

        // For child nodes, we can't create/delete directly, just edit if exists
        if let Some(node) = node_part.get_data_mut() {
            ui.horizontal(|ui| {
                node.title(context).label(ui);
            });
            changed |= node.edit(context, ui);
        } else {
            ui.label(format!("{} not available", T::kind_s().cstr()));
        }

        changed
    }
}

/// Composer for editing lists of child nodes
pub struct ChildrenListEditComposer;

impl ChildrenListEditComposer {
    pub fn new() -> Self {
        Self
    }
}

impl<T> ComposerMut<NodeParts<Child, T>> for ChildrenListEditComposer
where
    T: Node + FEdit + FTitle + Default + Clone + 'static,
{
    fn compose_mut(
        &self,
        node_parts: &mut NodeParts<Child, T>,
        context: &Context,
        ui: &mut Ui,
    ) -> bool {
        let mut changed = false;
        let mut to_delete = Vec::new();

        // Get mutable reference to child nodes
        if let Some(children) = node_parts.get_data_mut() {
            if children.is_empty() {
                ui.label(format!("No {} items", T::kind_s().cstr()));
            } else {
                for (i, child) in children.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        (&*child).render(context).title_button(ui);
                        if ui.small_button("🗑").clicked() {
                            to_delete.push(i);
                            changed = true;
                        }
                    });

                    changed |= child.edit(context, ui);
                }
            }

            // Remove deleted items (in reverse order to maintain indices)
            for &index in to_delete.iter().rev() {
                children.remove(index);
            }

            if ui
                .button(format!("➕ Add {}", T::kind_s().cstr()))
                .clicked()
            {
                children.push(T::default());
                changed = true;
            }
        } else {
            ui.horizontal(|ui| {
                ui.label(format!("No {} items", T::kind_s().cstr()));
                if ui
                    .button(format!("➕ Add {}", T::kind_s().cstr()))
                    .clicked()
                {
                    node_parts.set_data(vec![T::default()]);
                    changed = true;
                }
            });
        }

        changed
    }
}

/// Composer for editing lists with closure callbacks
pub struct ChildrenListEditWithCallbacks<OnAdd, OnDelete> {
    on_add: OnAdd,
    on_delete: OnDelete,
}

impl<OnAdd, OnDelete> ChildrenListEditWithCallbacks<OnAdd, OnDelete>
where
    OnAdd: Fn(),
    OnDelete: Fn(usize),
{
    pub fn new(on_add: OnAdd, on_delete: OnDelete) -> Self {
        Self { on_add, on_delete }
    }
}

impl<T, OnAdd, OnDelete> ComposerMut<NodeParts<Child, T>>
    for ChildrenListEditWithCallbacks<OnAdd, OnDelete>
where
    T: Node + FEdit + FTitle + Default + Clone + 'static,
    OnAdd: Fn(),
    OnDelete: Fn(usize),
{
    fn compose_mut(
        &self,
        node_parts: &mut NodeParts<Child, T>,
        context: &Context,
        ui: &mut Ui,
    ) -> bool {
        let mut changed = false;

        // Get mutable reference to child nodes
        if let Some(children) = node_parts.get_data_mut() {
            if children.is_empty() {
                ui.label(format!("No {} items", T::kind_s().cstr()));
            } else {
                for (i, child) in children.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        (&*child).render(context).title_button(ui);
                        if ui.small_button("🗑").clicked() {
                            (self.on_delete)(i);
                            changed = true;
                        }
                    });

                    changed |= child.edit(context, ui);
                }
            }

            if ui
                .button(format!("➕ Add {}", T::kind_s().cstr()))
                .clicked()
            {
                (self.on_add)();
                changed = true;
            }
        } else {
            ui.horizontal(|ui| {
                ui.label(format!("No {} items", T::kind_s().cstr()));
                if ui
                    .button(format!("➕ Add {}", T::kind_s().cstr()))
                    .clicked()
                {
                    (self.on_add)();
                    changed = true;
                }
            });
        }

        changed
    }
}

/// Composer for editing parent node with closure callbacks
pub struct ParentNodeEditWithCallbacks<OnAdd, OnDelete> {
    on_add: OnAdd,
    on_delete: OnDelete,
}

impl<OnAdd, OnDelete> ParentNodeEditWithCallbacks<OnAdd, OnDelete>
where
    OnAdd: Fn(),
    OnDelete: Fn(),
{
    pub fn new(on_add: OnAdd, on_delete: OnDelete) -> Self {
        Self { on_add, on_delete }
    }
}

impl<T, OnAdd, OnDelete> ComposerMut<NodePart<Parent, T>>
    for ParentNodeEditWithCallbacks<OnAdd, OnDelete>
where
    T: Node + FEdit + FTitle + Default + Clone + 'static,
    OnAdd: Fn(),
    OnDelete: Fn(),
{
    fn compose_mut(
        &self,
        node_part: &mut NodePart<Parent, T>,
        context: &Context,
        ui: &mut Ui,
    ) -> bool {
        let mut changed = false;

        if let Some(node) = node_part.get_data_mut() {
            ui.horizontal(|ui| {
                node.title(context).button(ui);
                if ui.small_button("🗑").on_hover_text("Delete").clicked() {
                    (self.on_delete)();
                    changed = true;
                }
            });

            changed |= node.edit(context, ui);
        } else {
            ui.horizontal(|ui| {
                ui.label(format!("{} not set", T::kind_s().cstr()));
                if ui
                    .button(format!("➕ Add {}", T::kind_s().cstr()))
                    .clicked()
                {
                    (self.on_add)();
                    changed = true;
                }
            });
        }

        changed
    }
}

/// Composer for editing lists of parent nodes
pub struct ParentListEditComposer;

impl ParentListEditComposer {
    pub fn new() -> Self {
        Self
    }
}

impl<T> ComposerMut<NodeParts<Parent, T>> for ParentListEditComposer
where
    T: Node + FEdit + FTitle + Default + Clone + 'static,
{
    fn compose_mut(
        &self,
        node_parts: &mut NodeParts<Parent, T>,
        context: &Context,
        ui: &mut Ui,
    ) -> bool {
        let mut changed = false;
        let mut to_delete = Vec::new();

        // Get all parent nodes (mutable)
        let parents = if let Some(nodes) = node_parts.get_data_mut() {
            nodes
        } else {
            // If no data exists yet, create an empty list for potential additions
            if ui
                .button(format!("➕ Add {}", T::kind_s().cstr()))
                .clicked()
            {
                let new_node = T::default();
                node_parts.set_data(vec![new_node]);
                changed = true;
            }
            return changed;
        };

        if parents.is_empty() {
            ui.label(format!("No {} items", T::kind_s().cstr()));
        } else {
            for (index, parent) in parents.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    parent.title(context).button(ui);
                    if ui.small_button("🗑").on_hover_text("Delete").clicked() {
                        to_delete.push(index);
                        changed = true;
                    }
                });

                changed |= parent.edit(context, ui);
            }
        }

        if ui
            .button(format!("➕ Add {}", T::kind_s().cstr()))
            .clicked()
        {
            // Add a new default item to the list
            let new_node = T::default();
            parents.push(new_node);
            changed = true;
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
