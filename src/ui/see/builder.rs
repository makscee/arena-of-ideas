use super::*;

pub struct SeeBuilder<'a, T> {
    data: &'a T,
    ctx: &'a Context<'a>,
}

impl<'a, T> SeeBuilder<'a, T> {
    pub fn new(data: &'a T, ctx: &'a Context<'a>) -> Self {
        Self { data, ctx }
    }
}

impl<'a, T: SFnTitle> SeeBuilder<'a, T> {
    pub fn button(self, ui: &mut Ui) -> Response {
        self.data.see_title_cstr().button(ui)
    }
}

impl<'a, T: SFnTag> SeeBuilder<'a, T> {
    pub fn tag(self, ui: &mut Ui) -> Response {
        self.data.see_tag(self.ctx, ui)
    }
}

impl<'a, T: SFnCard> SeeBuilder<'a, T> {
    pub fn card(self, ui: &mut Ui) -> Result<Response, ExpressionError> {
        self.data.see_card(self.ctx, ui)
    }
}

impl<'a, T: SFnTagCard> SeeBuilder<'a, T> {
    pub fn tag_card(self, ui: &mut Ui) -> Result<(), ExpressionError> {
        self.data.see_tag_card(self.ctx, ui)
    }

    pub fn tag_card_expanded(self, expanded: bool, ui: &mut Ui) -> Result<(), ExpressionError> {
        let expanded_id = self.data.egui_id().with(ui.id()).with("expanded");
        ui.ctx().data_mut(|w| w.insert_temp(expanded_id, expanded));
        self.data.see_tag_card(self.ctx, ui)
    }
}

impl<'a, T> SeeBuilder<'a, &'a T>
where
    &'a T: SFnNodeRating,
{
    pub fn node_rating(self, ui: &mut Ui) {
        self.data.see_node_rating(self.ctx, ui)
    }
}

impl<'a, T> SeeBuilder<'a, &'a T>
where
    &'a T: SFnNodeLinkRating,
{
    pub fn node_link_rating(self, ui: &mut Ui, is_parent: bool, id: u64) {
        self.data.see_node_link_rating(self.ctx, ui, is_parent, id)
    }
}
