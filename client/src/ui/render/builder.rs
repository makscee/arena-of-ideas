use super::*;
use crate::ui::render::composers::recursive::{RecursiveField, RecursiveFieldMut};

pub enum RenderDataRef<'a, T> {
    Immutable(&'a T),
    Mutable(&'a mut T),
}

impl<'a, T> RenderDataRef<'a, T> {
    pub fn as_ref(&self) -> &T {
        match self {
            RenderDataRef::Immutable(data) => data,
            RenderDataRef::Mutable(data) => data,
        }
    }

    pub fn as_mut(&mut self) -> Option<&mut T> {
        match self {
            RenderDataRef::Immutable(_) => None,
            RenderDataRef::Mutable(data) => Some(data),
        }
    }
}

pub struct RenderBuilder<'a, T> {
    pub(super) data: RenderDataRef<'a, T>,
    pub(super) ctx: &'a Context<'a>,
    composers: Vec<Box<dyn Composer<T> + 'a>>,
}

impl<'a, T> RenderBuilder<'a, T> {
    pub fn new(data: &'a T, ctx: &'a Context<'a>) -> Self {
        Self {
            data: RenderDataRef::Immutable(data),
            ctx,
            composers: Vec::new(),
        }
    }

    pub fn new_mut(data: &'a mut T, ctx: &'a Context<'a>) -> Self {
        Self {
            data: RenderDataRef::Mutable(data),
            ctx,
            composers: Vec::new(),
        }
    }

    pub fn data(&self) -> &T {
        self.data.as_ref()
    }

    pub fn context(&self) -> &'a Context<'a> {
        self.ctx
    }

    pub fn is_mutable(&self) -> bool {
        matches!(self.data, RenderDataRef::Mutable(_))
    }

    /// Add a composer to the pipeline
    pub fn with_composer<C: Composer<T> + 'a>(mut self, composer: C) -> Self {
        self.composers.push(Box::new(composer));
        self
    }

    /// Apply all composers in sequence
    pub fn compose(&self, ui: &mut Ui) -> Response {
        if self.composers.is_empty() {
            panic!("Tried to compose without any composers")
        }
        self.composers
            .iter()
            .map(|c| c.compose(self.data(), self.ctx, ui))
            .reduce(|a, b| a.union(b))
            .unwrap()
    }
}

// Extension methods for FTitle
impl<'a, T: FTitle> RenderBuilder<'a, T> {
    pub fn title(self, ui: &mut Ui) -> Response {
        TitleComposer.compose(self.data.as_ref(), self.ctx, ui)
    }

    pub fn title_label(&mut self, ui: &mut Ui) -> Response {
        self.data().title(self.ctx).label(ui)
    }

    pub fn title_button(&mut self, ui: &mut Ui) -> Response {
        self.data().title(self.ctx).button(ui)
    }
}

// Extension methods for FColoredTitle
impl<'a, T: FColoredTitle> RenderBuilder<'a, T> {
    pub fn colored_title(self, ui: &mut Ui) -> Response {
        self.data().colored_title(self.ctx).button(ui)
    }
}

// Extension methods for FTag
impl<'a, T: FTag> RenderBuilder<'a, T> {
    pub fn tag(self, ui: &mut Ui) -> Response {
        TagComposer.compose(self.data.as_ref(), self.ctx, ui)
    }
}

// Extension methods for FCompactView
impl<'a, T: FCompactView> RenderBuilder<'a, T> {
    pub fn compact_view(self, ui: &mut Ui) -> Response {
        CompactViewComposer::new().compose(self.data.as_ref(), self.ctx, ui)
    }

    pub fn compact_view_button(self, ui: &mut Ui) -> Response {
        CompactViewComposer::as_button().compose(self.data.as_ref(), self.ctx, ui)
    }
}

// Extension methods for FDisplay
impl<'a, T: FDisplay> RenderBuilder<'a, T> {
    pub fn display(self, ui: &mut Ui) -> Response {
        self.data().display(self.ctx, ui)
    }
}

// Extension methods for FEdit
impl<'a, T: FEdit> RenderBuilder<'a, T> {
    pub fn edit(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => data.edit(self.ctx, ui),
            RenderDataRef::Immutable(_) => {
                panic!("Tried to edit immutable data");
            }
        }
    }
}

// Extension methods for FRecursive
impl<'a, T: FRecursive> RenderBuilder<'a, T> {
    pub fn recursive<F>(self, ui: &mut Ui, f: F) -> Response
    where
        F: Fn(&mut Ui, &Context, &RecursiveField<'_>) -> Response + Clone,
    {
        RecursiveComposer::new(f)
            .with_layout(RecursiveLayout::HorizontalVertical)
            .compose(self.data.as_ref(), self.ctx, ui)
    }

    pub fn recursive_show(self, ui: &mut Ui) -> Response {
        recursive_display_composer(RecursiveLayout::HorizontalVertical).compose(
            self.data.as_ref(),
            self.ctx,
            ui,
        )
    }

    pub fn recursive_tree(self, ui: &mut Ui) -> Response {
        recursive_display_composer(RecursiveLayout::Tree { indent: 16.0 })
            .collapsible(true)
            .compose(self.data.as_ref(), self.ctx, ui)
    }
}

// Extension methods for FRecursive with mutable support
impl<'a, T: FRecursive> RenderBuilder<'a, T> {
    pub fn recursive_mut<F>(self, ui: &mut Ui, f: F) -> bool
    where
        F: FnMut(&mut Ui, &Context, &mut RecursiveFieldMut<'_>) -> bool,
    {
        match self.data {
            RenderDataRef::Mutable(data) => RecursiveComposer::new(f)
                .with_layout(RecursiveLayout::HorizontalVertical)
                .compose_mut(data, self.ctx, ui),
            RenderDataRef::Immutable(_) => {
                panic!("Tried to do mut operation on immutable data");
            }
        }
    }

    pub fn recursive_edit(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => {
                recursive_edit_composer(RecursiveLayout::HorizontalVertical)
                    .compose_mut(data, self.ctx, ui)
            }
            RenderDataRef::Immutable(_) => {
                panic!("Tried to do mut operation on immutable data");
            }
        }
    }

