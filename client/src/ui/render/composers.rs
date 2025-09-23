use super::*;

pub mod recursive;
pub use recursive::*;

mod advanced_composers;
pub use advanced_composers::*;

mod selector_composer;
pub use selector_composer::*;

pub mod menu;
pub use menu::*;

/// Base trait for all composers - they wrap data and can be composed
pub trait Composer<T> {
    /// Get immutable reference to the wrapped data
    fn data(&self) -> &T;

    /// Get mutable reference to the wrapped data - panics if data is not mutable
    fn data_mut(&mut self) -> &mut T;

    /// Check if the data reference is mutable
    fn is_mutable(&self) -> bool;

    /// Compose (render) the data to UI
    fn compose(self, context: &Context, ui: &mut Ui) -> Response;
}

/// Reference wrapper that composers use to hold data
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

    pub fn as_mut(&mut self) -> &mut T {
        match self {
            DataRef::Immutable(_) => {
                panic!("Attempted to get mutable reference from immutable data")
            }
            DataRef::Mutable(data) => data,
        }
    }

    pub fn is_mutable(&self) -> bool {
        matches!(self, DataRef::Mutable(_))
    }
}

/// Basic composer that just wraps data
pub struct DataComposer<'a, T> {
    data: DataRef<'a, T>,
}

impl<'a, T> DataComposer<'a, T> {
    pub fn new(data: &'a T) -> Self {
        Self {
            data: DataRef::Immutable(data),
        }
    }

    pub fn new_mut(data: &'a mut T) -> Self {
        Self {
            data: DataRef::Mutable(data),
        }
    }
}

impl<'a, T: FDisplay> Composer<T> for DataComposer<'a, T> {
    fn data(&self) -> &T {
        self.data.as_ref()
    }

    fn data_mut(&mut self) -> &mut T {
        self.data.as_mut()
    }

    fn is_mutable(&self) -> bool {
        self.data.is_mutable()
    }

    fn compose(self, context: &Context, ui: &mut Ui) -> Response {
        self.data.as_ref().display(context, ui)
    }
}

/// Title composer - wraps data that implements FTitle
pub struct TitleComposer<'a, T: FTitle> {
    data: DataRef<'a, T>,
}

impl<'a, T: FTitle> TitleComposer<'a, T> {
    pub fn new(data: &'a T) -> Self {
        Self {
            data: DataRef::Immutable(data),
        }
    }

    pub fn new_mut(data: &'a mut T) -> Self {
        Self {
            data: DataRef::Mutable(data),
        }
    }
}

impl<'a, T: FTitle> Composer<T> for TitleComposer<'a, T> {
    fn data(&self) -> &T {
        self.data.as_ref()
    }

    fn data_mut(&mut self) -> &mut T {
        self.data.as_mut()
    }

    fn is_mutable(&self) -> bool {
        self.data.is_mutable()
    }

    fn compose(self, context: &Context, ui: &mut Ui) -> Response {
        self.data.as_ref().title(context).label(ui)
    }
}

/// Button composer - wraps another composer and makes it clickable
pub struct ButtonComposer<C> {
    inner: C,
    semantic: Option<crate::ui::core::colorix::Semantic>,
    disabled: bool,
    min_width: Option<f32>,
}

impl<C> ButtonComposer<C> {
    pub fn new(inner: C) -> Self {
        Self {
            inner,
            semantic: None,
            disabled: false,
            min_width: None,
        }
    }

    pub fn semantic(mut self, semantic: crate::ui::core::colorix::Semantic) -> Self {
        self.semantic = Some(semantic);
        self
    }

    pub fn accent(mut self) -> Self {
        self.semantic = Some(crate::ui::core::colorix::Semantic::Accent);
        self
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = Some(width);
        self
    }
}

impl<T, C: Composer<T>> Composer<T> for ButtonComposer<C> {
    fn data(&self) -> &T {
        self.inner.data()
    }

    fn data_mut(&mut self) -> &mut T {
        self.inner.data_mut()
    }

    fn is_mutable(&self) -> bool {
        self.inner.is_mutable()
    }

