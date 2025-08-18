use super::*;

pub trait SFnNodeRating {
    fn see_node_rating(&self, context: &Context, ui: &mut Ui);
}

impl<T> SFnNodeRating for &T
where
    T: NodeExt,
{
    fn see_node_rating(&self, _context: &Context, ui: &mut Ui) {
        let Some(r) = self.id().node_rating() else {
            "[red Node not found]".cstr().label(ui);
            return;
        };
        rating_button(
            ui,
            r.cstr_expanded(),
            false,
            |ui| {
                "node rating vote".cstr().label(ui);
            },
            || {
                cn().reducers
                    .content_vote_node(self.id(), true)
                    .notify_error_op();
            },
            || {
                cn().reducers
                    .content_vote_node(self.id(), false)
                    .notify_error_op();
            },
        );
    }
}

fn rating_button(
    ui: &mut Ui,
    text: String,
    active: bool,
    open: impl FnOnce(&mut Ui),
    minus: impl FnOnce(),
    plus: impl FnOnce(),
) {
    text.as_button().active(active, ui).ui(ui).bar_menu(|ui| {
        ui.vertical(|ui| {
            open(ui);
            ui.horizontal(|ui| {
                if "[red [b -]]".cstr().button(ui).clicked() {
                    plus()
                }
                if "[green [b +]]".cstr().button(ui).clicked() {
                    minus()
                }
            });
        });
    });
}
