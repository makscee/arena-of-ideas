use super::*;
use crate::ui::core::colorix::{Semantic, UiColorixExt};
use egui::{Frame, Rect, Response, Sense, Ui};

/// Button builder compositor that wraps RenderBuilder responses in a clickable frame
///
/// # Usage Examples
///
/// ```rust
/// // Basic usage with title composer
/// let response = data.render(context)
///     .with_composer(TitleComposer)
///     .as_button()
///     .accent()
///     .disabled()
///     .compose(ui);
///
/// // With multiple composers for button body
/// let response = data.render(context)
///     .with_composer(TagComposer)
///     .with_composer(TitleComposer)
///     .as_button()
///     .semantic(Semantic::Success)
///     .min_width(150.0)
///     .compose(ui);
/// ```
pub struct ButtonBuilderComposer<'a, T> {
    render_builder: RenderBuilder<'a, T>,
    semantic: Option<Semantic>,
    disabled: bool,
    inactive: bool,
    min_width: Option<f32>,
    frame_margin: Option<f32>,
    custom_style: Option<egui::Style>,
}

impl<'a, T> ButtonBuilderComposer<'a, T> {
    pub fn new(render_builder: RenderBuilder<'a, T>) -> Self {
        Self {
            render_builder,
            semantic: None,
            disabled: false,
            inactive: false,
            min_width: None,
            frame_margin: None,
            custom_style: None,
        }
    }

    fn apply_button_state(&self, ui: &mut Ui) {
        if self.inactive {
            let visuals = &mut ui.style_mut().visuals;
            visuals.widgets.inactive.fg_stroke.color = visuals.weak_text_color();
            visuals.widgets.hovered.fg_stroke.color = visuals.weak_text_color();
        }

        if self.disabled {
            ui.disable();
        }
    }

    fn apply_button_frame(&self, ui: &mut Ui, f: impl FnOnce(&mut Ui) -> Response) -> Response {
        if let Some(margin) = self.frame_margin {
            Frame::new().inner_margin(margin).show(ui, f).inner
        } else {
            f(ui)
        }
    }

    /// Set semantic coloring
    pub fn semantic(mut self, semantic: Semantic) -> Self {
        self.semantic = Some(semantic);
        self
    }

    /// Use accent semantic coloring
    pub fn accent(mut self) -> Self {
        self.semantic = Some(Semantic::Accent);
        self
    }

    /// Use success semantic coloring
    pub fn success(mut self) -> Self {
        self.semantic = Some(Semantic::Success);
        self
    }

    /// Use error semantic coloring
    pub fn error(mut self) -> Self {
        self.semantic = Some(Semantic::Error);
        self
    }

    /// Use warning semantic coloring
    pub fn warning(mut self) -> Self {
        self.semantic = Some(Semantic::Warning);
        self
    }

    /// Use background semantic coloring
    pub fn background(mut self) -> Self {
        self.semantic = Some(Semantic::Background);
        self
    }

    /// Set button as disabled (non-interactive but visible)
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// Set button as inactive (visually dimmed)
    pub fn inactive(mut self) -> Self {
        self.inactive = true;
        self
    }

    /// Set minimum width for the button
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Set frame margin
    pub fn frame_margin(mut self, margin: f32) -> Self {
        self.frame_margin = Some(margin);
        self
    }

    /// Make button appear selected/active
    pub fn selected(mut self, selected: bool) -> Self {
        if selected {
            self.semantic = Some(Semantic::Accent);
        }
        self
    }

    /// Make button appear as primary action
    pub fn primary(mut self) -> Self {
        self.semantic = Some(Semantic::Accent);
        self
    }

    /// Make button appear as dangerous/destructive action
    pub fn danger(mut self) -> Self {
        self.semantic = Some(Semantic::Error);
        self
    }
}

impl<'a, T> Composer<T> for ButtonBuilderComposer<'a, T> {
    fn compose(&self, _data: &T, _context: &Context, ui: &mut Ui) -> Response {
        let original_style = if self.custom_style.is_some() || self.semantic.is_some() {
            Some(ui.style().clone())
        } else {
            None
        };

        let response = if let Some(semantic) = self.semantic {
            ui.colorix_semantic(semantic, |ui| self.render_button_internal(ui))
        } else {
            // Apply custom style if specified
            if let Some(style) = &self.custom_style {
                ui.set_style(style.clone());
            }

            let response = self.render_button_internal(ui);

            // Restore original style if we modified it
            if let Some(style) = original_style {
                ui.set_style(style);
            }

            response
        };

        response
    }
}

impl<'a, T> ButtonBuilderComposer<'a, T> {
    fn render_button_internal(&self, ui: &mut Ui) -> Response {
        self.apply_button_state(ui);

        self.apply_button_frame(ui, |ui| {
            // Get the body content from RenderBuilder composers
            let body_response = if let Some(response) = self.render_builder.compose_safe(ui) {
                response
            } else {
                // If no composers, create a default label
                ui.label("Button")
            };

            // Create a clickable frame around the body content
            let sense = if self.disabled {
                Sense::hover()
            } else {
                Sense::click()
            };

            // Get the rect that encompasses the body content
            let button_rect = body_response.rect;

            // Apply minimum width if specified
            let final_rect = if let Some(min_width) = self.min_width {
                if button_rect.width() < min_width {
                    Rect::from_min_size(
                        button_rect.min,
                        egui::vec2(min_width, button_rect.height()),
                    )
                } else {
                    button_rect
                }
            } else {
                button_rect
            };

            // Create the interactive response for the entire button area
            let button_response = ui.interact(final_rect, body_response.id, sense);

            // Apply hover effects to the frame
            if button_response.hovered() && !self.disabled {
                let hover_color = ui.style().interact(&button_response).bg_stroke.color;
                ui.painter().rect_stroke(
                    final_rect,
                    egui::CornerRadius::same(4),
                    egui::Stroke::new(1.0, hover_color),
                    egui::StrokeKind::Outside,
                );
            }

            // Return the button response, not the body response
            button_response
        })
    }
}

// Extension to RenderBuilder to create ButtonBuilderComposer
impl<'a, T> RenderBuilder<'a, T> {
    /// Create a button builder compositor for this render builder
    pub fn as_button(self) -> ButtonBuilderComposer<'a, T> {
        ButtonBuilderComposer::new(self)
    }
}
