use super::*;
use bevy_egui::egui::{Popup, PopupAnchor, PopupCloseBehavior};
use egui_extras::{Column, TableBuilder, TableRow};
use std::cmp::Ordering;

const CACHE_TTL_SECS: f64 = 2.0;

#[derive(Clone)]
struct CacheEntry {
    value: VarValue,
    timestamp: f64,
}

pub struct Table<'a, T> {
    row_getter: RowGetter<'a, T>,
    columns: Vec<TableColumn<'a, T>>,
    default_sort: Option<(usize, bool)>, // (column_index, ascending)
}

enum RowGetter<'a, T> {
    Data(&'a Vec<T>),
    FnRow(
        usize,
        Box<dyn Fn(&ClientContext, usize) -> Option<&'a T> + 'a + Send + Sync>,
    ),
}

#[derive(Clone)]
pub struct TableState {
    indices: Vec<usize>,
    sorting: Option<(usize, bool)>,
    filters: Vec<(usize, String, bool)>, // (column_index, filter_text, is_equals)
}

pub struct TableColumn<'a, T> {
    name: String,
    show:
        Box<dyn FnMut(&ClientContext, &mut Ui, &T, VarValue) -> NodeResult<()> + 'a + Send + Sync>,
    value: Option<
        Box<dyn FnMut(&ClientContext, &T) -> Result<VarValue, NodeError> + 'a + Send + Sync>,
    >,
    initial_width: Option<f32>,
    remainder: bool,
    on_hover_ui: Option<Box<dyn Fn(&mut Ui) + 'a + Send + Sync>>,
}

impl<'a, T> RowGetter<'a, T> {
    fn len(&self) -> usize {
        match self {
            RowGetter::Data(vec) => vec.len(),
            RowGetter::FnRow(len, _) => *len,
        }
    }

    fn get(&self, context: &ClientContext, index: usize) -> Option<&T> {
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
        context: &ClientContext,
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

                                    if primary_ord == Ordering::Equal {
                                        a.cmp(b)
                                    } else {
                                        primary_ord
                                    }
                                }
                                Err(_) => a.cmp(b),
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

