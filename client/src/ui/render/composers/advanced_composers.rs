use super::*;
use std::marker::PhantomData;

/// Composer for selectable items
pub struct SelectableComposer<'a, T> {
    data: DataRef<'a, T>,
    selected: Option<T>,
    _phantom: PhantomData<T>,
}

impl<'a, T: PartialEq + Clone> SelectableComposer<'a, T> {
    pub fn new(data: &'a T, selected: Option<T>) -> Self {
        Self {
            data: DataRef::Immutable(data),
            selected,
            _phantom: PhantomData,
        }
    }

    pub fn new_mut(data: &'a mut T, selected: Option<T>) -> Self {
        Self {
            data: DataRef::Mutable(data),
            selected,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: FDisplay + PartialEq + Clone> Composer<T> for SelectableComposer<'a, T> {
    fn data(&self) -> &T {
        self.data.as_ref()
    }

    fn data_mut(&mut self) -> &mut T {
        self.data.as_mut()
    }

    fn is_mutable(&self) -> bool {
        self.data.is_mutable()
    }

    fn compose(&self, context: &Context, ui: &mut Ui) -> Response {
        let data = self.data.as_ref();
        let is_selected = self.selected.as_ref().map_or(false, |s| s == data);

        let response = if is_selected {
            Frame::new()
                .fill(ui.visuals().selection.bg_fill)
                .stroke(ui.visuals().selection.stroke)
                .corner_radius(ROUNDING)
                .inner_margin(2)
                .show(ui, |ui| data.display(context, ui))
                .inner
        } else {
            data.display(context, ui)
        };

        response
    }
}
