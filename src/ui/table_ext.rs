use super::*;

pub trait TableExt<T> {
    fn add_player_column(self, name: &'static str, f: impl Fn(&T, &World) -> u64 + 'static)
        -> Self;
}

impl<'a, T: 'static + Clone + Send + Sync> TableExt<T> for Table<'a, T> {
    fn add_player_column(
        self,
        name: &'static str,
        f: impl Fn(&T, &World) -> u64 + 'static,
    ) -> Self {
        self.column_cstr_dyn(name, move |d, world| {
            let id = f(d, world);
            match Player::get_by_id(id, world) {
                Some(p) => p.name.cstr_c(tokens_global().high_contrast_text()),
                None => format!("[red [s player#{id} not found]]"),
            }
        })
    }
}
