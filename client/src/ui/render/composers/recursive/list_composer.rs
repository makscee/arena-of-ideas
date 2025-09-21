use super::*;
use std::marker::PhantomData;

/// A unified list composer that can render any list with custom item functions
pub struct ListComposer<T, F> {
    item_fn: F,
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

impl<T, F> ListComposer<T, F> {
    pub fn new(item_fn: F) -> Self {
        Self {
            item_fn,
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
impl<T: Clone, F> ListComposer<T, F>
where
    F: Fn(&T, &Context, &mut Ui) -> Response,
{
    fn render_list(&self, data: &Vec<T>, context: &Context, ui: &mut Ui) -> Response {
        match self.layout {
            ListLayout::Vertical => {
                ui.vertical(|ui| {
                    for (i, item) in data.iter().enumerate() {
                        self.render_item(item, i, context, ui);
                    }
                })
                .response
            }
            ListLayout::Horizontal => {
                ui.horizontal(|ui| {
                    for (i, item) in data.iter().enumerate() {
                        self.render_item(item, i, context, ui);
                    }
                })
                .response
            }
            ListLayout::Wrapped { max_width } => {
                ui.horizontal_wrapped(|ui| {
                    ui.set_max_width(max_width);
                    for (i, item) in data.iter().enumerate() {
                        self.render_item(item, i, context, ui);
                    }
                })
                .response
            }
            ListLayout::Grid { columns } => {
                egui::Grid::new(ui.id().with("list_grid"))
                    .num_columns(columns)
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        for (i, item) in data.iter().enumerate() {
                            self.render_item(item, i, context, ui);
                            if (i + 1) % columns == 0 {
                                ui.end_row();
                            }
                        }
                    })
                    .response
            }
        }
    }

    fn render_item(&self, item: &T, index: usize, context: &Context, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            if self.show_index {
                format!("[{}]", index).cstr_c(Color32::DARK_GRAY).label(ui);
            }
            (self.item_fn)(item, context, ui)
        })
        .inner
    }
}

/// Composer implementation for immutable lists
impl<T: Clone, F> Composer<Vec<T>> for ListComposer<T, F>
where
    F: Fn(&T, &Context, &mut Ui) -> Response,
{
    fn data(&self) -> &Vec<T> {
        panic!("ListComposer does not hold data directly")
    }

    fn data_mut(&mut self) -> &mut Vec<T> {
        panic!("ListComposer does not hold data directly")
    }

    fn is_mutable(&self) -> bool {
        false
    }

    fn compose(self, _context: &Context, ui: &mut Ui) -> Response {
        ui.label("List composer requires external data handling")
    }
}
