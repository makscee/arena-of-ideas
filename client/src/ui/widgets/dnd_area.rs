use std::{any::Any, marker::PhantomData, sync::Arc};

use bevy_egui::egui::{DragAndDrop, LayerId};

use super::*;

pub struct DndArea<T> {
    pd: PhantomData<T>,
    text: Option<String>,
    rect: Rect,
    id: Id,
}

impl<T: Any + Send + Sync> DndArea<T> {
    pub fn new(rect: Rect) -> DndArea<T> {
        Self {
            pd: PhantomData,
            text: None,
            rect,
            id: Id::NULL,
        }
    }
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.id = Id::new(id);
        self
    }
    pub fn text(mut self, text: impl ToString) -> Self {
        self.text = Some(text.to_string());
        self
    }
    pub fn text_fn(mut self, ui: &mut Ui, text: impl FnOnce(&T) -> String) -> Self {
        if let Some(payload) = DragAndDrop::payload::<T>(ui.ctx()) {
            self.text = Some(text(&payload));
            self
        } else {
            self
        }
    }
    pub fn ui(self, ui: &mut Ui) -> Option<Arc<T>> {
        if !DragAndDrop::has_any_payload(ui.ctx())
            || !DragAndDrop::has_payload_of_type::<T>(ui.ctx())
        {
            // to keep auto_ids order since ui.new_child() increments it by 1
            ui.skip_ahead_auto_ids(1);
            return None;
        }
        let ui = &mut ui.new_child(
            UiBuilder::new()
                .max_rect(self.rect)
                .layer_id(LayerId::new(Order::Foreground, ui.id())),
        );
        let resp = ui.allocate_rect(self.rect, Sense::drag());
        let hovered = resp.contains_pointer();
        let color = if hovered {
            YELLOW
        } else {
            ui.visuals().widgets.active.fg_stroke.color
        };
        let t = ui.ctx().animate_bool(resp.id.with(self.id), hovered);
        ui.painter().rect_filled(
            self.rect,
            CornerRadius::ZERO,
            ui.visuals().faint_bg_color.alpha(0.8),
        );
        if let Some(text) = &self.text {
            let galley = text.cstr_cs(color, CstrStyle::Bold).galley(1.0, ui);
            let rect = Align2::CENTER_CENTER.anchor_size(self.rect.center(), galley.size());
            ui.painter()
                .galley(rect.min, galley, ui.visuals().text_color());
        }
        let length = resp.rect.width().min(resp.rect.height()) * 0.2;
        corners_rounded_rect(
            self.rect,
            length * (t * 0.5 + 1.0),
            color.stroke_w(2.0 + t * 2.0),
            ui,
        );
        resp.dnd_release_payload::<T>()
    }
}
