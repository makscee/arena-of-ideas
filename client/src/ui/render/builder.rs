use super::*;
use crate::ui::see::{Cstr, CstrTrait, RecursiveField, RecursiveFieldMut};

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

pub struct RenderBuilder<'a, T> {
    data: DataRef<'a, T>,
    ctx: &'a Context<'a>,
    composers: Vec<Box<dyn Composer<T> + 'a>>,
}

impl<'a, T> RenderBuilder<'a, T> {
    pub fn new(data: &'a T, ctx: &'a Context<'a>) -> Self {
        Self {
            data: DataRef::Immutable(data),
            ctx,
            composers: Vec::new(),
        }
    }

    pub fn new_mut(data: &'a mut T, ctx: &'a Context<'a>) -> Self {
        Self {
            data: DataRef::Mutable(data),
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
        matches!(self.data, DataRef::Mutable(_))
    }

    /// Add a composer to the pipeline
    pub fn with_composer<C: Composer<T> + 'a>(mut self, composer: C) -> Self {
        self.composers.push(Box::new(composer));
        self
    }

    /// Apply all composers in sequence
    pub fn compose(self, ui: &mut Ui) -> Response {
        let mut response = ui.label("");
        for composer in self.composers {
            response = response.union(composer.compose(self.data.as_ref(), self.ctx, ui));
        }
        response
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

// Extension methods for FDisplay
impl<'a, T: FDisplay> RenderBuilder<'a, T> {
    pub fn display(self, ui: &mut Ui) {
        self.data().display(self.ctx, ui)
    }
}

// Extension methods for FEdit
impl<'a, T: FEdit> RenderBuilder<'a, T> {
    pub fn edit(self, ui: &mut Ui) -> bool {
        match self.data {
            DataRef::Mutable(data) => data.edit(self.ctx, ui),
            DataRef::Immutable(_) => {
                panic!("Tried to edit immutable data");
            }
        }
    }
}

// Extension methods for FRecursive
impl<'a, T: FRecursive + RecursiveFields + ToRecursiveValue> RenderBuilder<'a, T> {
    pub fn recursive<F>(self, ui: &mut Ui, f: F)
    where
        F: FnMut(&mut Ui, &Context, RecursiveField<'_>),
    {
        RecursiveComposer::new(f).compose(self.data.as_ref(), self.ctx, ui);
    }

    pub fn recursive_show(self, ui: &mut Ui) {
        self.recursive(ui, |ui, context, field| {
            if !field.name.is_empty() {
                field.name.label(ui);
            }
            call_on_recursive_value!(field, display, context, ui);
        })
    }
}

// Extension methods for FRecursiveMut
impl<'a, T: FRecursiveMut + RecursiveFieldsMut + ToRecursiveValueMut> RenderBuilder<'a, T> {
    pub fn recursive_mut<F>(self, ui: &mut Ui, f: F) -> bool
    where
        F: FnMut(&mut Ui, &Context, RecursiveFieldMut),
    {
        match self.data {
            DataRef::Mutable(data) => RecursiveMutComposer::new(f).compose_mut(data, self.ctx, ui),
            DataRef::Immutable(_) => {
                panic!("Tried to do mut operation on immutable data");
            }
        }
    }

    pub fn recursive_edit(self, ui: &mut Ui) -> bool {
        let mut changed = false;
        self.recursive_mut(ui, |ui, context, field| {
            if !field.name.is_empty() {
                field.name.label(ui);
            }
            changed |= call_on_recursive_value_mut!(field, show_mut, context, ui);
        });
        changed
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

// Extension methods for context menu
impl<'a, T: FContextMenu + FTitle> RenderBuilder<'a, T> {
    pub fn with_menu(self) -> ContextMenuBuilder<'a, T> {
        ContextMenuBuilder::new(self)
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

// Context menu builder
pub struct ContextMenuBuilder<'a, T: FContextMenu> {
    builder: RenderBuilder<'a, T>,
    actions: Vec<ContextAction<T>>,
    dangerous_actions: Vec<ContextAction<T>>,
}

impl<'a, T: FContextMenu + FTitle> ContextMenuBuilder<'a, T> {
    pub fn new(builder: RenderBuilder<'a, T>) -> Self {
        let default_actions = builder.data.as_ref().context_actions(builder.ctx);
        Self {
            builder,
            actions: default_actions,
            dangerous_actions: Vec::new(),
        }
    }

    pub fn add_action<F>(mut self, name: String, f: F) -> Self
    where
        F: FnOnce(T, &Context) -> Option<ActionResult<T>> + 'static,
    {
        self.actions.push(ContextAction::Action(name, Box::new(f)));
        self
    }

    pub fn add_dangerous_action<F>(mut self, name: String, f: F) -> Self
    where
        F: FnOnce(T, &Context) -> Option<ActionResult<T>> + 'static,
    {
        self.dangerous_actions
            .push(ContextAction::Action(name, Box::new(f)));
        self
    }

    pub fn add_separator(mut self) -> Self {
        self.actions.push(ContextAction::Separator);
        self
    }

    pub fn add_dangerous_separator(mut self) -> Self {
        self.dangerous_actions.push(ContextAction::Separator);
        self
    }

    pub fn title(self, ui: &mut Ui) -> ContextMenuResponse<T> {
        self.compose_with(ui, |builder, ui| builder.title_button(ui))
    }

    pub fn card(self, ui: &mut Ui) -> ContextMenuResponse<T>
    where
        T: FDescription + FStats,
    {
        self.compose_with(ui, |builder, ui| builder.card(ui))
    }

    fn compose_with<F>(mut self, ui: &mut Ui, f: F) -> ContextMenuResponse<T>
    where
        F: FnOnce(&mut RenderBuilder<'a, T>, &mut Ui) -> Response,
    {
        let mut action = None;

        // Extract all necessary data before moving self
        let actions = self.actions;
        let dangerous_actions = self.dangerous_actions;

        ui.horizontal(|ui| {
            let response = f(&mut self.builder, ui);

            // Render context menu button
            let menu_response = RectButton::new_size(12.0.v2()).ui(ui, |color, rect, _, ui| {
                const SIZE: f32 = 0.1;
                ui.painter()
                    .circle_filled(rect.center_top(), rect.width() * SIZE, color);
                ui.painter()
                    .circle_filled(rect.center(), rect.width() * SIZE, color);
                ui.painter()
                    .circle_filled(rect.center_bottom(), rect.width() * SIZE, color);
            });

            menu_response.bar_menu(|ui| {
                action = Self::render_menu_items(
                    actions,
                    dangerous_actions,
                    self.builder.data(),
                    &self.builder.context(),
                    ui,
                );
            });

            if response.clicked() && action.is_none() {
                action = Some(ActionResult::None);
            }
        });

        ContextMenuResponse { action }
    }

    fn render_menu_items(
        actions: Vec<ContextAction<T>>,
        dangerous_actions: Vec<ContextAction<T>>,
        data: &T,
        context: &Context,
        ui: &mut Ui,
    ) -> Option<ActionResult<T>> {
        let mut result = None;

        for action in actions {
            if let Some(r) = Self::render_menu_item(action, data, context, ui, false) {
                result = Some(r);
                break;
            }
        }

        if !dangerous_actions.is_empty() {
            ui.separator();
        }

        for action in dangerous_actions {
            if let Some(r) = Self::render_menu_item(action, data, context, ui, true) {
                result = Some(r);
                break;
            }
        }

        result
    }

    fn render_menu_item(
        item: ContextAction<T>,
        data: &T,
        context: &Context,
        ui: &mut Ui,
        dangerous: bool,
    ) -> Option<ActionResult<T>> {
        match item {
            ContextAction::Action(name, action) => {
                let button = if dangerous {
                    ui.add(
                        egui::Button::new(&name)
                            .fill(ui.visuals().error_fg_color.gamma_multiply(0.2)),
                    )
                } else {
                    ui.button(&name)
                };

                if button.clicked() {
                    ui.close_menu();
                    return action(data.clone(), context);
                }
            }
            ContextAction::Submenu(name, items) => {
                ui.menu_button(&name, |ui| {
                    for sub_item in items {
                        if let Some(r) =
                            Self::render_menu_item(sub_item, data, context, ui, dangerous)
                        {
                            return Some(r);
                        }
                    }
                    None
                });
            }
            ContextAction::Separator => {
                ui.separator();
            }
        }
        None
    }
}

pub struct ContextMenuResponse<T> {
    pub action: Option<ActionResult<T>>,
}

impl<T> ContextMenuResponse<T> {
    pub fn clicked(&self) -> bool {
        matches!(self.action, Some(ActionResult::None))
    }

    pub fn deleted(&self) -> Option<&T> {
        if let Some(ActionResult::Delete(ref data)) = self.action {
            Some(data)
        } else {
            None
        }
    }

    pub fn replaced(&self) -> Option<&T> {
        if let Some(ActionResult::Replace(ref data)) = self.action {
            Some(data)
        } else {
            None
        }
    }

    pub fn modified(&self) -> Option<&T> {
        if let Some(ActionResult::Modified(ref data)) = self.action {
            Some(data)
        } else {
            None
        }
    }
}

// Helper builders for common patterns
impl<'a, T: FCopy + FContextMenu + FTitle> ContextMenuBuilder<'a, T> {
    pub fn add_copy(self) -> Self {
        self.add_action("📋 Copy".to_string(), |item, _| {
            item.copy_to_clipboard();
            None
        })
    }
}

impl<'a, T: FPaste + FContextMenu + FTitle> ContextMenuBuilder<'a, T> {
    pub fn add_paste(self) -> Self {
        self.add_action("📋 Paste".to_string(), |_, _| {
            T::paste_from_clipboard().map(ActionResult::Replace)
        })
    }
}

impl<'a, T: FContextMenu + FTitle> ContextMenuBuilder<'a, T> {
    pub fn add_delete(self) -> Self {
        self.add_dangerous_action("🗑 Delete".to_string(), |item, _| {
            Some(ActionResult::Delete(item))
        })
    }
}
