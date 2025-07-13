use super::*;
use egui_extras::{Column, TableBuilder, TableRow};
use std::cmp::Ordering;

pub struct Table<'a, T> {
    row_getter: RowGetter<'a, T>,
    columns: Vec<TableColumn<'a, T>>,
    default_sort: Option<(usize, bool)>, // (column_index, ascending)
}

enum RowGetter<'a, T> {
    Data(&'a Vec<T>),
    FnRow(
        usize,
        Box<dyn Fn(&Context, usize) -> Option<&'a T> + 'a + Send + Sync>,
    ),
}

#[derive(Clone)]
pub struct TableState {
    indices: Vec<usize>,
    sorting: Option<(usize, bool)>,
    filters: Vec<(usize, String)>, // (column_index, filter_text)
}

pub struct TableColumn<'a, T> {
    name: String,
    show: Box<
        dyn FnMut(&Context, &mut Ui, &T, VarValue) -> Result<(), ExpressionError>
            + 'a
            + Send
            + Sync,
    >,
    value: Option<
        Box<dyn FnMut(&Context, &T) -> Result<VarValue, ExpressionError> + 'a + Send + Sync>,
    >,
    initial_width: Option<f32>,
    remainder: bool,
}

impl<'a, T> RowGetter<'a, T> {
    fn len(&self) -> usize {
        match self {
            RowGetter::Data(vec) => vec.len(),
            RowGetter::FnRow(len, _) => *len,
        }
    }

    fn get(&self, context: &Context, index: usize) -> Option<&T> {
        match self {
            RowGetter::Data(vec) => vec.get(index),
            RowGetter::FnRow(_, getter) => getter(context, index),
        }
    }
}

impl TableState {
    fn new<T>(table: &Table<T>) -> Self {
        let indices = (0..table.row_getter.len()).collect();
        Self {
            indices,
            sorting: None,
            filters: Vec::new(),
        }
    }

    fn apply_sorting<T>(
        &mut self,
        table: &mut Table<T>,
        context: &Context,
        column_index: usize,
        ascending: bool,
    ) {
        if let Some(column) = table.columns.get_mut(column_index) {
            if let Some(value_fn) = &mut column.value {
                self.indices.sort_by(|a, b| {
                    let item_a = table.row_getter.get(context, *a);
                    let item_b = table.row_getter.get(context, *b);

                    match (item_a, item_b) {
                        (Some(data_a), Some(data_b)) => {
                            let val_a = value_fn(context, data_a).unwrap_or_default();
                            let val_b = value_fn(context, data_b).unwrap_or_default();

                            match VarValue::compare(&val_a, &val_b) {
                                Ok(ord) => {
                                    let primary_ord = if ascending { ord } else { ord.reverse() };

                                    // If primary values are equal, sort by index (stable sort)
                                    if primary_ord == Ordering::Equal {
                                        a.cmp(b)
                                    } else {
                                        primary_ord
                                    }
                                }
                                Err(_) => a.cmp(b), // Fall back to index comparison
                            }
                        }
                        (Some(_), None) => Ordering::Less,
                        (None, Some(_)) => Ordering::Greater,
                        (None, None) => a.cmp(b),
                    }
                });
            }
        }
        self.sorting = Some((column_index, ascending));
    }

    fn sort<T>(&mut self, table: &mut Table<T>, context: &Context, column_index: usize) {
        let ascending = match self.sorting {
            Some((idx, asc)) if idx == column_index => !asc,
            _ => false,
        };
        self.apply_sorting(table, context, column_index, ascending);
    }

    fn apply_filters<T>(&mut self, table: &mut Table<T>, context: &Context) {
        if self.filters.is_empty() {
            return;
        }

        self.indices = (0..table.row_getter.len())
            .filter(|&index| {
                if let Some(data) = table.row_getter.get(context, index) {
                    for (column_index, filter_text) in &self.filters {
                        if let Some(column) = table.columns.get_mut(*column_index) {
                            if let Some(value_fn) = &mut column.value {
                                if let Ok(value) = value_fn(context, data) {
                                    let value_str = match value {
                                        VarValue::String(s) => s,
                                        VarValue::i32(n) => n.to_string(),
                                        VarValue::f32(n) => n.to_string(),
                                        VarValue::u64(n) => n.to_string(),
                                        VarValue::bool(b) => b.to_string(),
                                        _ => continue,
                                    };
                                    if !value_str
                                        .to_lowercase()
                                        .contains(&filter_text.to_lowercase())
                                    {
                                        return false;
                                    }
                                }
                            }
                        }
                    }
                    true
                } else {
                    false
                }
            })
            .collect();
    }

    fn add_filter(&mut self, column_index: usize, filter_text: String) {
        // Remove existing filter for this column
        self.filters.retain(|(idx, _)| *idx != column_index);
        if !filter_text.is_empty() {
            self.filters.push((column_index, filter_text));
        }
    }

