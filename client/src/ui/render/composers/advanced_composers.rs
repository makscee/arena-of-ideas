use super::*;
use crate::ui::see::CstrTrait;
use std::marker::PhantomData;

/// Composer for selectable items
pub struct SelectableComposer<T, C> {
    inner: C,
    selected: Option<T>,
    _phantom: PhantomData<T>,
}

impl<T: PartialEq + Clone, C> SelectableComposer<T, C> {
    pub fn new(inner: C, selected: Option<T>) -> Self {
        Self {
            inner,
            selected,
            _phantom: PhantomData,
        }
    }
}

impl<T: FSelectable + Clone, C: Composer<T>> Composer<T> for SelectableComposer<T, C> {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        let is_selected = self.selected.as_ref().map_or(false, |s| s == data);

        let response = if is_selected {
            Frame::new()
                .fill(ui.visuals().selection.bg_fill)
                .stroke(ui.visuals().selection.stroke)
                .corner_radius(ROUNDING)
                .inner_margin(2)
                .show(ui, |ui| self.inner.compose(data, context, ui))
                .inner
        } else {
            self.inner.compose(data, context, ui)
        };

        response
    }
}

/// Composer for filterable lists
pub struct FilteredListComposer<T, C, F> {
    item_composer: C,
    filter: F,
    empty_message: String,
    _phantom: PhantomData<T>,
}

impl<T, C, F> FilteredListComposer<T, C, F> {
    pub fn new(item_composer: C, filter: F) -> Self {
        Self {
            item_composer,
            filter,
            empty_message: "No items match the filter".to_string(),
            _phantom: PhantomData,
        }
    }

    pub fn with_empty_message(mut self, msg: String) -> Self {
        self.empty_message = msg;
        self
    }
}

impl<T: Clone, C: Composer<T>, F: Fn(&T, &Context) -> bool> Composer<Vec<T>>
    for FilteredListComposer<T, C, F>
{
    fn compose(&self, data: &Vec<T>, context: &Context, ui: &mut Ui) -> Response {
        let filtered: Vec<_> = data
            .iter()
            .filter(|item| (self.filter)(item, context))
            .collect();

        if filtered.is_empty() {
            self.empty_message
                .cstr_c(Color32::from_rgb(128, 128, 128))
                .label(ui)
        } else {
            let mut response = ui.label("");
            for item in filtered {
                response = response.union(self.item_composer.compose(item, context, ui));
            }
            response
        }
    }
}

/// Composer for sortable lists
pub struct SortedListComposer<T, C, K> {
    item_composer: C,
    key_fn: fn(&T, &Context) -> K,
    reverse: bool,
    _phantom: PhantomData<T>,
}

impl<T, C, K: Ord> SortedListComposer<T, C, K> {
    pub fn new(item_composer: C, key_fn: fn(&T, &Context) -> K) -> Self {
        Self {
            item_composer,
            key_fn,
            reverse: false,
            _phantom: PhantomData,
        }
    }

    pub fn reversed(mut self) -> Self {
        self.reverse = true;
        self
    }
}

impl<T: Clone, C: Composer<T>, K: Ord> Composer<Vec<T>> for SortedListComposer<T, C, K> {
    fn compose(&self, data: &Vec<T>, context: &Context, ui: &mut Ui) -> Response {
        let mut sorted = data.clone();
        sorted.sort_by_key(|item| (self.key_fn)(item, context));

        if self.reverse {
            sorted.reverse();
        }

        let mut response = ui.label("");
        for item in sorted {
            response = response.union(self.item_composer.compose(&item, context, ui));
        }
        response
    }
}

/// Composer for paginated lists
pub struct PaginatedComposer<T, C> {
    item_composer: C,
    items_per_page: usize,
    current_page: usize,
    _phantom: PhantomData<T>,
}

impl<T, C> PaginatedComposer<T, C> {
    pub fn new(item_composer: C, items_per_page: usize) -> Self {
        Self {
            item_composer,
            items_per_page,
            current_page: 0,
            _phantom: PhantomData,
        }
    }

    pub fn with_page(mut self, page: usize) -> Self {
        self.current_page = page;
        self
    }
}

