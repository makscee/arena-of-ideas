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
            Context::from_world_ref_r(world, |context| {
                Ok(match NPlayer::get_by_id(id, context) {
                    Ok(p) => p.player_name.cstr_c(tokens_global().high_contrast_text()),
                    Err(e) => format!("[red [s player#{id}] {e}]"),
                })
            })
            .unwrap()
        })
    }
}