    fn remove_filter(&mut self, column_index: usize) {
        self.filters.retain(|(idx, _)| *idx != column_index);
    }

    fn clear_sorting(&mut self) {
        self.sorting = None;
    }
}

impl<'a, T> Table<'a, T> {
    pub fn from_data(data: &'a Vec<T>) -> Self {
        Self {
            row_getter: RowGetter::Data(data),
            columns: Vec::new(),
            default_sort: None,
        }
    }

    pub fn from_fn_row(
        len: usize,
        getter: impl Fn(&Context, usize) -> Option<&'a T> + Send + Sync + 'a,
    ) -> Self {
        Self {
            row_getter: RowGetter::FnRow(len, Box::new(getter)),
            columns: Vec::new(),
            default_sort: None,
        }
    }

    pub fn column_cstr(
        self,
        name: impl Into<String>,
        f: impl Fn(&Context, &T) -> Result<String, ExpressionError> + 'a + Send + Sync + Clone,
    ) -> Self {
        let f_clone = f.clone();
        self.column(
            name,
            move |_context, ui, _data, value| match value {
                VarValue::String(s) => {
                    ui.label(s);
                    Ok(())
                }
                _ => {
                    ui.label("Invalid");
                    Err(ExpressionErrorVariants::Custom("Type mismatch".to_string()).into())
                }
            },
            move |context, data| f_clone(context, data).map(VarValue::String),
        )
    }

    pub fn column(
        mut self,
        name: impl Into<String>,
        show_fn: impl FnMut(&Context, &mut Ui, &T, VarValue) -> Result<(), ExpressionError>
        + 'a
        + Send
        + Sync,
        value_fn: impl FnMut(&Context, &T) -> Result<VarValue, ExpressionError> + 'a + Send + Sync,
    ) -> Self {
        self.columns.push(TableColumn {
            name: name.into(),
            show: Box::new(show_fn),
            value: Some(Box::new(value_fn)),
            initial_width: None,
            remainder: false,
        });
        self
    }

    pub fn column_no_sort(
        mut self,
        name: impl Into<String>,
        show_fn: impl FnMut(&Context, &mut Ui, &T, VarValue) -> Result<(), ExpressionError>
        + 'a
        + Send
        + Sync,
    ) -> Self {
        self.columns.push(TableColumn {
            name: name.into(),
            show: Box::new(show_fn),
            value: None,
            initial_width: None,
            remainder: false,
        });
        self
    }

    pub fn column_initial_width(mut self, max_width: f32) -> Self {
        if let Some(last_column) = self.columns.last_mut() {
            last_column.initial_width = Some(max_width);
        }
        self
    }

    /// Makes the last added column take up all remaining available space.
    /// This is useful for columns that should expand to fill the table width.
    ///
    /// # Examples
    /// ```
    /// table.column("name", show_fn, value_fn)
    ///      .column_remainder(); // "name" column takes remaining space
    /// ```
    pub fn column_remainder(mut self) -> Self {
        if let Some(last_column) = self.columns.last_mut() {
            last_column.remainder = true;
        }
        self
    }

    /// Sets the default sorting for the table.
    /// The table will be sorted by the specified column on initial display.
    ///
    /// # Arguments
    /// * `column_index` - Index of the column to sort by (0-based)
    /// * `ascending` - True for ascending order, false for descending
    pub fn default_sort(mut self, column_index: usize, ascending: bool) -> Self {
        self.default_sort = Some((column_index, ascending));
        self
    }

    fn show_row(&mut self, context: &Context, state: &mut TableState, row: &mut TableRow) {
        let i = *state.indices.get(row.index()).unwrap();
        if let Some(data) = self.row_getter.get(context, i) {
            for column in self.columns.iter_mut() {
                row.col(|ui| {
                    ui.push_id(i, |ui| {
                        let value = if let Some(value_fn) = &mut column.value {
                            match value_fn(context, data) {
                                Ok(v) => v,
                                Err(e) => {
                                    e.ui(ui);
                                    default()
                                }
                            }
                        } else {
                            VarValue::default()
                        };
                        (column.show)(context, ui, data, value).ui(ui);
                    });
                });
            }
        }
    }

