use super::*;
use crate::IntoEnumIterator;
use crate::ui::widgets::Selector;

/// Composer for rendering enum selectors
pub struct SelectorComposer<'a, T> {
    data: DataRef<'a, T>,
}

impl<'a, T> SelectorComposer<'a, T> {
    pub fn new_mut(data: &'a mut T) -> Self {
        Self {
            data: DataRef::Mutable(data),
        }
    }
}

impl<'a, T: ToCstr + AsRef<str> + IntoEnumIterator + Clone + PartialEq> Composer<T>
    for SelectorComposer<'a, T>
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

    fn compose(mut self, _context: &ClientContext, ui: &mut Ui) -> Response {
        if self.is_mutable() {
            let (_old_value, response) = Selector::ui_enum(self.data_mut(), ui);
            response
        } else {
            let data_clone = self.data().clone();
            ui.label(data_clone.as_ref())
        }
    }
}
