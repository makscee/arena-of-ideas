use super::*;
use crate::IntoEnumIterator;
use crate::ui::widgets::Selector;

/// Composer for rendering enum selectors
pub struct SelectorComposer<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for SelectorComposer<T> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> SelectorComposer<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: ToCstr + AsRef<str> + IntoEnumIterator + Clone + PartialEq> Composer<T>
    for SelectorComposer<T>
{
    fn compose(&self, data: &T, _context: &Context, ui: &mut Ui) -> Response {
        let mut data_clone = data.clone();
        let (_old_value, response) = Selector::ui_enum(&mut data_clone, ui);
        response
    }
}

impl<T: ToCstr + AsRef<str> + IntoEnumIterator + Clone + PartialEq> ComposerMut<T>
    for SelectorComposer<T>
{
    fn compose_mut(&self, data: &mut T, _context: &Context, ui: &mut Ui) -> bool {
        let (old_value, _response) = Selector::ui_enum(data, ui);
        old_value.is_some()
    }
}

/// Extension methods for RenderBuilder to use selector composers
impl<'a, T> RenderBuilder<'a, T>
where
    T: ToCstr + AsRef<str> + IntoEnumIterator + Clone + PartialEq,
{
    /// Render as a selector
    pub fn selector(self, ui: &mut Ui) -> Response {
        SelectorComposer::new().compose(self.data(), self.context(), ui)
    }

    /// Edit as a selector
    pub fn edit_selector(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => SelectorComposer::new().compose_mut(data, self.ctx, ui),
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}
