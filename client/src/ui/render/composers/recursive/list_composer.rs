use super::*;

/// A unified list composer that can render any list with custom item composers
pub struct ListComposer<T, C> {
    item_composer: C,
    allow_reorder: bool,
    allow_add: bool,
    allow_remove: bool,
    show_index: bool,
    layout: ListLayout,
    _phantom: PhantomData<T>,
}

/// Layout options for list rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ListLayout {
    Vertical,
    Horizontal,
    Wrapped { max_width: f32 },
    Grid { columns: usize },
}

impl<T, C> ListComposer<T, C> {
    pub fn new(item_composer: C) -> Self {
        Self {
            item_composer,
            allow_reorder: false,
            allow_add: false,
            allow_remove: false,
            show_index: false,
            layout: ListLayout::Vertical,
            _phantom: PhantomData,
        }
    }

    pub fn with_reorder(mut self, allow: bool) -> Self {
        self.allow_reorder = allow;
        self
    }

    pub fn with_add(mut self, allow: bool) -> Self {
        self.allow_add = allow;
        self
    }

    pub fn with_remove(mut self, allow: bool) -> Self {
        self.allow_remove = allow;
        self
    }

    pub fn with_controls(mut self, reorder: bool, add: bool, remove: bool) -> Self {
        self.allow_reorder = reorder;
        self.allow_add = add;
        self.allow_remove = remove;
        self
    }

    pub fn show_index(mut self, show: bool) -> Self {
        self.show_index = show;
        self
    }

    pub fn with_layout(mut self, layout: ListLayout) -> Self {
        self.layout = layout;
        self
    }
}

/// Immutable list rendering
impl<T: Clone, C: Composer<T>> Composer<Vec<T>> for ListComposer<T, C> {
    fn compose(&self, data: &Vec<T>, context: &Context, ui: &mut Ui) -> Response {
        let mut response = ui.label("");

        match self.layout {
            ListLayout::Vertical => {
                ui.vertical(|ui| {
                    for (i, item) in data.iter().enumerate() {
                        response = response.union(self.render_item(item, i, context, ui));
                    }
                });
            }
            ListLayout::Horizontal => {
                ui.horizontal(|ui| {
                    for (i, item) in data.iter().enumerate() {
                        response = response.union(self.render_item(item, i, context, ui));
                    }
                });
            }
            ListLayout::Wrapped { max_width } => {
                ui.horizontal_wrapped(|ui| {
                    ui.set_max_width(max_width);
                    for (i, item) in data.iter().enumerate() {
                        response = response.union(self.render_item(item, i, context, ui));
                    }
                });
            }
            ListLayout::Grid { columns } => {
                egui::Grid::new(ui.id().with("list_grid"))
                    .num_columns(columns)
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        for (i, item) in data.iter().enumerate() {
                            response = response.union(self.render_item(item, i, context, ui));
                            if (i + 1) % columns == 0 {
                                ui.end_row();
                            }
                        }
                    });
            }
        }

        response
    }
}

impl<T: Clone, C: Composer<T>> ListComposer<T, C> {
    fn render_item(&self, item: &T, index: usize, context: &Context, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            if self.show_index {
                format!("[{}]", index).cstr_c(Color32::DARK_GRAY).label(ui);
            }
            self.item_composer.compose(item, context, ui)
        })
        .inner
    }
}

/// Mutable list rendering with controls
impl<T: Clone + Default, C: ComposerMut<T>> ComposerMut<Vec<T>> for ListComposer<T, C> {
    fn compose_mut(&self, data: &mut Vec<T>, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        let mut to_remove = None;
        let mut to_move = None;

        match self.layout {
            ListLayout::Vertical => {
                ui.vertical(|ui| {
                    changed |=
                        self.render_mutable_list(data, context, ui, &mut to_remove, &mut to_move);
                });
            }
            ListLayout::Horizontal => {
                ui.horizontal(|ui| {
                    changed |=
                        self.render_mutable_list(data, context, ui, &mut to_remove, &mut to_move);
                });
            }
            ListLayout::Wrapped { max_width } => {
                ui.horizontal_wrapped(|ui| {
                    ui.set_max_width(max_width);
                    changed |=
                        self.render_mutable_list(data, context, ui, &mut to_remove, &mut to_move);
                });
            }
            ListLayout::Grid { columns } => {
                egui::Grid::new(ui.id().with("list_grid_mut"))
                    .num_columns(columns)
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        let len = data.len();
                        for (i, item) in data.iter_mut().enumerate() {
                            changed |= self.render_mutable_item(
                                item,
                                i,
                                len,
                                context,
                                ui,
                                &mut to_remove,
                                &mut to_move,
                            );
                            if (i + 1) % columns == 0 {
                                ui.end_row();
                            }
                        }
                    });
            }
        }

        // Handle removals
        if let Some(idx) = to_remove {
            data.remove(idx);
            changed = true;
        }

        // Handle moves
        if let Some((from, to)) = to_move {
            data.swap(from, to);
            changed = true;
        }

        // Add button
        if self.allow_add {
            ui.separator();
            if ui.button("➕ Add Item").clicked() {
                data.push(T::default());
                changed = true;
            }
        }

        changed
    }
}

impl<T: Clone + Default, C: ComposerMut<T>> ListComposer<T, C> {
    fn render_mutable_list(
        &self,
        data: &mut Vec<T>,
        context: &Context,
        ui: &mut Ui,
        to_remove: &mut Option<usize>,
        to_move: &mut Option<(usize, usize)>,
    ) -> bool {
        let mut changed = false;
        let len = data.len();

        for (i, item) in data.iter_mut().enumerate() {
            changed |= self.render_mutable_item(item, i, len, context, ui, to_remove, to_move);
        }

        changed
    }

