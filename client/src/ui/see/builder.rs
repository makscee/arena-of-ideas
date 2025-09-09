use super::*;

pub enum DataRef<'a, T> {
    Immutable(&'a T),
    Mutable(&'a mut T),
}

impl<'a, T> DataRef<'a, T> {
    pub fn as_ref(&self) -> &T {
        match self {
            DataRef::Immutable(data) => data,
            DataRef::Mutable(data) => data,
        }
    }

    pub fn as_mut(&mut self) -> Option<&mut T> {
        match self {
            DataRef::Immutable(_) => None,
            DataRef::Mutable(data) => Some(data),
        }
    }
}

pub struct SeeBuilder<'a, T> {
    data: DataRef<'a, T>,
    ctx: &'a Context<'a>,
}

impl<'a, T> SeeBuilder<'a, T> {
    pub fn data_ref(&self) -> &DataRef<'a, T> {
        &self.data
    }
}

impl<'a, T> SeeBuilder<'a, T> {
    pub fn new(data: &'a T, ctx: &'a Context<'a>) -> Self {
        Self {
            data: DataRef::Immutable(data),
            ctx,
        }
    }

    pub fn new_mut(data: &'a mut T, ctx: &'a Context<'a>) -> Self {
        Self {
            data: DataRef::Mutable(data),
            ctx,
        }
    }

    pub fn data(&self) -> &T {
        self.data.as_ref()
    }

    pub fn context(&self) -> &'a Context<'a> {
        self.ctx
    }

    pub fn is_mutable(&self) -> bool {
        matches!(self.data, DataRef::Mutable(_))
    }
}

impl<'a, T: SFnCstrTitle> SeeBuilder<'a, T> {
    pub fn button(self, ui: &mut Ui) -> Response {
        self.data().cstr_title(self.context()).button(ui)
    }
}

impl<'a, T: SFnTag> SeeBuilder<'a, T> {
    pub fn tag(self, ui: &mut Ui) -> Response {
        self.data().see_tag(self.ctx, ui)
    }
}

impl<'a, T: SFnCard> SeeBuilder<'a, T> {
    pub fn card(self, ui: &mut Ui) -> Result<Response, ExpressionError> {
        self.data().see_card(self.ctx, ui)
    }
}

impl<'a, T: SFnTagCard> SeeBuilder<'a, T> {
    pub fn tag_card(self, ui: &mut Ui) -> Result<(), ExpressionError> {
        self.data().see_tag_card(self.ctx, ui)
    }

    pub fn tag_card_expanded(self, expanded: bool, ui: &mut Ui) -> Result<(), ExpressionError> {
        let expanded_id = self.data().egui_id().with(ui.id()).with("expanded");
        ui.ctx().data_mut(|w| w.insert_temp(expanded_id, expanded));
        self.data().see_tag_card(self.ctx, ui)
    }
}

impl<'a, T> SeeBuilder<'a, &'a T>
where
    &'a T: SFnNodeRating,
{
    pub fn node_rating(self, ui: &mut Ui) {
        self.data().see_node_rating(self.ctx, ui)
    }
}

impl<'a, T> SeeBuilder<'a, &'a T>
where
    &'a T: SFnNodeLinkRating,
{
    pub fn node_link_rating(self, ui: &mut Ui, is_parent: bool, id: u64) {
        self.data()
            .see_node_link_rating(self.ctx, ui, is_parent, id)
    }
}

impl<'a, T: SFnInfo> SeeBuilder<'a, T> {
    pub fn info(self) -> Cstr {
        self.data().see_info_cstr(self.ctx)
    }
}

impl<'a, T: SFnShow> SeeBuilder<'a, T> {
    pub fn show(self, ui: &mut Ui) {
        self.data().show(self.ctx, ui)
    }
}

impl<'a, T: SFnShowMut> SeeBuilder<'a, T> {
    pub fn show_mut(self, ui: &mut Ui) -> bool {
        match self.data {
            DataRef::Mutable(data) => data.show_mut(self.ctx, ui),
            DataRef::Immutable(_) => panic!("Tried to do mut operation on immutable data"),
        }
    }
}

impl<'a, T: SFnRecursive> SeeBuilder<'a, T> {
    pub fn recursive_readonly<F>(self, ui: &mut Ui, mut f: F)
    where
        F: FnMut(&mut Ui, &Context, RecursiveField<'_>),
    {
        self.data().recursive(self.ctx, ui, &mut f)
    }

    pub fn recursive_show(self, ui: &mut Ui) {
        self.recursive_readonly(ui, |ui, context, field| {
            if !field.name.is_empty() {
                field.name.label(ui);
            }
            call_on_recursive_value!(field, show, context, ui);
        })
    }
}

impl<'a, T: SFnRecursiveMut> SeeBuilder<'a, T> {
    pub fn recursive<F>(self, ui: &mut Ui, mut f: F)
    where
        F: FnMut(&mut Ui, &Context, RecursiveFieldMut<'_>),
    {
        match self.data {
            DataRef::Mutable(data) => data.recursive(self.ctx, ui, &mut f),
            DataRef::Immutable(_) => {
                panic!("Tried to do mut operation on immutable data")
            }
        }
    }

    pub fn recursive_show_mut(self, ui: &mut Ui) -> bool {
        let mut changed = false;
        self.recursive(ui, |ui, context, field| {
            if !field.name.is_empty() {
                field.name.label(ui);
            }
            changed |= call_on_recursive_value_mut!(field, show_mut, context, ui);
        });
        changed
    }
}
