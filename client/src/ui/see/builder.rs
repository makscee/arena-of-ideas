use super::*;

pub struct SeeBuilder<'a, T> {
    data: &'a T,
    ctx: &'a Context<'a>,
}

pub struct SeeBuilderMut<'a, T> {
    data: &'a mut T,
    ctx: &'a Context<'a>,
}

impl<'a, T> SeeBuilder<'a, T> {
    pub fn new(data: &'a T, ctx: &'a Context<'a>) -> Self {
        Self { data, ctx }
    }

    pub fn data(&self) -> &'a T {
        self.data
    }

    pub fn context(&self) -> &'a Context<'a> {
        self.ctx
    }
}

impl<'a, T> SeeBuilderMut<'a, T> {
    pub fn new(data: &'a mut T, ctx: &'a Context<'a>) -> Self {
        Self { data, ctx }
    }

    pub fn data(&mut self) -> &mut T {
        self.data
    }

    pub fn context(&self) -> &'a Context<'a> {
        self.ctx
    }
}

impl<'a, T: SFnTitle> SeeBuilder<'a, T> {
    pub fn button(self, ui: &mut Ui) -> Response {
        self.data.cstr_title().button(ui)
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

impl<'a, T: SFnInfo> SeeBuilder<'a, T> {
    pub fn info(self) -> Cstr {
        self.data.see_info_cstr(self.ctx)
    }
}

impl<'a, T: SFnShow> SeeBuilder<'a, T> {
    pub fn show(self, ui: &mut Ui) {
        self.data.show(self.ctx, ui)
    }
}

impl<'a, T: SFnRecursive> SeeBuilder<'a, T> {
    pub fn recursive<F>(self, ui: &mut Ui, mut f: F)
    where
        F: FnMut(&mut Ui, &Context, RecursiveField<'_>),
    {
        self.data.recursive(self.ctx, ui, &mut f)
    }
}

impl<'a, T: SFnShowMut> SeeBuilderMut<'a, T> {
    pub fn show(self, ui: &mut Ui) -> bool {
        self.data.show_mut(self.ctx, ui)
    }
}