impl<T: Clone, C: Composer<T>> Composer<Vec<T>> for PaginatedComposer<T, C> {
    fn compose(&self, data: &Vec<T>, context: &Context, ui: &mut Ui) -> Response {
        let total_pages = (data.len() + self.items_per_page - 1) / self.items_per_page;
        let start = self.current_page * self.items_per_page;
        let end = (start + self.items_per_page).min(data.len());

        let mut response = ui.label("");

        // Render items for current page
        for item in &data[start..end] {
            response = response.union(self.item_composer.compose(item, context, ui));
        }

        // Pagination controls
        ui.horizontal(|ui| {
            if ui
                .add_enabled(self.current_page > 0, egui::Button::new("◀ Previous"))
                .clicked()
            {
                // Note: In real usage, this would update state
            }

            ui.label(format!("Page {} of {}", self.current_page + 1, total_pages));

            if ui
                .add_enabled(
                    self.current_page < total_pages - 1,
                    egui::Button::new("Next ▶"),
                )
                .clicked()
            {
                // Note: In real usage, this would update state
            }
        });

        response
    }
}

/// Composer for grouped items
pub struct GroupedComposer<T, C, G> {
    item_composer: C,
    group_fn: fn(&T, &Context) -> G,
    _phantom: PhantomData<T>,
}

impl<T, C, G: Ord + std::fmt::Display> GroupedComposer<T, C, G> {
    pub fn new(item_composer: C, group_fn: fn(&T, &Context) -> G) -> Self {
        Self {
            item_composer,
            group_fn,
            _phantom: PhantomData,
        }
    }
}

impl<T: Clone, C: Composer<T>, G: Ord + std::fmt::Display + Clone> Composer<Vec<T>>
    for GroupedComposer<T, C, G>
{
    fn compose(&self, data: &Vec<T>, context: &Context, ui: &mut Ui) -> Response {
        use std::collections::BTreeMap;

        let mut groups: BTreeMap<G, Vec<T>> = BTreeMap::new();

        for item in data {
            let key = (self.group_fn)(item, context);
            groups.entry(key).or_default().push(item.clone());
        }

        let mut response = ui.label("");

        for (group_key, items) in groups {
            ui.collapsing(format!("{}", group_key), |ui| {
                for item in items {
                    response = response.union(self.item_composer.compose(&item, context, ui));
                }
            });
        }

        response
    }
}

/// Composer that adds drag and drop support
pub struct DraggableComposer<T, C> {
    inner: C,
    drag_id: String,
    _phantom: PhantomData<T>,
}

impl<T: Clone, C> DraggableComposer<T, C> {
    pub fn new(inner: C, drag_id: String) -> Self {
        Self {
            inner,
            drag_id,
            _phantom: PhantomData,
        }
    }
}

impl<T: Clone + 'static + std::marker::Send + std::marker::Sync, C: Composer<T>> Composer<T>
    for DraggableComposer<T, C>
{
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        let response = self.inner.compose(data, context, ui);

        if response.dragged() {
            ui.ctx()
                .data_mut(|d| d.insert_temp(egui::Id::new(&self.drag_id), data.clone()));
        }

        response
    }
}

/// Composer for drop target
pub struct DropTargetComposer<T, C> {
    inner: C,
    drag_id: String,
    on_drop: Box<dyn Fn(T, &Context)>,
    _phantom: PhantomData<T>,
}

impl<T: Clone + 'static, C> DropTargetComposer<T, C> {
    pub fn new<F>(inner: C, drag_id: String, on_drop: F) -> Self
    where
        F: Fn(T, &Context) + 'static,
    {
        Self {
            inner,
            drag_id,
            on_drop: Box::new(on_drop),
            _phantom: PhantomData,
        }
    }
}

impl<T: Clone + 'static, C: Composer<T>> Composer<T> for DropTargetComposer<T, C> {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        let response = self.inner.compose(data, context, ui);

        if ui.ctx().input(|i| i.pointer.any_released()) && response.hovered() {
            if let Some(dragged) = ui
                .ctx()
                .data(|d| d.get_temp::<T>(egui::Id::new(&self.drag_id)))
            {
                (self.on_drop)(dragged, context);
                ui.ctx()
                    .data_mut(|d| d.remove::<T>(egui::Id::new(&self.drag_id)));
            }
        }

        response
    }
}

/// Composer for lazy loading
pub struct LazyComposer<T, C> {
    inner: C,
    loader: Box<dyn Fn(&Context) -> Option<T>>,
    _phantom: PhantomData<T>,
}

impl<T, C> LazyComposer<T, C> {
    pub fn new<F>(inner: C, loader: F) -> Self
    where
        F: Fn(&Context) -> Option<T> + 'static,
    {
        Self {
            inner,
            loader: Box::new(loader),
            _phantom: PhantomData,
        }
    }
}

impl<T: Default, C: Composer<T>> Composer<T> for LazyComposer<T, C> {
    fn compose(&self, _data: &T, context: &Context, ui: &mut Ui) -> Response {
        if let Some(loaded_data) = (self.loader)(context) {
            self.inner.compose(&loaded_data, context, ui)
        } else {
            ui.spinner();
            ui.label("")
        }
    }
}
