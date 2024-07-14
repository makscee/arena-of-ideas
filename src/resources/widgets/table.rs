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
}

#[derive(Clone)]
pub struct TableColumn<T> {
    show: fn(&T, &Self, &mut Ui) -> Response,
    value: fn(&T) -> VarValue,
    width: f32,
}

impl<T> TableColumn<T> {
    pub fn new(value: fn(&T) -> VarValue) -> Self {
        Self {
            show: |v, s, ui| (s.value)(v).cstr().label(ui),
            value,
            width: 0.0,
        }
    }
    pub fn show(mut self, f: fn(&T, &Self, &mut Ui) -> Response) -> Self {
        self.show = f;
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
        ui.horizontal(|ui| {
            for (name, column) in &mut self.columns {
                ui.vertical(|ui| {
                    column.width = column
                        .width
                        .max(name.cstr_c(VISIBLE_DARK).label(ui).rect.width());
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
                            ui.painter().line_segment(
                                [cell.left_top(), cell.right_top()],
                                Stroke {
                                    width: 1.0,
                                    color: YELLOW,
                                },
                            );
                            ui.painter().line_segment(
                                [cell.left_bottom(), cell.right_bottom()],
                                Stroke {
                                    width: 1.0,
                                    color: YELLOW,
                                },
                            );
                        }
                        ui.reset_style();
                    }
                });
            }
        });
        if self.cached {
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