    pub fn ui(mut self, context: &Context, ui: &mut Ui) {
        let table_id = ui.id().with("table");
        let mut state = ui
            .ctx()
            .data(|r| r.get_temp::<TableState>(table_id))
            .unwrap_or_else(|| TableState::new(&self));

        let data_changed = state.indices.len() != self.row_getter.len();
        if data_changed {
            state.indices = (0..self.row_getter.len()).collect();
        }

        // Apply default sorting only if this is a new table state (no existing sorting)
        if let Some((column_index, ascending)) = self.default_sort {
            if state.sorting.is_none()
                && state.filters.is_empty()
                && column_index < self.columns.len()
            {
                state.apply_sorting(&mut self, context, column_index, ascending);
            }
        }

        // Display current sorting and filters above the table
        ui.horizontal(|ui| {
            let mut actions = Vec::new();

            // Show current sorting
            if let Some((column_index, ascending)) = state.sorting {
                if let Some(column) = self.columns.get(column_index) {
                    ui.label(format!(
                        "Sorted by: {} {}",
                        column.name,
                        if ascending { "↑" } else { "↓" }
                    ));
                    if ui.button("✕").clicked() {
                        actions.push(("clear_sort", column_index, String::new()));
                    }
                }
            }

            // Show current filters
            for (column_index, filter_text) in &state.filters {
                if let Some(column) = self.columns.get(*column_index) {
                    ui.label(format!("Filter {}: '{}'", column.name, filter_text));
                    if ui.button("✕").clicked() {
                        actions.push(("remove_filter", *column_index, String::new()));
                    }
                }
            }

            // Process actions
            for (action, column_index, _) in actions {
                match action {
                    "clear_sort" => {
                        state.clear_sorting();
                        // Rebuild indices and reapply filters
                        state.indices = (0..self.row_getter.len()).collect();
                        state.apply_filters(&mut self, context);
                    }
                    "remove_filter" => {
                        // Clear the filter text from UI data
                        let filter_id = ui.id().with("filter").with(column_index);
                        ui.data_mut(|data| {
                            data.remove::<String>(filter_id);
                        });
                        state.remove_filter(column_index);
                        // Rebuild indices and reapply remaining filters
                        state.indices = (0..self.row_getter.len()).collect();
                        state.apply_filters(&mut self, context);
                        // Reapply sorting if it exists
                        if let Some((sort_column, ascending)) = state.sorting {
                            state.apply_sorting(&mut self, context, sort_column, ascending);
                        }
                    }
                    _ => {}
                }
            }
        });

        // Apply filters to current data
        if data_changed || !state.filters.is_empty() {
            if data_changed {
                state.indices = (0..self.row_getter.len()).collect();
            }
            state.apply_filters(&mut self, context);

            // Reapply sorting after filtering
            if let Some((column_index, ascending)) = state.sorting {
                state.apply_sorting(&mut self, context, column_index, ascending);
            }
        }

        let mut table_builder = TableBuilder::new(ui);
        for column in &self.columns {
            let col = if column.remainder {
                Column::remainder()
            } else if let Some(initial_width) = column.initial_width {
                Column::initial(initial_width)
            } else {
                Column::auto()
            }
            .resizable(true);
            table_builder = table_builder.column(col);
        }

        // Collect filter updates and sort request outside the closure
        let mut filter_updates = Vec::new();
        let mut sort_request = None;

        table_builder
            .auto_shrink([false, true])
            .cell_layout(Layout::centered_and_justified(egui::Direction::TopDown))
            .header(24.0, |mut row| {
                for (column_index, column) in self.columns.iter().enumerate() {
                    row.col(|ui| {
                        ui.vertical(|ui| {
                            // Column header button
                            let response = ui.button(&column.name);
                            if response.clicked() && column.value.is_some() {
                                sort_request = Some(column_index);
                            }
                            if let Some((sorted_column, ascending)) = state.sorting {
                                if sorted_column == column_index {
                                    ui.label(if ascending { "↑" } else { "↓" });
                                }
                            }

                            // Filter input (only for columns with value function)
                            if column.value.is_some() {
                                let filter_id = ui.id().with("filter").with(column_index);
                                let mut filter_text = ui.data_mut(|data| {
                                    data.get_temp_mut_or_default::<String>(filter_id).clone()
                                });

                                let response = ui.text_edit_singleline(&mut filter_text);
                                if response.changed() {
                                    ui.data_mut(|data| {
                                        data.insert_temp(filter_id, filter_text.clone());
                                    });
                                    filter_updates.push((column_index, filter_text));
                                }
                            }
                        });
                    });
                }
            })
            .body(|mut body| {
                for _ in 0..state.indices.len() {
                    body.row(24.0, |mut row| {
                        self.show_row(context, &mut state, &mut row);
                    });
                }
            });

        // Process filter updates after the table is built
        for (column_index, filter_text) in filter_updates {
            state.add_filter(column_index, filter_text);
            // Rebuild indices and apply all filters
            state.indices = (0..self.row_getter.len()).collect();
            state.apply_filters(&mut self, context);
            // Reapply sorting if it exists
            if let Some((sort_column, ascending)) = state.sorting {
                state.apply_sorting(&mut self, context, sort_column, ascending);
            }
        }

        // Process sort request after the table is built
        if let Some(column_index) = sort_request {
            state.sort(&mut self, context, column_index);
        }

        ui.ctx().data_mut(|w| w.insert_temp(table_id, state));
    }
}

pub trait TableExt<T> {
    /// Create a table widget from this vector
    fn table(&self) -> Table<T>;
}

impl<T> TableExt<T> for Vec<T> {
    fn table(&self) -> Table<T> {
        Table::from_data(self)
    }
}