    fn render_mutable_item(
        &self,
        item: &mut T,
        index: usize,
        total_len: usize,
        context: &Context,
        ui: &mut Ui,
        to_remove: &mut Option<usize>,
        to_move: &mut Option<(usize, usize)>,
    ) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            // Reorder controls
            if self.allow_reorder {
                ui.vertical(|ui| {
                    ui.set_width(20.0);
                    ui.add_enabled_ui(index > 0, |ui| {
                        if ui.small_button("↑").on_hover_text("Move up").clicked() {
                            *to_move = Some((index, index - 1));
                        }
                    });
                    ui.add_enabled_ui(index < total_len - 1, |ui| {
                        if ui.small_button("↓").on_hover_text("Move down").clicked() {
                            *to_move = Some((index, index + 1));
                        }
                    });
                });
            }

            // Index label
            if self.show_index {
                format!("[{}]", index).cstr_c(Color32::DARK_GRAY).label(ui);
            }

            // Item content
            ui.vertical(|ui| {
                changed |= self.item_composer.compose_mut(item, context, ui);
            });

            // Remove button
            if self.allow_remove {
                if ui.small_button("🗑").on_hover_text("Remove").clicked() {
                    *to_remove = Some(index);
                }
            }
        });

        ui.separator();
        changed
    }
}

/// Specialized list composer for recursive values
pub struct RecursiveListComposer {
    layout: ListLayout,
    allow_reorder: bool,
    allow_add: bool,
    allow_remove: bool,
    show_index: bool,
}

impl Default for RecursiveListComposer {
    fn default() -> Self {
        Self {
            layout: ListLayout::Vertical,
            allow_reorder: true,
            allow_add: true,
            allow_remove: true,
            show_index: true,
        }
    }
}

impl RecursiveListComposer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_layout(mut self, layout: ListLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_controls(mut self, reorder: bool, add: bool, remove: bool) -> Self {
        self.allow_reorder = reorder;
        self.allow_add = add;
        self.allow_remove = remove;
        self
    }
}

impl<T> ComposerMut<Vec<T>> for RecursiveListComposer
where
    T: FRecursive + Clone + Default,
{
    fn compose_mut(&self, data: &mut Vec<T>, context: &Context, ui: &mut Ui) -> bool {
        let item_composer = RecursiveComposer::new(
            |ui: &mut Ui, context: &Context, field: &mut RecursiveFieldMut<'_>| -> bool {
                call_on_recursive_value_mut!(field, edit, context, ui)
            },
        )
        .with_layout(RecursiveLayout::HorizontalVertical);

        let list_composer = ListComposer::new(item_composer)
            .with_layout(self.layout)
            .with_controls(self.allow_reorder, self.allow_add, self.allow_remove)
            .show_index(self.show_index);

        list_composer.compose_mut(data, context, ui)
    }
}

impl<T> Composer<Vec<T>> for RecursiveListComposer
where
    T: FRecursive + Clone,
{
    fn compose(&self, data: &Vec<T>, context: &Context, ui: &mut Ui) -> Response {
        let item_composer = RecursiveComposer::new(
            |ui: &mut Ui, context: &Context, field: &RecursiveField<'_>| -> Response {
                call_on_recursive_value!(field, display, context, ui);
                ui.label("")
            },
        )
        .with_layout(RecursiveLayout::HorizontalVertical);

        let list_composer = ListComposer::new(item_composer)
            .with_layout(self.layout)
            .show_index(self.show_index);

        list_composer.compose(data, context, ui)
    }
}

/// Helper function to create a list composer with a value renderer
pub fn list_with_renderer<T, F>(
    renderer: F,
    layout: ListLayout,
) -> ListComposer<T, impl Composer<T>>
where
    T: Clone,
    F: FnMut(&T, &Context, &mut Ui) -> Response + 'static,
{
    struct RendererComposer<T, F> {
        renderer: std::cell::RefCell<F>,
        _phantom: PhantomData<T>,
    }

    impl<T, F> Composer<T> for RendererComposer<T, F>
    where
        F: FnMut(&T, &Context, &mut Ui) -> Response,
    {
        fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
            (self.renderer.borrow_mut())(data, context, ui)
        }
    }

    ListComposer::new(RendererComposer {
        renderer: std::cell::RefCell::new(renderer),
        _phantom: PhantomData,
    })
    .with_layout(layout)
}

/// Helper function to create a mutable list composer with a value renderer
pub fn list_mut_with_renderer<T, F>(
    renderer: F,
    layout: ListLayout,
) -> ListComposer<T, impl ComposerMut<T>>
where
    T: Clone + Default,
    F: FnMut(&mut T, &Context, &mut Ui) -> bool + 'static,
{
    struct RendererComposerMut<T, F> {
        renderer: std::cell::RefCell<F>,
        _phantom: PhantomData<T>,
    }

    impl<T, F> ComposerMut<T> for RendererComposerMut<T, F>
    where
        F: FnMut(&mut T, &Context, &mut Ui) -> bool,
    {
        fn compose_mut(&self, data: &mut T, context: &Context, ui: &mut Ui) -> bool {
            (self.renderer.borrow_mut())(data, context, ui)
        }
    }

    ListComposer::new(RendererComposerMut {
        renderer: std::cell::RefCell::new(renderer),
        _phantom: PhantomData,
    })
    .with_layout(layout)
}
