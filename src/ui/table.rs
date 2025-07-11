use super::*;
use egui_extras::{Column, TableBuilder, TableRow};
use std::cmp::Ordering;

pub struct Table<'a, T> {
    row_getter: RowGetter<'a, T>,
    columns: Vec<TableColumn<'a, T>>,
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
        }
    }

    fn sort<T>(&mut self, table: &mut Table<T>, context: &Context, column_index: usize) {
        let ascending = match self.sorting {
            Some((idx, asc)) if idx == column_index => !asc,
            _ => false,
        };

        if let Some(column) = table.columns.get_mut(column_index) {
            if let Some(value_fn) = &mut column.value {
                self.indices.sort_by(|a, b| {
                    let item_a = table.row_getter.get(context, *a);
                    let item_b = table.row_getter.get(context, *b);

                    match (item_a, item_b) {
                        (Some(a), Some(b)) => {
                            let val_a = value_fn(context, a).unwrap_or_default();
                            let val_b = value_fn(context, b).unwrap_or_default();

                            match VarValue::compare(&val_a, &val_b) {
                                Ok(ord) => {
                                    if ascending {
                                        ord
                                    } else {
                                        ord.reverse()
                                    }
                                }
                                Err(_) => Ordering::Equal,
                            }
                        }
                        (Some(_), None) => Ordering::Less,
                        (None, Some(_)) => Ordering::Greater,
                        (None, None) => Ordering::Equal,
                    }
                });
            }
        }

        self.sorting = Some((column_index, ascending));
    }
}

impl<'a, T> Table<'a, T> {
    pub fn from_data(data: &'a Vec<T>) -> Self {
        Self {
            row_getter: RowGetter::Data(data),
            columns: Vec::new(),
        }
    }

    pub fn from_fn_row(
        len: usize,
        getter: impl Fn(&Context, usize) -> Option<&'a T> + Send + Sync + 'a,
    ) -> Self {
        Self {
            row_getter: RowGetter::FnRow(len, Box::new(getter)),
            columns: Vec::new(),
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
        let table_id = ui.id();
        let mut state = ui
            .ctx()
            .data(|r| r.get_temp::<TableState>(table_id))
            .unwrap_or_else(|| TableState::new(&self));
        if state.indices.len() != self.row_getter.len() {
            state = TableState::new(&self);
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

        table_builder
            .auto_shrink([false, true])
            .cell_layout(Layout::centered_and_justified(egui::Direction::TopDown))
            .header(24.0, |mut row| {
                let mut need_sort = None;
                for (column_index, column) in self.columns.iter().enumerate() {
                    row.col(|ui| {
                        let response = ui.button(&column.name);
                        if response.clicked() && column.value.is_some() {
                            need_sort = Some(column_index);
                        }
                        if let Some((sorted_column, ascending)) = state.sorting {
                            if sorted_column == column_index {
                                ui.label(if ascending { "↑" } else { "↓" });
                            }
                        }
                    });
                }
                if let Some(column_index) = need_sort {
                    state.sort(&mut self, context, column_index);
                }
            })
            .body(|mut body| {
                for _ in 0..state.indices.len() {
                    body.row(24.0, |mut row| {
                        self.show_row(context, &mut state, &mut row);
                    });
                }
            });

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