    fn compose(self, context: &Context, ui: &mut Ui) -> Response {
        use crate::ui::core::colorix::UiColorixExt;

        if self.disabled {
            ui.disable();
        }
        let ButtonComposer {
            inner,
            semantic: _,
            disabled,
            min_width,
        } = self;
        let response = if let Some(semantic) = self.semantic {
            ui.colorix_semantic(semantic, |ui| {
                // Get the body content from inner composer
                let body_response = inner.compose(context, ui);
                Self::apply_button_styling(body_response, disabled, min_width, ui)
            })
        } else {
            // Get the body content from inner composer
            let body_response = inner.compose(context, ui);
            Self::apply_button_styling(body_response, disabled, min_width, ui)
        };

        response
    }
}

impl<C> ButtonComposer<C> {
    fn apply_button_styling(
        body_response: Response,
        disabled: bool,
        min_width: Option<f32>,
        ui: &mut Ui,
    ) -> Response {
        let sense = if disabled {
            egui::Sense::hover()
        } else {
            egui::Sense::click()
        };

        let button_rect = body_response.rect;

        // Apply minimum width if specified
        let final_rect = if let Some(min_width) = min_width {
            if button_rect.width() < min_width {
                egui::Rect::from_min_size(
                    button_rect.min,
                    egui::vec2(min_width, button_rect.height()),
                )
            } else {
                button_rect
            }
        } else {
            button_rect
        };

        let button_response = ui.interact(final_rect, body_response.id, sense);

        // Apply hover effects to the frame
        if button_response.hovered() && !disabled {
            let hover_color = ui.style().interact(&button_response).bg_stroke.color;
            ui.painter().rect_stroke(
                final_rect,
                egui::CornerRadius::same(4),
                egui::Stroke::new(1.0, hover_color),
                egui::StrokeKind::Outside,
            );
        }

        button_response
    }
}

/// Tag composer for compact tag view
pub struct TagComposer<'a, T: FTag> {
    data: DataRef<'a, T>,
}

impl<'a, T: FTag> TagComposer<'a, T> {
    pub fn new(data: &'a T) -> Self {
        Self {
            data: DataRef::Immutable(data),
        }
    }

    pub fn new_mut(data: &'a mut T) -> Self {
        Self {
            data: DataRef::Mutable(data),
        }
    }
}

impl<'a, T: FTag> Composer<T> for TagComposer<'a, T> {
    fn data(&self) -> &T {
        self.data.as_ref()
    }

    fn data_mut(&mut self) -> &mut T {
        self.data.as_mut()
    }

    fn is_mutable(&self) -> bool {
        self.data.is_mutable()
    }

    fn compose(self, context: &Context, ui: &mut Ui) -> Response {
        let data = self.data.as_ref();
        let name = data.tag_name(context);
        let color = data.tag_color(context);

        if let Some(value) = data.tag_value(context) {
            TagWidget::new_name_value(name, color, value).ui(ui)
        } else {
            TagWidget::new_name(name, color).ui(ui)
        }
    }
}

/// List composer that wraps Vec<T> and an element composer function
pub struct ListComposer<'a, T, F> {
    data: DataRef<'a, Vec<T>>,
    element_fn: F,
    hover_fn: Option<Box<dyn FnMut(&T, &Context, &mut Ui) + 'a>>,
    filter_fn: Option<Box<dyn Fn(&str, &T, &Context) -> bool>>,
    filter_id: Option<egui::Id>,
}

impl<'a, T, F> ListComposer<'a, T, F>
where
    F: Fn(&T, &Context, &mut Ui) -> Response,
{
    pub fn new(data: &'a Vec<T>, element_fn: F) -> Self {
        Self {
            data: DataRef::Immutable(data),
            element_fn,
            hover_fn: None,
            filter_fn: None,
            filter_id: None,
        }
    }

    pub fn new_mut(data: &'a mut Vec<T>, element_fn: F) -> Self {
        Self {
            data: DataRef::Mutable(data),
            element_fn,
            hover_fn: None,
            filter_fn: None,
            filter_id: None,
        }
    }

    pub fn with_hover<H>(mut self, hover_fn: H) -> Self
    where
        H: FnMut(&T, &Context, &mut Ui) + 'a,
    {
        self.hover_fn = Some(Box::new(hover_fn));
        self
    }

    pub fn with_filter<G>(mut self, filter_id: egui::Id, filter_fn: G) -> Self
    where
        G: Fn(&str, &T, &Context) -> bool + 'static,
    {
        self.filter_id = Some(filter_id);
        self.filter_fn = Some(Box::new(filter_fn));
        self
    }
}

