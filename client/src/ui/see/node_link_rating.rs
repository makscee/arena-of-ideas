use super::*;

pub trait SFnNodeLinkRating {
    fn see_node_link_rating(&self, context: &Context, ui: &mut Ui, is_parent: bool, id: u64);
    fn get_node_link_rating(
        &self,
        context: &Context,
        is_parent: bool,
        id: u64,
    ) -> Result<(i32, bool), ExpressionError>;
}

impl<T> SFnNodeLinkRating for &T
where
    T: NodeExt,
{
    fn see_node_link_rating(&self, context: &Context, ui: &mut Ui, is_parent: bool, id: u64) {
        let (text, solid) =
            if let Ok((r, solid)) = self.get_node_link_rating(context, is_parent, id) {
                (r.cstr_expanded(), solid)
            } else {
                ("[tw _]".cstr(), false)
            };
        let (child, parent) = if is_parent {
            (self.id(), id)
        } else {
            (id, self.id())
        };
        rating_button(
            ui,
            text,
            solid,
            |ui| {
                "link rating vote".cstr().label(ui);
            },
            || {
                cn().reducers
                    .content_vote_link(parent, child, true)
                    .notify_error_op()
            },
            || {
                cn().reducers
                    .content_vote_link(parent, child, false)
                    .notify_error_op()
            },
        );
    }
    fn get_node_link_rating(
        &self,
        context: &Context,
        is_parent: bool,
        id: u64,
    ) -> Result<(i32, bool), ExpressionError> {
        let (child, parent) = if is_parent {
            (self.id(), id)
        } else {
            (id, self.id())
        };
        context
            .world()?
            .get_any_link_rating(parent, child)
            .to_e_not_found()
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
