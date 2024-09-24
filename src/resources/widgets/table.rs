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
    value: Box<dyn Fn(&T, &World) -> VarValue>,
    show: Box<dyn Fn(&T, VarValue, &mut Ui, &mut World) -> Response>,
    sortable: bool,
}

impl<T> TableColumn<T> {
    pub fn no_sort(mut self) -> Self {
        self.sortable = false;
        self
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
        show: fn(&T, VarValue, &mut Ui, &mut World) -> Response,
        sortable: bool,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(value),
                show: Box::new(show),
                sortable,
            },
        );
        self
    }
    pub fn column_btn(mut self, name: &'static str, on_click: fn(&T, &mut Ui, &mut World)) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(|_, _| name.to_string().into()),
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
    pub fn column_cstr(mut self, name: &'static str, s: fn(&T, &World) -> Cstr) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, w| s(d, w).into()),
                show: Box::new(|_, v, ui, _| v.get_cstr().unwrap().label(ui)),
                sortable: true,
            },
        );
        self
    }
    pub fn column_cstr_value(
        mut self,
        name: &'static str,
        v: fn(&T) -> VarValue,
        s: fn(&T) -> Cstr,
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, _| v(d).into()),
                show: Box::new(move |d, _, ui, _| s(d).label(ui)),
                sortable: true,
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
                    let r = v.get_cstr().unwrap().clone().button(ui);
                    if r.clicked() {
                        on_click(d, w);
                    }
                    r
                }),
                sortable: false,
            },
        );
        self
    }
    pub fn column_int(mut self, name: &'static str, value: fn(&T) -> i32) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, _| value(d).into()),
                show: Box::new(|_, v, ui, _| v.cstr().label(ui)),
                sortable: true,
            },
        );
        self
    }
    pub fn column_gid(mut self, name: &'static str, value: fn(&T) -> u64) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, _| value(d).into()),
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
                value: Box::new(move |d, _| value(d).into()),
                show: Box::new(|_, v, ui, _| {
                    format_timestamp(v.get_u64().unwrap())
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
        gid: fn(&T) -> u64,
        on_click: fn(u64, &mut Ui, &mut World),
    ) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, _| gid(d).into()),
                show: Box::new(move |_, v, ui, w| {
                    let gid = v.get_u64().unwrap();
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
    pub fn column_team(mut self, name: &'static str, gid: fn(&T) -> u64) -> Self {
        self.columns.insert(
            name,
            TableColumn {
                value: Box::new(move |d, _| gid(d).into()),
                show: Box::new(|_, gid: VarValue, ui: &mut Ui, w: &mut World| {
                    let gid = gid.get_u64().unwrap();
                    if gid == 0 {
                        "...".cstr().label(ui)
                    } else {
                        let team = gid.get_team();
                        let r = team.cstr().button(ui);
                        if r.clicked() {
                            TilePlugin::add_team(team.id, w);
                        }
                        r
                    }
                }),
                sortable: true,
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
                    Rarity::from_int(v.get_int().unwrap() as i8)
                        .cstr()
                        .label(ui)
                }),
                sortable: true,
            },
        );
        self
    }
    pub fn column_base_unit_box(
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
                    let r = match try_name_color(&name) {
                        Some(color) => {
                            let r = name.cstr_c(color).label(ui);
                            if r.hovered() {
                                cursor_window(ui.ctx(), |ui| {
                                    match cached_base_card(
                                        &TBaseUnit::find_by_name(name).unwrap(),
                                        ui,
                                        world,
                                    ) {
                                        Ok(_) => {}
                                        Err(e) => error!("{e}"),
                                    }
                                });
                            }
                            r
                        }
                        None => name.cstr_c(VISIBLE_LIGHT).label(ui),
                    };
                    r
                }),
                sortable: true,
            },
        );
        self
    }
    pub fn column_base_unit(self, name: &'static str, unit: fn(&T) -> String) -> Self {
        self.column_base_unit_box(name, Box::new(unit))
    }
    pub fn columns_item_kind(mut self, data: fn(&T) -> (ItemKind, u64)) -> Self {
        self.columns.insert(
            "type",
            TableColumn {
                value: Box::new(move |d, _| {
                    let (kind, id) = data(d);
                    match kind {
                        ItemKind::Unit => "unit".cstr_c(
                            Rarity::from_base(&TUnitItem::find_by_id(id).unwrap().unit.bases[0])
                                .color(),
                        ),
                        ItemKind::UnitShard => "unit shard".cstr_c(rarity_color(
                            TBaseUnit::find_by_name(id.get_unit_shard_item().unit)
                                .unwrap()
                                .rarity,
                        )),
                        ItemKind::Lootbox => "lootbox".cstr_c(CYAN),
                    }
                    .into()
                }),
                show: Box::new(|_, v, ui, _| v.get_cstr().unwrap().label(ui)),
                sortable: true,
            },
        );
        self.column_base_unit_box(
            "name",
            Box::new(move |d| {
                let (kind, id) = data(d);
                match kind {
                    ItemKind::Unit => TUnitItem::find_by_id(id).unwrap().unit.bases[0].clone(),
                    ItemKind::UnitShard => TUnitShardItem::find_by_id(id)
                        .map(|u| u.unit)
                        .unwrap_or_else(|| "error".into()),
                    ItemKind::Lootbox => TLootboxItem::find_by_id(id)
                        .map(|u| match u.kind {
                            LootboxKind::Regular => "Regular".into(),
                        })
                        .unwrap_or_else(|| "error".into()),
                }
                .into()
            }),
        )
    }
    pub fn ui(&mut self, data: &[T], ui: &mut Ui, world: &mut World) -> TableState {
        let mut need_sort = false;
        let mut need_filter = false;
        let id = Id::new("table_").with(self.name).with(ui.id());
        let mut state = ui
            .ctx()
            .data_mut(|w| w.get_temp::<TableState>(id))
            .unwrap_or_default();
        if state.indices.is_empty() && state.filter.is_none() || state.indices.len() > data.len() {
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
                        format!("total: {}", data.len()).cstr().label(ui);
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
                                        let v = (col.value)(d, world);
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
