use core::f32;

use bevy::utils::hashbrown::HashMap;
use egui::NumExt;
use egui_extras::{Column, TableBuilder, TableRow};

use super::*;

#[must_use]
pub struct Table<'a, T> {
    name: String,
    rows_getter: Box<dyn Fn(&mut World) -> Vec<T> + Send + 'a>,
    rows_saved: Option<Vec<T>>,
    columns: IndexMap<&'static str, TableColumn<T>>,
    row_height: f32,
    title: bool,
    selectable: bool,
    per_row_render: bool,
    filters: Vec<(&'static str, &'static str, VarValue)>,
}

#[derive(Resource)]
struct TableCacheResource<T> {
    map: HashMap<Id, TableCacheData<T>>,
}

struct TableCacheData<T> {
    data: Vec<T>,
    ts: f32,
}

impl<T> Default for TableCacheResource<T> {
    fn default() -> Self {
        Self { map: default() }
    }
}

#[derive(Default, Clone, Debug)]
pub struct TableState {
    cells: HashMap<(usize, usize), CellState>,
    filter: Option<usize>,
    sorting: Option<(usize, bool)>,
    indices: Vec<usize>,
    frame_nr: u64,
    pub selected_row: Option<usize>,
}

#[derive(Default, Clone, Debug)]
pub struct CellState {
    cache: VarValue,
    highlight: f32,
}
const CACHE_LIFETIME: f32 = 1.0;

pub struct TableColumn<T> {
    value: Box<dyn Fn(&T, &World) -> VarValue>,
    show: Box<dyn Fn(&T, VarValue, &mut Ui, &mut World)>,
    sortable: bool,
    hide_name: bool,
}

impl<T> TableColumn<T> {
    pub fn no_sort(mut self) -> Self {
        self.sortable = false;
        self
    }
    pub fn no_name(mut self) -> Self {
        self.hide_name = true;
        self.sortable = false;
        self
    }
}

impl CellState {
    fn get_cached<T>(
        &mut self,
        index: (usize, usize),
        data: &T,
        f: &Box<dyn Fn(&T, &World) -> VarValue>,
        world: &World,
    ) -> VarValue {
        let offset = (index.0 + index.1) as f32 * 0.05;
        if !gt().ticked(CACHE_LIFETIME, -offset) && self.cache != VarValue::default() {
            self.cache.clone()
        } else {
            let value = f(data, world);
            if !self.cache.eq(&value) {
                self.highlight = 1.0;
            }
            self.cache = value.clone();
            value
        }
    }
    fn update(&mut self) {
        self.highlight = (self.highlight - gt().last_delta()).at_least(0.0);
    }
}

impl TableState {
    pub fn reset_cache(ctx: &egui::Context) {
        ctx.data_mut(|w| w.remove_by_type::<Self>());
    }
    pub fn reset_rows_cache<T: Send + Sync + 'static>(world: &mut World) {
        world.remove_resource::<TableCacheResource<T>>();
    }
    pub fn reset_cache_op() {
        OperationsPlugin::add(|w| Self::reset_cache(&egui_context(w).unwrap()));
    }
}