impl<'a, T, F> Composer<Vec<T>> for ListComposer<'a, T, F>
where
    F: Fn(&T, &Context, &mut Ui) -> Response,
{
    fn data(&self) -> &Vec<T> {
        self.data.as_ref()
    }

    fn data_mut(&mut self) -> &mut Vec<T> {
        self.data.as_mut()
    }

    fn is_mutable(&self) -> bool {
        self.data.is_mutable()
    }

    fn compose(mut self, context: &Context, ui: &mut Ui) -> Response {
        let mut response = "[tw List:]".cstr().label(ui);
        let items: Vec<&T> =
            if let (Some(filter_id), Some(filter_fn)) = (self.filter_id, &self.filter_fn) {
                let filter_text: String = ui
                    .ctx()
                    .data_mut(|d| d.get_persisted_mut_or(filter_id, String::new()).clone());

                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    let mut filter_input = filter_text.clone();
                    if ui.text_edit_singleline(&mut filter_input).changed() {
                        ui.ctx().data_mut(|d| {
                            d.insert_persisted(filter_id, filter_input.clone());
                        });
                    }
                });

                if filter_text.is_empty() {
                    self.data.as_ref().iter().collect()
                } else {
                    self.data
                        .as_ref()
                        .iter()
                        .filter(|item| filter_fn(&filter_text, item, context))
                        .collect()
                }
            } else {
                self.data.as_ref().iter().collect()
            };

        for item in items {
            let item_response = ui.group(|ui| {
                ui.expand_to_include_x(ui.available_rect_before_wrap().right());
                (self.element_fn)(item, context, ui)
            });
            response = response.union(item_response.inner);

            if let Some(hover_fn) = self.hover_fn.as_mut() {
                if ui.rect_contains_pointer(item_response.response.rect) {
                    let ui = &mut ui.new_child(
                        UiBuilder::new()
                            .max_rect(item_response.response.rect)
                            .layout(Layout::right_to_left(Align::Center)),
                    );
                    ui.add_space(5.0);
                    hover_fn(item, context, ui);
                }
            }
        }

        response
    }
}

/// Card composer for full card views
pub struct CardComposer<'a, T: FTitle + FDescription + FStats> {
    data: DataRef<'a, T>,
}

impl<'a, T: FTitle + FDescription + FStats> CardComposer<'a, T> {
    pub fn new(data: &'a T) -> Self {
        Self {
            data: DataRef::Immutable(data),
        }
    }
}

impl<'a, T: FTitle + FDescription + FStats> Composer<T> for CardComposer<'a, T> {
    fn data(&self) -> &T {
        self.data.as_ref()
    }

    fn data_mut(&mut self) -> &mut T {
        self.data.as_mut()
    }

    fn is_mutable(&self) -> bool {
        self.data.is_mutable()
    }

    fn compose(self, context: &Context, ui: &mut Ui) -> Response {
        let data = self.data.as_ref();
        let color = context.color(ui);

        Frame::new()
            .inner_margin(2)
            .corner_radius(ROUNDING)
            .stroke(color.stroke())
            .show(ui, |ui| {
                let resp = ui.horizontal(|ui| data.title(context).button(ui)).inner;

                data.description(context).label_w(ui);

                ui.horizontal(|ui| {
                    for (var_name, var_value) in data.stats(context) {
                        TagWidget::new_var_value(var_name, var_value).ui(ui);
                    }
                });

                resp
            })
            .inner
    }
}

/// Frame composer that adds a frame around another composer
pub struct FramedComposer<C> {
    inner: C,
    color: Option<Color32>,
}

impl<C> FramedComposer<C> {
    pub fn new(inner: C) -> Self {
        Self { inner, color: None }
    }

    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
}

impl<T, C: Composer<T>> Composer<T> for FramedComposer<C> {
    fn data(&self) -> &T {
        self.inner.data()
    }

    fn data_mut(&mut self) -> &mut T {
        self.inner.data_mut()
    }

    fn is_mutable(&self) -> bool {
        self.inner.is_mutable()
    }

    fn compose(self, context: &Context, ui: &mut Ui) -> Response {
        let color = self.color.unwrap_or_else(|| context.color(ui));

        Frame::new()
            .inner_margin(2)
            .corner_radius(ROUNDING)
            .stroke(color.stroke())
            .show(ui, |ui| self.inner.compose(context, ui))
            .inner
    }
}