    fn apply_filters<T>(&mut self, table: &mut Table<T>, context: &ClientContext) {
        if self.filters.is_empty() {
            return;
        }

        self.indices = (0..table.row_getter.len())
            .filter(|&index| {
                if let Some(data) = table.row_getter.get(context, index) {
                    for (column_index, filter_text, is_equals) in &self.filters {
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
                                    let matches = if *is_equals {
                                        value_str.to_lowercase() == filter_text.to_lowercase()
                                    } else {
                                        value_str
                                            .to_lowercase()
                                            .contains(&filter_text.to_lowercase())
                                    };
                                    if !matches {
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

    fn add_filter(&mut self, column_index: usize, filter_text: String, is_equals: bool) {
        self.filters.retain(|(idx, _, _)| *idx != column_index);
        if !filter_text.is_empty() {
            self.filters.push((column_index, filter_text, is_equals));
        }
    }

    fn remove_filter(&mut self, column_index: usize) {
        self.filters.retain(|(idx, _, _)| *idx != column_index);
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
        getter: impl Fn(&ClientContext, usize) -> Option<&'a T> + Send + Sync + 'a,
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
        f: impl Fn(&ClientContext, &T) -> Result<String, NodeError> + 'a + Send + Sync + Clone,
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
                    Err(NodeError::custom("Type mismatch").into())
                }
            },
            move |context, data| f_clone(context, data).map(VarValue::String),
        )
    }

    pub fn column(
        mut self,
        name: impl Into<String>,
        show_fn: impl FnMut(&ClientContext, &mut Ui, &T, VarValue) -> NodeResult<()> + 'a + Send + Sync,
        value_fn: impl FnMut(&ClientContext, &T) -> Result<VarValue, NodeError> + 'a + Send + Sync,
    ) -> Self {
        self.columns.push(TableColumn {
            name: name.into(),
            show: Box::new(show_fn),
            value: Some(Box::new(value_fn)),
            initial_width: None,
            remainder: false,
            on_hover_ui: None,
        });
        self
    }

    pub fn column_no_sort(
        mut self,
        name: impl Into<String>,
        show_fn: impl FnMut(&ClientContext, &mut Ui, &T, VarValue) -> NodeResult<()> + 'a + Send + Sync,
    ) -> Self {
        self.columns.push(TableColumn {
            name: name.into(),
            show: Box::new(show_fn),
            value: None,
            initial_width: None,
            remainder: false,
            on_hover_ui: None,
        });
        self
    }

    pub fn column_initial_width(mut self, max_width: f32) -> Self {
        if let Some(last_column) = self.columns.last_mut() {
            last_column.initial_width = Some(max_width);
        }
        self
    }

    pub fn column_remainder(mut self) -> Self {
        if let Some(last_column) = self.columns.last_mut() {
            last_column.remainder = true;
        }
        self
    }

    pub fn default_sort(mut self, column_index: usize, ascending: bool) -> Self {
        self.default_sort = Some((column_index, ascending));
        self
    }

    pub fn column_on_hover_ui(mut self, hover_fn: impl Fn(&mut Ui) + 'a + Send + Sync) -> Self {
        if let Some(last_column) = self.columns.last_mut() {
            last_column.on_hover_ui = Some(Box::new(hover_fn));
        }
        self
    }

    pub fn column_with_hover_text(
        self,
        name: impl Into<String>,
        hover_text: impl Into<String> + 'a,
        show_fn: impl FnMut(&ClientContext, &mut Ui, &T, VarValue) -> NodeResult<()> + 'a + Send + Sync,
        value_fn: impl FnMut(&ClientContext, &T) -> Result<VarValue, NodeError> + 'a + Send + Sync,
    ) -> Self {
        let hover_text = hover_text.into();
        self.column(name, show_fn, value_fn)
            .column_on_hover_ui(move |ui| {
                ui.label(&hover_text);
            })
    }

    fn show_row(&mut self, ctx: &ClientContext, state: &mut TableState, row: &mut TableRow) {
        let i = *state.indices.get(row.index()).unwrap();
        if let Some(data) = self.row_getter.get(ctx, i) {
            for (col_idx, column) in self.columns.iter_mut().enumerate() {
                row.col(|ui| {
                    ui.push_id(i, |ui| {
                        let value = if let Some(value_fn) = &mut column.value {
                            let table_id = ui.id().with("table");
                            let cache_id = table_id.with(col_idx).with(i);
                            let current_time = ui.ctx().input(|r| r.time);

                            let cached_value = ui.ctx().data(|d| {
                                d.get_temp::<CacheEntry>(cache_id).and_then(|entry| {
                                    if current_time - entry.timestamp < CACHE_TTL_SECS {
                                        Some(entry.value.clone())
                                    } else {
                                        None
                                    }
                                })
                            });

                            if let Some(v) = cached_value {
                                v
                            } else {
                                match value_fn(ctx, data) {
                                    Ok(v) => {
                                        ui.ctx().data_mut(|d| {
                                            d.insert_temp(
                                                cache_id,
                                                CacheEntry {
                                                    value: v.clone(),
                                                    timestamp: current_time,
                                                },
                                            );
                                        });
                                        v
                                    }
                                    Err(e) => {
                                        e.ui(ui);
                                        default()
                                    }
                                }
                            }
                        } else {
                            VarValue::default()
                        };
                        (column.show)(ctx, ui, data, value).ui(ui);
                    });
                });
            }
        }
    }

    pub fn ui(mut self, ctx: &ClientContext, ui: &mut Ui) {
        let table_id = ui.id().with("table");
        let mut state = ui
            .ctx()
            .data(|r| r.get_temp::<TableState>(table_id))
            .unwrap_or_else(|| TableState::new(&self));

        let data_changed = state.indices.len() != self.row_getter.len();
        if data_changed {
            state.indices = (0..self.row_getter.len()).collect();
        }

        if let Some((column_index, ascending)) = self.default_sort {
            if state.sorting.is_none()
                && state.filters.is_empty()
                && column_index < self.columns.len()
            {
                state.apply_sorting(&mut self, ctx, column_index, ascending);
            }
        }

        ui.horizontal(|ui| {
            if let Some((column_index, ascending)) = state.sorting {
                if let Some(column) = self.columns.get(column_index) {
                    ui.label(format!(
                        "Sorted by: {} {}",
                        column.name,
                        if ascending { "↑" } else { "↓" }
                    ));
                    if ui.button("❌").clicked() {
                        state.clear_sorting();
                        state.indices = (0..self.row_getter.len()).collect();
                        state.apply_filters(&mut self, ctx);
                    }
                }
            }

            let mut filters_to_remove = Vec::new();
            for (column_index, filter_text, is_equals) in &state.filters {
                if let Some(column) = self.columns.get(*column_index) {
                    let mode_str = if *is_equals { "equals" } else { "contains" };
                    ui.label(format!(
                        "Filter {}: '{}' ({})",
                        column.name, filter_text, mode_str
                    ));
                    if ui.button("❌").clicked() {
                        filters_to_remove.push(*column_index);
                    }
                }
            }

            for column_index in filters_to_remove {
                let filter_id = ui.id().with("filter").with(column_index);
                ui.data_mut(|data| {
                    data.remove::<String>(filter_id);
                });
                state.remove_filter(column_index);
                state.indices = (0..self.row_getter.len()).collect();
                state.apply_filters(&mut self, ctx);
                if let Some((sort_column, ascending)) = state.sorting {
                    state.apply_sorting(&mut self, ctx, sort_column, ascending);
                }
            }
        });

        if data_changed || !state.filters.is_empty() {
            if data_changed {
                state.indices = (0..self.row_getter.len()).collect();
            }
            state.apply_filters(&mut self, ctx);
            if let Some((column_index, ascending)) = state.sorting {
                state.apply_sorting(&mut self, ctx, column_index, ascending);
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

        let mut filter_updates: Vec<(usize, String, bool)> = Vec::new();
        let mut sort_change: Option<Option<(usize, bool)>> = None;

        table_builder
            .auto_shrink([false, true])
            .cell_layout(Layout::centered_and_justified(egui::Direction::TopDown))
            .header(24.0, |mut row| {
                for (column_index, column) in self.columns.iter().enumerate() {
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(&column.name);

                            if let Some(hover_fn) = &column.on_hover_ui {
                                ui.label(&column.name).on_hover_ui(|ui| {
                                    hover_fn(ui);
                                });
                            }

                            if column.value.is_some() {
                                let filter_button = ui.button("🔍");
                                let popup_id = ui.id().with("filter_popup").with(column_index);

                                let popup = Popup::new(
                                    popup_id,
                                    ui.ctx().clone(),
                                    PopupAnchor::from(&filter_button),
                                    ui.layer_id(),
                                )
                                .open_memory(None)
                                .close_behavior(PopupCloseBehavior::CloseOnClickOutside);

                                if filter_button.clicked() {
                                    Popup::toggle_id(ui.ctx(), popup_id);
                                }

                                popup.show(|ui| {
                                    ui.set_max_width(200.0);
                                    ui.label("Filter:");

                                    let filter_id = ui.id().with("filter").with(column_index);
                                    let is_equals_id = ui.id().with("is_equals").with(column_index);

                                    let mut filter_text = ui.data_mut(|data| {
                                        data.get_temp_mut_or_default::<String>(filter_id).clone()
                                    });

                                    let mut is_equals = ui.data_mut(|data| {
                                        data.get_temp_mut_or_default::<bool>(is_equals_id).clone()
                                    });

                                    let response = ui.text_edit_singleline(&mut filter_text);
                                    if response.changed() {
                                        ui.data_mut(|data| {
                                            data.insert_temp(filter_id, filter_text.clone());
                                        });
                                        filter_updates.push((
                                            column_index,
                                            filter_text.clone(),
                                            is_equals,
                                        ));
                                    }

                                    ui.separator();

                                    ui.horizontal(|ui| {
                                        if ui.radio(!is_equals, "Contains").clicked() {
                                            is_equals = false;
                                            ui.data_mut(|data| {
                                                data.insert_temp(is_equals_id, is_equals);
                                            });
                                            if !filter_text.is_empty() {
                                                filter_updates.push((
                                                    column_index,
                                                    filter_text.clone(),
                                                    is_equals,
                                                ));
                                            }
                                        }

                                        if ui.radio(is_equals, "Equals").clicked() {
                                            is_equals = true;
                                            ui.data_mut(|data| {
                                                data.insert_temp(is_equals_id, is_equals);
                                            });
                                            if !filter_text.is_empty() {
                                                filter_updates.push((
                                                    column_index,
                                                    filter_text.clone(),
                                                    is_equals,
                                                ));
                                            }
                                        }
                                    });

                                    if ui.button("Clear Filter").clicked() {
                                        ui.data_mut(|data| {
                                            data.insert_temp(filter_id, String::new());
                                        });
                                        filter_updates.push((
                                            column_index,
                                            String::new(),
                                            is_equals,
                                        ));
                                    }
                                });

                                let sort_icon = match state.sorting {
                                    Some((sorted_column, ascending))
                                        if sorted_column == column_index =>
                                    {
                                        if ascending {
                                            "↑"
                                        } else {
                                            "↓"
                                        }
                                    }
                                    _ => "-",
                                };

                                if ui.button(sort_icon).clicked() {
                                    sort_change = Some(match state.sorting {
                                        Some((sorted_column, ascending))
                                            if sorted_column == column_index =>
                                        {
                                            if ascending {
                                                Some((column_index, false))
                                            } else {
                                                None
                                            }
                                        }
                                        _ => Some((column_index, true)),
                                    });
                                }
                            }
                        });
                    });
                }
            })
            .body(|mut body| {
                for _ in 0..state.indices.len() {
                    body.row(24.0, |mut row| {
                        self.show_row(ctx, &mut state, &mut row);
                    });
                }
            });
        for (column_index, filter_text, is_equals) in filter_updates {
            state.add_filter(column_index, filter_text, is_equals);
            state.indices = (0..self.row_getter.len()).collect();
            state.apply_filters(&mut self, ctx);
            if let Some((sort_column, ascending)) = state.sorting {
                state.apply_sorting(&mut self, ctx, sort_column, ascending);
            }
        }

        if let Some(sort_opt) = sort_change {
            match sort_opt {
                Some((column_index, ascending)) => {
                    state.apply_sorting(&mut self, ctx, column_index, ascending);
                }
                None => {
                    state.clear_sorting();
                    state.indices = (0..self.row_getter.len()).collect();
                    state.apply_filters(&mut self, ctx);
                }
            }
        }

        ui.ctx().data_mut(|w| w.insert_temp(table_id, state));
    }
}

pub trait TableExt<T> {
    fn table<'a>(&'a self) -> Table<'a, T>;
}

impl<T> TableExt<T> for Vec<T> {
    fn table<'a>(&'a self) -> Table<'a, T> {
        Table::from_data(self)
    }
}
