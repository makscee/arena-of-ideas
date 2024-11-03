use egui_extras::{Column, TableBuilder};

use super::*;

pub struct Table<T> {
    name: &'static str,
    columns: IndexMap<&'static str, TableColumn<T>>,
    title: bool,
    selectable: bool,
    filters: Vec<(&'static str, &'static str, VarValue)>,
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
        if !gt().ticked(CACHE_LIFETIME, -offset) {
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
    pub fn reset_cache_op() {
        OperationsPlugin::add(|w| Self::reset_cache(&egui_context(w).unwrap()));
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
        value: fn(&T, &World) -> VarValue,
        show: fn(&T, VarValue, &mut Ui, &mut World),
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
    pub fn column_dyn(
        mut self,
        name: &'static str,
        value: Box<dyn Fn(&T, &World) -> VarValue>,
        show: Box<dyn Fn(&T, VarValue, &mut Ui, &mut World)>,
        sortable: bool,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value,
                show,
                sortable,
                hide_name: false,
            },
        );
        self
    }
    pub fn column_btn_mod_dyn(
        mut self,
        name: &'static str,
        on_click: Box<dyn Fn(&T, &mut Ui, &mut World)>,
        modify: Box<dyn Fn(&T, &mut Ui, Button) -> Button>,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(|_, _| name.to_string().into()),
                show: Box::new(move |d, _, ui, w| {
                    if modify(d, ui, Button::click(name.to_string()))
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
        on_click: Box<dyn Fn(&T, &mut Ui, &mut World)>,
    ) -> Self {
        self.column_btn_mod_dyn(name, on_click, Box::new(|_, _, b| b))
    }
    pub fn column_btn(self, name: &'static str, on_click: fn(&T, &mut Ui, &mut World)) -> Self {
        self.column_btn_dyn(name, Box::new(on_click))
    }
    pub fn column_cstr_dyn(
        mut self,
        name: &'static str,
        s: Box<dyn Fn(&T, &World) -> Cstr>,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, w| s(d, w).into()),
                show: Box::new(|_, v, ui, _| {
                    v.get_cstr().unwrap().label(ui);
                }),
                sortable: true,
                hide_name: false,
            },
        );
        self
    }
    pub fn column_cstr(self, name: &'static str, s: fn(&T, &World) -> Cstr) -> Self {
        self.column_cstr_dyn(name, Box::new(s))
    }
    pub fn column_cstr_value(
        mut self,
        name: &'static str,
        v: fn(&T) -> VarValue,
        s: fn(&T, VarValue) -> Cstr,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, _| v(d).into()),
                show: Box::new(move |d, v, ui, _| {
                    s(d, v).label(ui);
                }),
                sortable: true,
                hide_name: false,
            },
        );
        self
    }
    pub fn column_cstr_click(
        mut self,
        name: &'static str,
        v: fn(&T, &World) -> Cstr,
        on_click: fn(&T, &mut World),
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, w| v(d, w).into()),
                show: Box::new(move |d, v, ui, w| {
                    if v.get_cstr().unwrap().clone().button(ui).clicked() {
                        on_click(d, w);
                    }
                }),
                sortable: false,
                hide_name: true,
            },
        );
        self
    }
    pub fn column_int_dyn(mut self, name: &'static str, value: Box<dyn Fn(&T) -> i32>) -> Self {
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
        self.column_int_dyn(name, Box::new(value))
    }
    pub fn column_float_dyn(mut self, name: &'static str, value: Box<dyn Fn(&T) -> f32>) -> Self {
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
        self.column_float_dyn(name, Box::new(value))
    }
    pub fn column_gid(mut self, name: &'static str, value: fn(&T) -> u64) -> Self {
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
    pub fn column_ts(mut self, name: &'static str, value: fn(&T) -> u64) -> Self {
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
    pub fn column_user_click(mut self, name: &'static str, gid: fn(&T) -> u64) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, _| gid(d).into()),
                show: Box::new(move |_, v, ui, w| {
                    let gid = v.get_u64().unwrap_or_default();
                    if gid == 0 {
                        "...".cstr().label(ui);
                    } else {
                        if gid
                            .get_user()
                            .cstr()
                            .as_button()
                            .active(gid == user_id())
                            .ui(ui)
                            .clicked()
                        {
                            TilePlugin::add_user(gid, w);
                        }
                    }
                }),
                sortable: true,
                hide_name: false,
            },
        );
        self
    }
    pub fn column_team(mut self, name: &'static str, gid: fn(&T) -> u64) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, _| gid(d).into()),
                show: Box::new(|_, gid: VarValue, ui: &mut Ui, w: &mut World| {
                    let gid = gid.get_u64().unwrap_or_default();
                    if gid == 0 {
                        "...".cstr().label(ui);
                    } else {
                        gid.get_team_cached().hover_label(ui, w);
                    }
                }),
                sortable: true,
                hide_name: false,
            },
        );
        self
    }
    pub fn column_rarity(mut self, value: fn(&T) -> i32) -> Self {
        self.columns.insert(
            "rarity",
            TableColumn {
                value: Box::new(move |d, _| value(d).into()),
                show: Box::new(|_, v, ui, _| {
                    let r = v.get_int().unwrap() as u8;
                    Rarity::from(r).cstr().label(ui);
                }),
                sortable: true,
                hide_name: false,
            },
        );
        self
    }
    pub fn column_base_unit_name_dyn(
        mut self,
        name: &'static str,
        unit: Box<dyn Fn(&T) -> String>,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, _| unit(d).into()),
                show: Box::new(|_, v, ui, world| {
                    let name = v.get_string().unwrap();
                    if let Some(unit) = TBaseUnit::find_by_name(name.clone()) {
                        let color = name_color(&unit.house);
                        if name.cstr_c(color).label(ui).hovered() {
                            cursor_window(ui.ctx(), |ui| {
                                match cached_base_card(&name.base_unit(), ui, world) {
                                    Ok(_) => {}
                                    Err(e) => error!("{e}"),
                                }
                            });
                        }
                    } else {
                        name.cstr_c(VISIBLE_LIGHT).label(ui);
                    }
                }),
                sortable: true,
                hide_name: false,
            },
        );
        self
    }
    pub fn column_base_unit_name(self, name: &'static str, unit: fn(&T) -> String) -> Self {
        self.column_base_unit_name_dyn(name, Box::new(unit))
    }
    pub fn column_base_unit(mut self, name: &'static str, unit: fn(&T) -> TBaseUnit) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, _| unit(d).name.into()),
                show: Box::new(move |d, _, ui, world| {
                    let unit = unit(d);
                    let resp = unit.name.cstr_c(name_color(&unit.house)).label(ui);
                    if resp.hovered() {
                        cursor_window(ui.ctx(), |ui| match cached_base_card(&unit, ui, world) {
                            Ok(_) => {}
                            Err(e) => error!("{e}"),
                        });
                    }
                }),
                sortable: true,
                hide_name: false,
            },
        );
        self
    }
    pub fn columns_item_kind(mut self, data: fn(&T) -> (ItemKind, u64)) -> Self {
        self.columns.insert(
            "type",
            TableColumn {
                value: Box::new(move |d, _| {
                    let (kind, id) = data(d);
                    match kind {
                        ItemKind::Unit => {
                            "unit".cstr_c(rarity_color(id.unit_item().unit.base_unit().rarity))
                        }
                        ItemKind::UnitShard => "unit shard"
                            .cstr_c(rarity_color(id.unit_shard_item().unit.base_unit().rarity)),
                        ItemKind::Lootbox => "lootbox".cstr_c(CYAN),
                        ItemKind::RainbowShard => "rainbow shard".cstr_rainbow(),
                    }
                    .into()
                }),
                show: Box::new(|_, v, ui, _| {
                    v.get_cstr().unwrap().label(ui);
                }),
                sortable: true,
                hide_name: false,
            },
        );
        self.columns.insert(
            "name",
            TableColumn {
                value: Box::new(move |d, _| {
                    let (kind, _) = data(d);
                    match kind {
                        ItemKind::Unit => "unit",
                        ItemKind::UnitShard => "shard",
                        ItemKind::Lootbox => "lootbox",
                        ItemKind::RainbowShard => "rainbow shard",
                    }
                    .into()
                }),
                show: Box::new(move |d, _, ui, world| {
                    let (kind, id) = data(d);
                    match kind {
                        ItemKind::Unit => {
                            let unit = id.unit_item().unit;
                            let r = unit.cstr_limit(0, true).button(ui);
                            if r.hovered() {
                                cursor_window(ui.ctx(), |ui| {
                                    match cached_fused_card(&unit, ui, world) {
                                        Ok(_) => {}
                                        Err(e) => error!("{e}"),
                                    }
                                });
                            }
                            if r.clicked() {
                                TilePlugin::add_fused_unit(unit, world);
                            }
                        }
                        ItemKind::UnitShard => {
                            let item = id.unit_shard_item();
                            let r = item.unit.cstr_c(name_color(&item.unit)).label(ui);
                            if r.hovered() {
                                cursor_window(ui.ctx(), |ui| {
                                    match cached_base_card(&item.unit.base_unit(), ui, world) {
                                        Ok(_) => {}
                                        Err(e) => error!("{e}"),
                                    }
                                });
                            }
                        }
                        ItemKind::Lootbox => {
                            match &id.lootbox_item().kind {
                                LootboxKind::Regular => "Regular".cstr_c(VISIBLE_LIGHT),
                                LootboxKind::House(house) => house.cstr_c(name_color(house)),
                            }
                            .label(ui);
                        }
                        ItemKind::RainbowShard => default(),
                    }
                }),
                sortable: true,
                hide_name: false,
            },
        );
        self
    }
    pub fn ui(&mut self, data: &[T], ui: &mut Ui, world: &mut World) -> TableState {
        let mut need_sort = false;
        let mut need_filter = false;
        let id = Id::new("table_").with(self.name).with(ui.id());
        let mut state = ui
            .ctx()
            .data_mut(|w| w.get_temp::<TableState>(id))
            .unwrap_or_default();
        let frame_nr = ui.ctx().frame_nr();
        if state.frame_nr + 1 != frame_nr {
            state = default();
        }
        state.frame_nr = frame_nr;

        if state.indices.len() != data.len() && state.filter.is_none() {
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

        Frame::none()
            .inner_margin(Margin::same(13.0))
            .show(ui, |ui| {
                ui.push_id(Id::new(self.name), |ui| {
                    ui.horizontal(|ui| {
                        format!("total: {}", state.indices.len()).cstr().label(ui);
                    });
                    TableBuilder::new(ui)
                        .columns(
                            Column::auto(),
                            self.columns.len() + self.selectable as usize,
                        )
                        .cell_layout(Layout::centered_and_justified(egui::Direction::TopDown))
                        .header(30.0, |mut row| {
                            for (i, (name, column)) in self.columns.iter().enumerate() {
                                row.col(|ui| {
                                    let clicked = if column.sortable {
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
                                        btn.ui(ui).clicked()
                                    } else if column.hide_name {
                                        false
                                    } else {
                                        Button::click(name.to_string())
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
                        .body(|body| {
                            body.rows(22.0, state.indices.len(), |mut row| {
                                let mut row_i = row.index();
                                if let Some(i) = state.indices.get(row_i) {
                                    row_i = *i;
                                }
                                row.set_selected(state.selected_row.is_some_and(|i| i == row_i));
                                for (col_i, (_, col)) in self.columns.iter().enumerate() {
                                    let index = (col_i, row_i);
                                    let cell = state.cells.entry(index).or_default();
                                    cell.update();
                                    row.col(|ui| {
                                        let d = &data[row_i];
                                        let v: VarValue =
                                            cell.get_cached(index, d, &col.value, world);
                                        (col.show)(d, v, ui, world);
                                        if cell.highlight > 0.0 {
                                            ui.painter().rect_stroke(
                                                ui.min_rect(),
                                                Rounding::same(13.0),
                                                Stroke::new(
                                                    1.0,
                                                    YELLOW.gamma_multiply(cell.highlight),
                                                ),
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
                            })
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
    }
}
