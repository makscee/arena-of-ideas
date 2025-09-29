mod composers;
mod features;
mod recursive_impl;

pub use composers::*;
pub use features::*;

use super::*;

/// Main trait that types implement to get access to composer creation
pub trait Render: Sized {
    /// Create a title composer for this data
    fn as_title(&self) -> TitleComposer<'_, Self>
    where
        Self: FTitle,
    {
        TitleComposer::new(self)
    }

    /// Create a mutable title composer for this data
    fn as_title_mut(&mut self) -> TitleComposer<'_, Self>
    where
        Self: FTitle,
    {
        TitleComposer::new_mut(self)
    }

    /// Create a tag composer for this data
    fn as_tag(&self) -> TagComposer<'_, Self>
    where
        Self: FTag,
    {
        TagComposer::new(self)
    }

    /// Create a mutable tag composer for this data
    fn as_tag_mut(&mut self) -> TagComposer<'_, Self>
    where
        Self: FTag,
    {
        TagComposer::new_mut(self)
    }

    /// Create a data composer for basic display
    fn as_display(&self) -> DataComposer<'_, Self>
    where
        Self: FDisplay,
    {
        DataComposer::new(self)
    }

    /// Create a mutable data composer for basic display
    fn as_display_mut(&mut self) -> DataComposer<'_, Self>
    where
        Self: FDisplay,
    {
        DataComposer::new_mut(self)
    }

    /// Create a data composer for basic display (alias for as_display)
    fn as_data(&self) -> DataComposer<'_, Self>
    where
        Self: FDisplay,
    {
        DataComposer::new(self)
    }

    /// Create a mutable data composer for basic display (alias for as_display_mut)
    fn as_data_mut(&mut self) -> DataComposer<'_, Self>
    where
        Self: FDisplay,
    {
        DataComposer::new_mut(self)
    }

    /// Create a card composer for this data
    fn as_card(&self) -> CardComposer<'_, Self>
    where
        Self: FTitle + FDescription + FStats,
    {
        CardComposer::new(self)
    }

    fn as_recursive<F>(&self, f: F) -> RecursiveComposer<'_, Self, F>
    where
        Self: FRecursive,
        F: FnMut(&ClientContext, &mut Ui, RecursiveValue<'_>) -> Response,
    {
        RecursiveComposer::new(self, f)
    }

    fn as_recursive_mut<F>(&mut self, f: F) -> RecursiveComposerMut<'_, Self, F>
    where
        Self: FRecursive,
        F: FnMut(&ClientContext, &mut Ui, &mut RecursiveValueMut<'_>) -> Response,
    {
        RecursiveComposerMut::new_mut(self, f)
    }

    fn as_selector_mut(&mut self) -> SelectorComposer<'_, Self>
    where
        Self: ToCstr + AsRef<str> + IntoEnumIterator + Clone + PartialEq,
    {
        SelectorComposer::new_mut(self)
    }

    fn as_button<T>(self) -> ButtonComposer<Self>
    where
        Self: Composer<T>,
    {
        ButtonComposer::new(self)
    }
}

/// Blanket implementation for all types
impl<T> Render for T {}

/// Extension trait for Vec<T> to create list composers
pub trait RenderList<T> {
    /// Create a list composer with a closure that creates composers for each element
    fn as_list<'a, F>(
        &'a self,
        f: F,
    ) -> ListComposer<'a, T, impl Fn(&T, &ClientContext, &mut Ui) -> Response>
    where
        F: Fn(&T, &ClientContext, &mut Ui) -> Response + 'a,
        T: 'a;

    /// Create a mutable list composer with a closure that creates composers for each element
    fn as_list_mut<'a, F>(
        &'a mut self,
        f: F,
    ) -> ListComposer<'a, T, impl Fn(&T, &ClientContext, &mut Ui) -> Response>
    where
        F: Fn(&T, &ClientContext, &mut Ui) -> Response + 'a,
        T: 'a;
}

impl<T> RenderList<T> for Vec<T> {
    fn as_list<'a, F>(
        &'a self,
        f: F,
    ) -> ListComposer<'a, T, impl Fn(&T, &ClientContext, &mut Ui) -> Response>
    where
        F: Fn(&T, &ClientContext, &mut Ui) -> Response + 'a,
        T: 'a,
    {
        ListComposer::new(self, f)
    }

    fn as_list_mut<'a, F>(
        &'a mut self,
        f: F,
    ) -> ListComposer<'a, T, impl Fn(&T, &ClientContext, &mut Ui) -> Response>
    where
        F: Fn(&T, &ClientContext, &mut Ui) -> Response + 'a,
        T: 'a,
    {
        ListComposer::new_mut(self, f)
    }
}

/// Function composer that executes a closure to render data
pub struct FnComposer<'a, T, F> {
    data: DataRef<'a, T>,
    render_fn: F,
}

impl<'a, T, F> FnComposer<'a, T, F>
where
    F: Fn(&T, &ClientContext, &mut Ui) -> Response,
{
    pub fn new(data: &'a T, render_fn: F) -> Self {
        Self {
            data: DataRef::Immutable(data),
            render_fn,
        }
    }

    pub fn new_mut(data: &'a mut T, render_fn: F) -> Self {
        Self {
            data: DataRef::Mutable(data),
            render_fn,
        }
    }
}

impl<'a, T, F> Composer<T> for FnComposer<'a, T, F>
where
    F: Fn(&T, &ClientContext, &mut Ui) -> Response,
{
    fn data(&self) -> &T {
        self.data.as_ref()
    }

    fn data_mut(&mut self) -> &mut T {
        self.data.as_mut()
    }

    fn is_mutable(&self) -> bool {
        self.data.is_mutable()
    }

    fn compose(self, context: &ClientContext, ui: &mut Ui) -> Response {
        (self.render_fn)(self.data.as_ref(), context, ui)
    }
}

/// Extension trait for pairs to create selectable composers
pub trait RenderSelectable<T> {
    /// Create a selectable composer for the first element with the second as selection
    fn as_selectable(&self) -> SelectableComposer<'_, T>
    where
        T: PartialEq + Clone;
}

impl<T: PartialEq + Clone> RenderSelectable<T> for (T, T) {
    fn as_selectable(&self) -> SelectableComposer<'_, T> {
        SelectableComposer::new(&self.0, Some(self.1.clone()))
    }
}
