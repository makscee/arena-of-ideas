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
    fn compose(self, ctx: &ClientContext, ui: &mut Ui) -> Response;
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

    fn compose(self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.data.as_ref().display(ctx, ui)
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

    fn compose(self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        self.data.as_ref().title(ctx).label(ui)
    }
}

pub struct EmptyComposer<'a, T> {
    data: DataRef<'a, T>,
}

impl<'a, T> EmptyComposer<'a, T> {
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

impl<'a, T> Composer<T> for EmptyComposer<'a, T> {
    fn data(&self) -> &T {
        self.data.as_ref()
    }

    fn data_mut(&mut self) -> &mut T {
        self.data.as_mut()
    }

    fn is_mutable(&self) -> bool {
        self.data.is_mutable()
    }

    fn compose(self, _ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.new_child(UiBuilder::new())
            .allocate_response(default(), Sense::hover())
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

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
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

    fn compose(self, ctx: &ClientContext, ui: &mut Ui) -> Response {
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
                let body_response = inner.compose(ctx, ui);
                Self::apply_button_styling(body_response, disabled, min_width, ui)
            })
        } else {
            // Get the body content from inner composer
            let body_response = inner.compose(ctx, ui);
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

    fn compose(self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        let data = self.data.as_ref();
        let name = data.tag_name(ctx);
        let color = data.tag_color(ctx);

        if let Some(value) = data.tag_value(ctx) {
            TagWidget::new_name_value(name, color, value).ui(ui)
        } else {
            TagWidget::new_name(name, color).ui(ui)
        }
    }
}

/// List data and element function wrapper
pub enum ListData<'a, T> {
    Immutable {
        data: &'a Vec<T>,
        element_fn: Box<dyn Fn(&T, &ClientContext, &mut Ui) -> Response + 'a>,
    },
    Mutable {
        data: &'a mut Vec<T>,
        element_fn: Box<dyn Fn(&mut T, &ClientContext, &mut Ui) -> Response + 'a>,
        default_factory: Option<Box<dyn Fn() -> T + 'a>>,
    },
}

impl<'a, T> ListData<'a, T> {
    pub fn is_mutable(&self) -> bool {
        matches!(self, ListData::Mutable { .. })
    }

    pub fn data_ref(&self) -> &Vec<T> {
        match self {
            ListData::Immutable { data, .. } => data,
            ListData::Mutable { data, .. } => data,
        }
    }

    pub fn data_mut(&mut self) -> &mut Vec<T> {
        match self {
            ListData::Immutable { .. } => {
                panic!("Cannot get mutable reference to immutable list data")
            }
            ListData::Mutable { data, .. } => data,
        }
    }
}

pub struct ListComposer<'a, T> {
    list_data: ListData<'a, T>,
    hover_fn: Option<Box<dyn FnMut(&T, &ClientContext, &mut Ui) + 'a>>,
    hover_fn_mut: Option<Box<dyn FnMut(&mut T, &ClientContext, &mut Ui) + 'a>>,
    filter_fn: Option<Box<dyn Fn(&str, &T, &ClientContext) -> bool>>,
    filter_id: Option<egui::Id>,
    editable: bool,
}

