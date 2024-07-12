use super::*;

pub struct Table<T> {
    name: &'static str,
    columns: Vec<TableColumn<T>>,
    rows: Vec<T>,
}

struct TableColumn<T> {
    name: &'static str,
    show: Box<dyn Fn(T, &mut Ui)>,
}

impl<T> Table<T> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            columns: default(),
            rows: default(),
        }
    }
    pub fn column(
        mut self,
        name: &'static str,
        show: impl Fn(T, &mut Ui) + Send + Sync + 'static,
    ) -> Self {
        self.columns.push(TableColumn {
            name,
            show: Box::new(show),
        });
        self
    }
    pub fn ui(self, ui: &mut Ui) {}
}