impl<'a, T: 'static + Clone + Send + Sync> Table<'a, T> {
    pub fn new(name: impl ToString, rows: impl Fn(&mut World) -> Vec<T> + Send + 'a) -> Self {
        Self {
            name: name.to_string(),
            columns: default(),
            title: default(),
            selectable: default(),
            filters: default(),
            row_height: 22.0,
            rows_getter: Box::new(rows),
            rows_saved: None,
            per_row_render: false,
        }
    }
    pub fn new_persistent(name: impl ToString, rows: Vec<T>) -> Self {
        Self {
            name: name.to_string(),
            rows_getter: Box::new(|_| default()),
            rows_saved: Some(rows),
            columns: IndexMap::new(),
            row_height: 0.0,
            title: false,
            selectable: false,
            filters: Vec::new(),
            per_row_render: false,
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
    pub fn row_height(mut self, value: f32) -> Self {
        self.row_height = value;
        self
    }
    pub fn per_row_render(mut self) -> Self {
        self.per_row_render = true;
        self
    }
    pub fn column(
        self,
        name: &'static str,
        value: fn(&T, &World) -> VarValue,
        show: fn(&T, VarValue, &mut Ui, &mut World),
        sortable: bool,
    ) -> Self {
        self.column_dyn(name, value, show, sortable)
    }
    pub fn column_dyn(
        mut self,
        name: &'static str,
        value: impl Fn(&T, &World) -> VarValue + 'static,
        show: impl Fn(&T, VarValue, &mut Ui, &mut World) + 'static,
        sortable: bool,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(value),
                show: Box::new(show),
                sortable,
                hide_name: false,
            },
        );
        self
    }
    pub fn column_ui(
        self,
        name: &'static str,
        show: fn(&T, VarValue, &mut Ui, &mut World),
    ) -> Self {
        self.column_ui_dyn(name, show)
    }
    pub fn column_ui_dyn(
        mut self,
        name: &'static str,
        show: impl Fn(&T, VarValue, &mut Ui, &mut World) + 'static,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(|_, _| default()),
                show: Box::new(show),
                sortable: false,
                hide_name: false,
            },
        );
        self
    }
    pub fn column_btn_mod_dyn(
        mut self,
        name: &'static str,
        on_click: impl Fn(&T, &mut Ui, &mut World) + 'static,
        modify: impl Fn(&T, &mut Ui, Button) -> Button + 'static,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(|_, _| name.to_string().into()),
                show: Box::new(move |d, _, ui, w| {
                    if modify(d, ui, Button::new(name.to_string()))
                        .ui(ui)
                        .clicked()
                    {
                        on_click(d, ui, w);
                    }
                }),
                sortable: false,
                hide_name: true,
            },
        );
        self
    }
    pub fn column_btn_mod(
        self,
        name: &'static str,
        on_click: fn(&T, &mut Ui, &mut World),
        modify: fn(&T, &mut Ui, Button) -> Button,
    ) -> Self {
        self.column_btn_mod_dyn(name, Box::new(on_click), Box::new(modify))
    }
    pub fn column_btn_dyn(
        self,
        name: &'static str,
        on_click: impl Fn(&T, &mut Ui, &mut World) + 'static,
    ) -> Self {
        self.column_btn_mod_dyn(name, on_click, |_, _, b| b)
    }
    pub fn column_btn(self, name: &'static str, on_click: fn(&T, &mut Ui, &mut World)) -> Self {
        self.column_btn_dyn(name, Box::new(on_click))
    }
    pub fn column_cstr_dyn(
        mut self,
        name: &'static str,
        s: impl Fn(&T, &World) -> Cstr + 'static,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, w| s(d, w).into()),
                show: Box::new(|_, v, ui, _| {
                    v.get_string().unwrap().label(ui);
                }),
                sortable: true,
                hide_name: false,
            },
        );
        self
    }
    pub fn column_cstr(self, name: &'static str, s: fn(&T, &World) -> Cstr) -> Self {
        self.column_cstr_dyn(name, s)
    }
    pub fn column_cstr_opt_dyn(
        self,
        name: &'static str,
        s: impl Fn(&T, &World) -> Option<Cstr> + 'static,
    ) -> Self {
        self.column_cstr_dyn(name, move |d, w| {
            s(d, w).unwrap_or("[red [s not found]]".into())
        })
    }
    pub fn column_cstr_value_dyn(
        mut self,
        name: &'static str,
        v: impl Fn(&T, &World) -> VarValue + 'static,
        s: impl Fn(&T, VarValue) -> Cstr + 'static,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, w| v(d, w).into()),
                show: Box::new(move |d, v, ui, _| {
                    s(d, v).label(ui);
                }),
                sortable: true,
                hide_name: false,
            },
        );
        self
    }
    pub fn column_cstr_value(
        self,
        name: &'static str,
        v: fn(&T, &World) -> VarValue,
        s: fn(&T, VarValue) -> Cstr,
    ) -> Self {
        self.column_cstr_value_dyn(name, v, s)
    }
    pub fn column_cstr_click_dyn(
        mut self,
        name: &'static str,
        v: impl Fn(&T, &World) -> Cstr + 'static,
        on_click: impl Fn(&T, &mut World) + 'static,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, w| v(d, w).into()),
                show: Box::new(move |d, v, ui, w| {
                    if v.get_string().unwrap().clone().button(ui).clicked() {
                        on_click(d, w);
                    }
                }),
                sortable: false,
                hide_name: true,
            },
        );
        self
    }
    pub fn column_int_dyn(
        mut self,
        name: &'static str,
        value: impl Fn(&T) -> i32 + 'static,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, _| value(d).into()),
                show: Box::new(|_, v, ui, _| {
                    v.cstr().label(ui);
                }),
                sortable: true,
                hide_name: false,
            },
        );
        self
    }
    pub fn column_int(self, name: &'static str, value: fn(&T) -> i32) -> Self {
        self.column_int_dyn(name, value)
    }
    pub fn column_float_dyn(
        mut self,
        name: &'static str,
        value: impl Fn(&T) -> f32 + 'static,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, _| value(d).into()),
                show: Box::new(|_, v, ui, _| {
                    v.cstr().label(ui);
                }),
                sortable: true,
                hide_name: false,
            },
        );
        self
    }
    pub fn column_float(self, name: &'static str, value: fn(&T) -> f32) -> Self {
        self.column_float_dyn(name, value)
    }
    pub fn column_id_dyn(
        mut self,
        name: &'static str,
        value: impl Fn(&T) -> u64 + 'static,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, _| value(d).into()),
                show: Box::new(|_, v, ui, _| {
                    v.cstr().label(ui);
                }),
                sortable: true,
                hide_name: false,
            },
        );
        self
    }
    pub fn column_id(self, name: &'static str, value: fn(&T) -> u64) -> Self {
        self.column_id_dyn(name, value)
    }
    pub fn column_ts_dyn(
        mut self,
        name: &'static str,
        value: impl Fn(&T) -> u64 + 'static,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, _| value(d).into()),
                show: Box::new(|_, v, ui, _| {
                    format_timestamp(v.get_u64().unwrap_or_default())
                        .cstr_cs(VISIBLE_DARK, CstrStyle::Small)
                        .label(ui);
                }),
                sortable: true,
                hide_name: false,
            },
        );
        self
    }
    pub fn column_ts(self, name: &'static str, value: fn(&T) -> u64) -> Self {
        self.column_ts_dyn(name, value)
    }
    fn cache_rows(&self, id: Id, world: &mut World) {
        let mut need_update = false;
        world.init_resource::<TableCacheResource<T>>();
        world.resource_scope(|world, mut r: Mut<TableCacheResource<T>>| {
            if let Some(cache) = r.map.get(&id) {
                if cache.ts < gt().play_head() - CACHE_LIFETIME {
                    need_update = true;
                }
            } else {
                need_update = true;
            }
            if need_update {
                if let Some(rows) = self.rows_saved.clone() {
                    r.map.insert(
                        id,
                        TableCacheData {
                            data: rows,
                            ts: f32::MAX,
                        },
                    );
                } else {
                    r.map.insert(
                        id,
                        TableCacheData {
                            data: (self.rows_getter)(world),
                            ts: gt().play_head(),
                        },
                    );
                }
            }
        })
    }
    fn show_row(
        &self,
        row: &mut TableRow,
        data: &Vec<T>,
        state: &mut TableState,
        world: &mut World,
    ) {
        let mut row_i = row.index();
        if let Some(i) = state.indices.get(row_i) {
            row_i = *i;
        }
        for (col_i, (_, col)) in self.columns.iter().enumerate() {
            let index = (col_i, row_i);
            let cell = state.cells.entry(index).or_default();
            cell.update();
            row.set_selected(state.selected_row.is_some_and(|i| i == row_i));
            row.col(|ui| {
                let d = &data[row_i];
                let v: VarValue = cell.get_cached(index, d, &col.value, world);
                (col.show)(d, v, ui, world);
                if cell.highlight > 0.0 {
                    ui.painter().rect_stroke(
                        ui.min_rect(),
                        CornerRadius::same(13),
                        Stroke::new(1.0, YELLOW.gamma_multiply(cell.highlight)),
                        egui::StrokeKind::Middle,
                    );
                }
            });
        }
        if self.selectable {
            row.col(|ui| {
                if "select".cstr_c(VISIBLE_BRIGHT).button(ui).clicked() {
                    state.selected_row = Some(row_i);
                }
            });
        }
    }
    pub fn ui(self, ui: &mut Ui, world: &mut World) -> TableState {
        let mut need_sort = false;
        let mut need_filter = false;
        let id = Id::new("table_").with(&self.name).with(ui.id());
        self.cache_rows(id, world);
        world.resource_scope(|world, rows: Mut<TableCacheResource<T>>| {
            let data = &rows.map.get(&id).unwrap().data;
            let mut state = ui
                .ctx()
                .data_mut(|w| w.get_temp::<TableState>(id))
                .unwrap_or_default();
            let frame_nr = ui.ctx().cumulative_pass_nr();
            if state.frame_nr + 1 != frame_nr {
                state = default();
            }
            state.frame_nr = frame_nr;

            if state.indices.len() != data.len() && state.filter.is_none() {
                state.indices = (0..data.len()).collect_vec();
            }
            if self.title {
                title(&self.name, ui);
            }
            if !self.filters.is_empty() {
                ui.horizontal(|ui| {
                    for (i, (name, _, _)) in self.filters.iter().enumerate() {
                        let active = state.filter.is_some_and(|f| f == i);
                        if Button::new(name.to_string())
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

            Frame::new().inner_margin(Margin::same(13)).show(ui, |ui| {
                ui.push_id(Id::new(&self.name), |ui| {
                    ui.horizontal(|ui| {
                        format!("total: {}", state.indices.len())
                            .cstr_c(VISIBLE_DARK)
                            .label(ui);
                    });
                    TableBuilder::new(ui)
                        .auto_shrink([false, true])
                        .columns(
                            Column::auto(),
                            self.columns.len() + self.selectable as usize,
                        )
                        .cell_layout(Layout::centered_and_justified(egui::Direction::TopDown))
                        .header(30.0, |mut row| {
                            for (i, (name, column)) in self.columns.iter().enumerate() {
                                row.col(|ui| {
                                    let clicked = if column.sortable {
                                        let mut btn = Button::new(name.to_string());
                                        btn = if state
                                            .sorting
                                            .as_ref()
                                            .is_some_and(|(i_sort, _)| *i_sort == i)
                                        {
                                            btn.bg(ui)
                                        } else {
                                            btn
                                        };
                                        btn.ui(ui).clicked()
                                    } else if column.hide_name {
                                        false
                                    } else {
                                        Button::new(name.to_string())
                                            .enabled(false)
                                            .gray(ui)
                                            .ui(ui);
                                        false
                                    };
                                    if clicked {
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
                        .body(|mut body| {
                            if self.per_row_render {
                                for _ in 0..state.indices.len() {
                                    body.row(self.row_height, |mut row| {
                                        self.show_row(&mut row, data, &mut state, world);
                                    });
                                }
                            } else {
                                body.rows(self.row_height, state.indices.len(), |mut row| {
                                    self.show_row(&mut row, data, &mut state, world);
                                });
                            }
                        });
                });
            });
            if need_filter {
                state.indices = (0..data.len()).collect_vec();
                state.sorting = None;
                if let Some(filter) = state.filter {
                    let (_, col, filter) = &self.filters[filter];
                    let col = self.columns.get(col).unwrap();
                    state
                        .indices
                        .retain(|v| (col.value)(&data[*v], world).eq(filter));
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
                    let a = (value)(&data[*a], world);
                    let b = (value)(&data[*b], world);
                    if desc {
                        VarValue::compare(&b, &a).unwrap()
                    } else {
                        VarValue::compare(&a, &b).unwrap()
                    }
                });
            }
            ui.ctx().data_mut(|w| w.insert_temp(id, state.clone()));
            state
        })
    }
}
