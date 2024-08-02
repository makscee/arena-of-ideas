use egui_extras::{Column, TableBuilder};
use ordered_hash_map::OrderedHashMap;

use super::*;

pub struct Table<T> {
    name: &'static str,
    columns: OrderedHashMap<&'static str, TableColumn<T>>,
    title: bool,
    selectable: bool,
    filters: Vec<(&'static str, &'static str, VarValue)>,
}

#[derive(Default, Clone, Debug)]
pub struct TableState {
    filter: Option<usize>,
    sorting: Option<(usize, bool)>,
    indices: Vec<usize>,
    pub selected_row: Option<usize>,
}

pub struct TableColumn<T> {
    value: Box<dyn Fn(&T) -> VarValue>,
    show: Box<dyn Fn(&T, VarValue, &mut Ui, &mut World) -> Response>,
    sortable: bool,
}

impl<T> TableColumn<T> {
    pub fn no_sort(mut self) -> Self {
        self.sortable = false;
        self
    }
}

impl<T: 'static + Clone + Send + Sync> Table<T> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            columns: default(),
            title: default(),
            selectable: default(),
            filters: default(),
        }
    }
    pub fn title(mut self) -> Self {
        self.title = true;
        self
    }
    pub fn selectable(mut self) -> Self {
        self.selectable = true;
        self
    }
    pub fn filter(mut self, name: &'static str, column: &'static str, value: VarValue) -> Self {
        self.filters.push((name, column, value));
        self
    }
    pub fn column(
        mut self,
        name: &'static str,
        value: fn(&T) -> VarValue,
        show: fn(&T, VarValue, &mut Ui, &mut World) -> Response,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                show: Box::new(show),
                value: Box::new(value),
                sortable: true,
            },
        );
        self
    }
    pub fn column_btn(mut self, name: &'static str, on_click: fn(&T, &mut Ui, &mut World)) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(|_| name.to_string().into()),
                show: Box::new(move |d, _, ui, w| {
                    let r = Button::click(name.to_string()).ui(ui);
                    if r.clicked() {
                        on_click(d, ui, w);
                    }
                    r
                }),
                sortable: false,
            },
        );
        self
    }
    pub fn column_cstr(mut self, name: &'static str, value: fn(&T) -> Cstr) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d| value(d).into()),
                show: Box::new(|_, v, ui, _| v.get_cstr().unwrap().label(ui)),
                sortable: true,
            },
        );
        self
    }
    pub fn column_cstr_click(
        mut self,
        name: &'static str,
        value: fn(&T) -> Cstr,
        on_click: fn(Cstr, &mut World),
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d| value(d).into()),
                show: Box::new(move |_, v, ui, w| {
                    let r = v.get_cstr().unwrap().button(ui);
                    if r.clicked() {
                        on_click(v.get_cstr().unwrap(), w);
                    }
                    r
                }),
                sortable: true,
            },
        );
        self
    }
    pub fn column_int(mut self, name: &'static str, value: fn(&T) -> i32) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d| value(d).into()),
                show: Box::new(|_, v, ui, _| v.cstr().label(ui)),
                sortable: true,
            },
        );
        self
    }
    pub fn column_gid(mut self, name: &'static str, value: fn(&T) -> GID) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d| value(d).into()),
                show: Box::new(|_, v, ui, _| v.cstr().label(ui)),
                sortable: true,
            },
        );
        self
    }
    pub fn column_ts(mut self, name: &'static str, value: fn(&T) -> u64) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d| value(d).into()),
                show: Box::new(|_, v, ui, _| {
                    format_timestamp(v.get_gid().unwrap())
                        .cstr_cs(VISIBLE_DARK, CstrStyle::Small)
                        .label(ui)
                }),
                sortable: true,
            },
        );
        self
    }
    pub fn column_user_click(
        mut self,
        name: &'static str,
        gid: fn(&T) -> GID,
        on_click: fn(GID, &mut Ui, &mut World),
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d| gid(d).into()),
                show: Box::new(move |_, v, ui, w| {
                    let gid = v.get_gid().unwrap();
                    if gid == 0 {
                        return "...".cstr().label(ui);
                    } else {
                        let r = gid.get_user().cstr().button(ui);
                        if r.clicked() {
                            on_click(gid, ui, w);
                        }
                        r
                    }
                }),
                sortable: true,
            },
        );
        self
    }
    pub fn ui(&mut self, data: &Vec<T>, ui: &mut Ui, world: &mut World) -> TableState {
        let mut need_sort = false;
        let mut need_filter = false;
        let id = Id::new("table_").with(self.name).with(ui.id());
        let mut state = ui
            .ctx()
            .data_mut(|w| w.get_temp::<TableState>(id))
            .unwrap_or_default();
        if state.indices.is_empty() && state.filter.is_none() {
            state.indices = (0..data.len()).collect_vec();
        }

        if self.title {
            title(self.name, ui);
        }
        if !self.filters.is_empty() {
            ui.horizontal(|ui| {
                for (i, (name, _, _)) in self.filters.iter().enumerate() {
                    let active = state.filter.is_some_and(|f| f == i);
                    if Button::click(name.to_string())
                        .min_width(100.0)
                        .active(active)
                        .ui(ui)
                        .clicked()
                    {
                        need_filter = true;
                        if active {
                            state.filter = None;
                        } else {
                            state.filter = Some(i);
                        }
                    }
                }
            });
        }
        TableBuilder::new(ui)
            .columns(
                Column::initial(5.0),
                self.columns.len() + self.selectable as usize,
            )
            .cell_layout(Layout::centered_and_justified(egui::Direction::TopDown))
            .header(30.0, |mut row| {
                for (i, (name, column)) in self.columns.iter().enumerate() {
                    row.col(|ui| {
                        let resp = if column.sortable {
                            let mut btn = Button::click(name.to_string());
                            btn = if state
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
                            if state.sorting.is_some_and(|(s_i, s)| s_i == i && !s) {
                                state.sorting = Some((i, true));
                            } else {
                                state.sorting = Some((i, false));
                            }
                            need_sort = true;
                        }
                    });
                }
            })
            .body(|body| {
                body.rows(22.0, state.indices.len(), |mut row| {
                    let mut row_i = row.index();
                    if let Some(i) = state.indices.get(row_i) {
                        row_i = *i;
                    }
                    row.set_selected(state.selected_row.is_some_and(|i| i == row_i));
                    for (_, col) in self.columns.iter() {
                        row.col(|ui| {
                            let d = &data[row_i];
                            let v = (col.value)(d);
                            (col.show)(d, v, ui, world);
                        });
                    }
                    if self.selectable {
                        row.col(|ui| {
                            if "select".cstr().button(ui).clicked() {
                                state.selected_row = Some(row_i);
                            }
                        });
                    }
                })
            });
        ui.horizontal(|ui| {
            format!("total: {}", data.len()).cstr().label(ui);
        });
        if need_filter {
            state.indices = (0..data.len()).collect_vec();
            state.sorting = None;
            if let Some(filter) = state.filter {
                let (_, col, filter) = &self.filters[filter];
                let col = self.columns.get(col).unwrap();
                state.indices.retain(|v| (col.value)(&data[*v]).eq(filter));
            }
        }
        if need_sort {
            let Some((i, desc)) = state.sorting else {
                panic!("No sorting data")
            };
            let value = &self.columns.values().nth(i).unwrap().value;
            if state.indices.is_empty() {
                state.indices = (0..data.len()).collect_vec();
            }
            state.indices.sort_by(|a, b| {
                let a = (value)(&data[*a]);
                let b = (value)(&data[*b]);
                if desc {
                    VarValue::compare(&b, &a).unwrap()
                } else {
                    VarValue::compare(&a, &b).unwrap()
                }
            });
        }
        ui.ctx().data_mut(|w| w.insert_temp(id, state.clone()));
        state
    }
}