impl<'a, T> ListComposer<'a, T> {
    pub fn new<F>(data: &'a Vec<T>, element_fn: F) -> Self
    where
        F: Fn(&T, &ClientContext, &mut Ui) -> Response + 'a,
    {
        Self {
            list_data: ListData::Immutable {
                data,
                element_fn: Box::new(element_fn),
            },
            hover_fn: None,
            hover_fn_mut: None,
            filter_fn: None,
            filter_id: None,
            editable: false,
        }
    }

    pub fn new_mut<F>(data: &'a mut Vec<T>, element_fn: F) -> Self
    where
        F: Fn(&mut T, &ClientContext, &mut Ui) -> Response + 'a,
    {
        Self {
            list_data: ListData::Mutable {
                data,
                element_fn: Box::new(element_fn),
                default_factory: None,
            },
            hover_fn: None,
            hover_fn_mut: None,
            filter_fn: None,
            filter_id: None,
            editable: false,
        }
    }

    pub fn editable<DF>(mut self, default_factory: DF) -> Self
    where
        DF: Fn() -> T + 'a,
    {
        match &mut self.list_data {
            ListData::Immutable { .. } => {
                panic!("Cannot enable editing on immutable list variant");
            }
            ListData::Mutable {
                default_factory: df,
                ..
            } => {
                *df = Some(Box::new(default_factory));
                self.editable = true;
            }
        }
        self
    }

    pub fn with_hover<H>(mut self, hover_fn: H) -> Self
    where
        H: FnMut(&T, &ClientContext, &mut Ui) + 'a,
    {
        self.hover_fn = Some(Box::new(hover_fn));
        self
    }

    pub fn with_hover_mut<H>(mut self, hover_fn: H) -> Self
    where
        H: FnMut(&mut T, &ClientContext, &mut Ui) + 'a,
    {
        self.hover_fn_mut = Some(Box::new(hover_fn));
        self
    }

    pub fn with_filter<G>(mut self, filter_id: egui::Id, filter_fn: G) -> Self
    where
        G: Fn(&str, &T, &ClientContext) -> bool + 'static,
    {
        self.filter_id = Some(filter_id);
        self.filter_fn = Some(Box::new(filter_fn));
        self
    }
}

impl<'a, T> Composer<Vec<T>> for ListComposer<'a, T> {
    fn data(&self) -> &Vec<T> {
        self.list_data.data_ref()
    }

    fn data_mut(&mut self) -> &mut Vec<T> {
        self.list_data.data_mut()
    }

    fn is_mutable(&self) -> bool {
        self.list_data.is_mutable()
    }

    fn compose(mut self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        let mut response = format!("[s [tw List ({}):]]", self.list_data.data_ref().len())
            .cstr()
            .label(ui);

        // Handle add button separately to avoid borrow checker issues
        if self.editable {
            let should_add_item: Option<bool> = match &self.list_data {
                ListData::Mutable {
                    default_factory: Some(_),
                    ..
                } => {
                    ui.horizontal(|ui| {
                        if ui.button("+ Add First").clicked() {
                            Some(true)
                        } else if ui.button("+ Add Last").clicked() {
                            Some(false)
                        } else {
                            None
                        }
                    })
                    .inner
                }
                _ => None,
            };

            if let Some(first) = should_add_item {
                match &mut self.list_data {
                    ListData::Mutable {
                        default_factory: Some(factory),
                        data,
                        ..
                    } => {
                        let new_item = factory();
                        if first {
                            data.insert(0, new_item);
                        } else {
                            data.push(new_item);
                        }
                        response.mark_changed();
                    }
                    _ => {}
                }
            }
        }

        let filtered_indices: Vec<usize> =
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
                    (0..self.data().len()).collect()
                } else {
                    (0..self.data().len())
                        .filter(|&i| filter_fn(&filter_text, &self.data()[i], ctx))
                        .collect()
                }
            } else {
                (0..self.data().len()).collect()
            };

        let mut indices_to_remove = Vec::new();
        let mut swap_operations = Vec::new();

        match &mut self.list_data {
            ListData::Immutable { element_fn, data } => {
                for (_display_idx, &actual_idx) in filtered_indices.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let item_response = ui.group(|ui| {
                            ui.expand_to_include_x(ui.available_rect_before_wrap().right());
                            element_fn(&data[actual_idx], ctx, ui)
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
                                hover_fn(&data[actual_idx], ctx, ui);
                            }
                        }
                    });
                }
            }
            ListData::Mutable {
                element_fn, data, ..
            } => {
                for (display_idx, &actual_idx) in filtered_indices.iter().enumerate() {
                    ui.horizontal(|ui| {
                        if self.editable {
                            ui.separator();

                            if "[b [red -]]".cstr().button(ui).clicked() {
                                indices_to_remove.push(actual_idx);
                            }

                            if display_idx > 0 && ui.small_button("🔼").clicked() {
                                let prev_actual_idx = filtered_indices[display_idx - 1];
                                swap_operations.push((actual_idx, prev_actual_idx));
                            }

                            if display_idx < filtered_indices.len() - 1
                                && ui.small_button("🔽").clicked()
                            {
                                let next_actual_idx = filtered_indices[display_idx + 1];
                                swap_operations.push((actual_idx, next_actual_idx));
                            }
                        }

                        let item_response = ui.group(|ui| {
                            ui.expand_to_include_x(ui.available_rect_before_wrap().right());
                            ui.push_id(actual_idx, |ui| element_fn(&mut data[actual_idx], ctx, ui))
                                .inner
                        });
                        response = response.union(item_response.inner);

                        if let Some(hover_fn) = self.hover_fn_mut.as_mut() {
                            if ui.rect_contains_pointer(item_response.response.rect) {
                                let ui = &mut ui.new_child(
                                    UiBuilder::new()
                                        .max_rect(item_response.response.rect)
                                        .layout(Layout::right_to_left(Align::Center)),
                                );
                                ui.add_space(5.0);
                                hover_fn(&mut data[actual_idx], ctx, ui);
                            }
                        }
                    });
                }
            }
        }

        // Apply remove operations (in reverse order to maintain indices)
        if self.is_mutable() {
            indices_to_remove.sort();
            indices_to_remove.reverse();
            for idx in indices_to_remove {
                self.data_mut().remove(idx);
                response.mark_changed();
            }

            // Apply swap operations
            for (idx1, idx2) in swap_operations {
                self.data_mut().swap(idx1, idx2);
                response.mark_changed();
            }
        }

        response
    }
}

/// Card composer for full card views
pub struct CardComposer<'a, T: FCard> {
    data: DataRef<'a, T>,
}

impl<'a, T: FCard> CardComposer<'a, T> {
    pub fn new(data: &'a T) -> Self {
        Self {
            data: DataRef::Immutable(data),
        }
    }
}

impl<'a, T: FCard> Composer<T> for CardComposer<'a, T> {
    fn data(&self) -> &T {
        self.data.as_ref()
    }

    fn data_mut(&mut self) -> &mut T {
        self.data.as_mut()
    }

    fn is_mutable(&self) -> bool {
        self.data.is_mutable()
    }

    fn compose(self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        let data = self.data.as_ref();
        data.render_card(ctx, ui)
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

    fn compose(self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        let color = self.color.unwrap_or_else(|| ctx.color());

        Frame::new()
            .inner_margin(2)
            .corner_radius(ROUNDING)
            .stroke(color.stroke())
            .show(ui, |ui| self.inner.compose(ctx, ui))
            .inner
    }
}
