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
    show: fn(&T, &Self, &mut Ui) -> Response,
    value: fn(&T) -> VarValue,
    width: f32,
    sortable: bool,
}

#[derive(Clone, PartialEq, Eq, Copy)]
enum Sorting {
    Asc,
    Desc,
}

impl<T> TableColumn<T> {
    pub fn new(value: fn(&T) -> VarValue) -> Self {
        Self {
            show: |v, s, ui| (s.value)(v).cstr().label(ui),
            value,
            width: 0.0,
            sortable: true,
        }
    }
    pub fn show(mut self, f: fn(&T, &Self, &mut Ui) -> Response) -> Self {
        self.show = f;
        self
    }
    pub fn no_sort(mut self) -> Self {
        self.sortable = false;
        self
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
    pub fn new_cached(name: &'static str, data: fn() -> Vec<T>, ctx: &egui::Context) -> Self {
        let id = Id::new(name);
        let cache = ctx.data_mut(|w| w.get_temp::<Table<T>>(id));
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
    pub fn ui(&mut self, ui: &mut Ui) {
        let mut need_sort = false;
        ui.horizontal(|ui| {
            for (i, (name, column)) in self.columns.iter_mut().enumerate() {
                ui.vertical(|ui| {
                    ui.set_width(column.width);
                    ui.vertical_centered_justified(|ui| {
                        let resp = if column.sortable {
                            let mut btn = Button::click(name.to_string()).gray(ui);
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
                        let rect = resp.rect;
                        column.width = column.width.max(rect.width());
                        ui.painter().line_segment(
                            [
                                rect.left_top() + egui::vec2(column.width + 5.0, 0.0),
                                rect.left_bottom() + egui::vec2(column.width + 5.0, 0.0),
                            ],
                            STROKE_DARK,
                        );
                    });
                    for (i, data) in self.data.iter().enumerate() {
                        let height = &mut self.heights[i];
                        let hovered = self.hovered_row.is_some_and(|r| i == r);
                        let selected = self.selected_row.is_some_and(|r| i == r);
                        if hovered {
                            ui.style_mut()
                                .visuals
                                .widgets
                                .noninteractive
                                .fg_stroke
                                .color = VISIBLE_BRIGHT;
                        } else if selected {
                            ui.style_mut()
                                .visuals
                                .widgets
                                .noninteractive
                                .fg_stroke
                                .color = YELLOW;
                        }
                        let cell =
                            Rect::from_min_size(ui.cursor().min, egui::vec2(column.width, *height))
                                .expand(3.0);
                        ui.horizontal(|ui| {
                            ui.set_min_height(*height);
                            let r = (column.show)(data, column, ui);
                            if r.hovered() {
                                self.hovered_row = Some(i);
                            }
                            if r.clicked() {
                                self.selected_row = Some(i);
                            }
                            *height = height.max(r.rect.height());
                            column.width = column.width.max(r.rect.width());
                        });

                        if selected {
                            ui.painter()
                                .line_segment([cell.left_top(), cell.right_top()], STROKE_YELLOW);
                            ui.painter().line_segment(
                                [cell.left_bottom(), cell.right_bottom()],
                                STROKE_YELLOW,
                            );
                        }
                        ui.reset_style();
                    }
                });
            }
        });
        if self.cached {
            if need_sort {
                let Some((i, sort)) = self.sorting else {
                    panic!("No sorting data")
                };
                let asc = sort == Sorting::Asc;
                let value = self.columns.values().nth(i).unwrap().value;
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
