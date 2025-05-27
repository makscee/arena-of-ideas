use egui_extras::{Column, TableBuilder, TableRow};

use super::*;

pub struct Table<'a, T> {
    row_getter: RowGetter<'a, T>,
    columns: Vec<TableColumn<T>>,
}

enum RowGetter<'a, T> {
    Owned(Vec<T>),
    FnRow(
        usize,
        Box<dyn Fn(&Context, usize) -> Option<&'a T> + Send + Sync>,
    ),
}

#[derive(Clone)]
pub struct TableState {
    indices: Vec<usize>,
    sorting: Option<(usize, bool)>,
}

pub struct TableColumn<T> {
    name: String,
    show: Box<dyn Fn(&Context, &mut Ui, &T)>,
    value: Option<Box<dyn Fn(&Context, &T) -> VarValue>>,
}

impl<'a, T> RowGetter<'a, T> {
    fn len(&self) -> usize {
        match self {
            RowGetter::Owned(vec) => vec.len(),
            RowGetter::FnRow(len, _) => *len,
        }
    }

    fn get(&self, context: &Context, index: usize) -> Option<&T> {
        match self {
            RowGetter::Owned(vec) => vec.get(index),
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
}

impl<'a, T> Table<'a, T> {
    pub fn from_owned(data: Vec<T>) -> Self {
        Self {
            row_getter: RowGetter::Owned(data),
            columns: Vec::new(),
        }
    }
    pub fn from_fn_row(
        len: usize,
        getter: impl Fn(&Context, usize) -> Option<&'a T> + Send + Sync + 'static,
    ) -> Self {
        Self {
            row_getter: RowGetter::FnRow(len, Box::new(getter)),
            columns: Vec::new(),
        }
    }
    pub fn column_cstr(
        self,
        name: impl Into<String>,
        f: impl Fn(&Context, &T) -> String + 'static + Send + Sync,
    ) -> Self {
        let mut table = self;
        table.columns.push(TableColumn {
            name: name.into(),
            show: Box::new(move |context, ui, data| {
                ui.label(f(context, data));
            }),
            value: None,
        });
        table
    }
    fn show_row(&self, context: &Context, state: &mut TableState, row: &mut TableRow) {
        let i = *state.indices.get(row.index()).unwrap();
        let data = self.row_getter.get(context, i).unwrap();
        for column in self.columns.iter() {
            row.col(|ui| {
                (column.show)(context, ui, data);
            });
        }
    }
    pub fn ui(self, context: &Context, ui: &mut Ui) {
        let mut state = ui
            .ctx()
            .data(|r| r.get_temp::<TableState>(ui.id()))
            .unwrap_or_else(|| TableState::new(&self));
        TableBuilder::new(ui)
            .columns(Column::auto(), self.columns.len())
            .cell_layout(Layout::centered_and_justified(egui::Direction::TopDown))
            .header(24.0, |mut row| {
                for column in self.columns.iter() {
                    row.col(|ui| {
                        ui.label(&column.name);
                    });
                }
            })
            .body(|body| {
                body.rows(24.0, state.indices.len(), |mut row| {
                    self.show_row(context, &mut state, &mut row);
                });
            });
    }
}