    pub fn recursive_edit_tree(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => {
                recursive_edit_composer(RecursiveLayout::Tree { indent: 0.0 })
                    .collapsible(true)
                    .compose_mut(data, self.ctx, ui)
            }
            RenderDataRef::Immutable(_) => {
                panic!("Tried to do mut operation on immutable data");
            }
        }
    }
}

// Extension methods for list rendering
impl<'a, T> RenderBuilder<'a, Vec<T>>
where
    T: Clone + Default,
{
    /// Render a list with a custom item composer
    pub fn list_with<C>(self, item_composer: C, ui: &mut Ui) -> Response
    where
        C: Composer<T>,
    {
        ListComposer::new(item_composer)
            .with_layout(ListLayout::Vertical)
            .compose(self.data.as_ref(), self.ctx, ui)
    }

    /// Edit a list with a custom item composer
    pub fn edit_list_with<C>(self, item_composer: C, ui: &mut Ui) -> bool
    where
        C: ComposerMut<T>,
    {
        match self.data {
            RenderDataRef::Mutable(data) => ListComposer::new(item_composer)
                .with_controls(true, true, true)
                .with_layout(ListLayout::Vertical)
                .compose_mut(data, self.ctx, ui),
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }

    /// Render a list in a grid layout
    pub fn list_grid<C>(self, item_composer: C, columns: usize, ui: &mut Ui) -> Response
    where
        C: Composer<T>,
    {
        ListComposer::new(item_composer)
            .with_layout(ListLayout::Grid { columns })
            .compose(self.data.as_ref(), self.ctx, ui)
    }

    /// Edit a list in a grid layout
    pub fn edit_list_grid<C>(self, item_composer: C, columns: usize, ui: &mut Ui) -> bool
    where
        C: ComposerMut<T>,
    {
        match self.data {
            RenderDataRef::Mutable(data) => ListComposer::new(item_composer)
                .with_controls(true, true, true)
                .with_layout(ListLayout::Grid { columns })
                .compose_mut(data, self.ctx, ui),
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

// Extension methods for recursive list rendering
impl<'a, T> RenderBuilder<'a, Vec<T>>
where
    T: FRecursive + Clone + Default,
{
    /// Display a list of recursive items
    pub fn recursive_list(self, ui: &mut Ui) -> Response {
        RecursiveListComposer::new().compose(self.data.as_ref(), self.ctx, ui)
    }

    /// Edit a list of recursive items
    pub fn edit_recursive_list(self, ui: &mut Ui) -> bool
    where
        T: FRecursive,
    {
        match self.data {
            RenderDataRef::Mutable(data) => RecursiveListComposer::new()
                .with_controls(true, true, true)
                .compose_mut(data, self.ctx, ui),
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

// Extension methods for card rendering (requires multiple features)
impl<'a, T: FTitle + FDescription + FStats> RenderBuilder<'a, T> {
    pub fn card(&mut self, ui: &mut Ui) -> Response {
        CardComposer.compose(self.data.as_ref(), self.ctx, ui)
    }
}

// Extension methods for expandable card
impl<'a, T: FTag + FTitle + FDescription + FStats + Node> RenderBuilder<'a, T> {
    pub fn tag_card(self, ui: &mut Ui) -> Response {
        TagCardComposer::default().compose(self.data.as_ref(), self.ctx, ui)
    }

    pub fn tag_card_expanded(self, expanded: bool, ui: &mut Ui) -> Response {
        TagCardComposer::new(expanded).compose(self.data.as_ref(), self.ctx, ui)
    }
}

// Extension method for menu (works with any Clone type)
impl<'a, T: Clone> RenderBuilder<'a, T> {
    pub fn with_menu(self) -> crate::ui::render::composers::menu::MenuBuilder<'a, T> {
        crate::ui::render::composers::menu::MenuBuilder::new(self)
    }
}

// Extension methods for info
impl<'a, T: FInfo> RenderBuilder<'a, T> {
    pub fn info(self) -> Cstr {
        self.data().info(self.ctx)
    }
}

// Extension methods for preview
impl<'a, T: FPreview> RenderBuilder<'a, T> {
    pub fn preview(self, ui: &mut Ui, size: Vec2) {
        self.data().preview(self.ctx, ui, size)
    }
}

// Extension methods for rating
impl<'a, T: FRating> RenderBuilder<'a, T> {
    pub fn rating(self, ui: &mut Ui) -> Response {
        RatingComposer.compose(self.data.as_ref(), self.ctx, ui)
    }
}
