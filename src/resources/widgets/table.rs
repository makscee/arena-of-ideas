use egui_extras::{Column, TableBuilder};
use ordered_hash_map::OrderedHashMap;

use super::*;

#[derive(Clone)]
pub struct Table<T> {
    name: &'static str,
    columns: OrderedHashMap<&'static str, TableColumn<T>>,
    data: Vec<T>,
    heights: Vec<f32>,
    hovered_row: Option<usize>,
    selected_row: Option<usize>,
    cached: bool,
    sorting: Option<(usize, Sorting)>,
}

#[derive(Clone)]
pub struct TableColumn<T> {
    show: fn(&T, &Self, &mut Ui, &mut World) -> Response,
    value: Option<fn(&T) -> VarValue>,
    width: f32,
    sortable: bool,
}

#[derive(Clone, PartialEq, Eq, Copy)]
enum Sorting {
    Asc,
    Desc,
}

impl<T> TableColumn<T> {
    pub fn show_fn(mut self, f: fn(&T, &Self, &mut Ui, &mut World) -> Response) -> Self {
        self.show = f;
        self
    }
    pub fn value_fn(mut self, f: fn(&T) -> VarValue) -> Self {
        self.value = Some(f);
        self.sortable = true;
        self
    }
    pub fn no_sort(mut self) -> Self {
        self.sortable = false;
        self
    }
}

pub fn column_value<T>(value: fn(&T) -> VarValue) -> TableColumn<T> {
    TableColumn {
        show: |v, s, ui, _| (s.value.unwrap())(v).cstr().label(ui),
        value: Some(value),
        width: 0.0,
        sortable: true,
    }
}
pub fn column_show<T>(
    show: fn(&T, &TableColumn<T>, &mut Ui, world: &mut World) -> Response,
) -> TableColumn<T> {
    TableColumn {
        show,
        value: None,
        width: 0.0,
        sortable: false,
    }
}

impl<T: 'static + Clone + Send + Sync> Table<T> {
    pub fn new(name: &'static str, data: Vec<T>) -> Self {
        Self {
            name,
            columns: default(),
            heights: vec![0.0; data.len()],
            data,
            cached: false,
            hovered_row: None,
            selected_row: None,
            sorting: None,
        }
    }
    pub fn new_cached_refreshed(
        name: &'static str,
        refresh: bool,
        data: impl Fn() -> Vec<T>,
        ctx: &egui::Context,
    ) -> Self {
        let id = Id::new(name);
        let cache = if refresh {
            None
        } else {
            ctx.data_mut(|w| w.get_temp::<Table<T>>(id))
        };
        let table = if let Some(table) = cache {
            table
        } else {
            let data = data();
            let mut table = Self::new(name, data);
            table.cached = true;
            table
        };
        table
    }
    pub fn new_cached(name: &'static str, data: impl Fn() -> Vec<T>, ctx: &egui::Context) -> Self {
        Self::new_cached_refreshed(name, false, data, ctx)
    }
    pub fn ui(&mut self, ui: &mut Ui, world: &mut World) {
        let mut need_sort = false;

        TableBuilder::new(ui)
            .striped(true)
            .columns(Column::auto(), self.columns.len())
            .header(30.0, |mut row| {
                for (i, (name, column)) in self.columns.iter().enumerate() {
                    row.col(|ui| {
                        let resp = if column.sortable {
                            let mut btn = Button::click(name.to_string());
                            btn = if self
                                .sorting
                                .as_ref()
                                .is_some_and(|(i_sort, _)| *i_sort == i)
                            {
                                btn.bg(ui)
                            } else {
                                btn
                            };
                            btn.ui(ui)
                        } else {
                            Button::click(name.to_string())
                                .enabled(false)
                                .gray(ui)
                                .ui(ui)
                        };
                        if resp.clicked() {
                            if self
                                .sorting
                                .is_some_and(|(s_i, s)| s_i == i && s == Sorting::Desc)
                            {
                                self.sorting = Some((i, Sorting::Asc));
                            } else {
                                self.sorting = Some((i, Sorting::Desc));
                            }
                            need_sort = true;
                        }
                    });
                }
            })
            .body(|body| {
                body.rows(30.0, self.data.len(), |mut row| {
                    let row_i = row.index();
                    for (_, col) in self.columns.iter() {
                        row.col(|ui| {
                            let data = &self.data[row_i];
                            (col.show)(data, col, ui, world);
                        });
                    }
                })
            });

        if self.cached {
            if need_sort {
                let Some((i, sort)) = self.sorting else {
                    panic!("No sorting data")
                };
                let asc = sort == Sorting::Asc;
                let value = self.columns.values().nth(i).unwrap().value.unwrap();
                self.data.sort_by(|a, b| {
                    let a = (value)(a);
                    let b = (value)(b);
                    if asc {
                        VarValue::compare(&a, &b).unwrap()
                    } else {
                        VarValue::compare(&b, &a).unwrap()
                    }
                });
            }
            let id = Id::new(self.name);
            ui.ctx().data_mut(|w| w.insert_temp(id, self.clone()));
        }
    }
    pub fn column(mut self, name: &'static str, column: TableColumn<T>) -> Self {
        if !self.columns.contains_key(name) {
            self.columns.insert(name, column);
        }
        self
    }
}
