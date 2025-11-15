use bevy_egui::egui::UiKind;

use super::*;

/// Result of a menu action
#[derive(Debug, Clone)]
pub enum MenuAction<T: Clone> {
    Delete(T),
    Paste(T),
    Copy,
    Custom(String),
}

/// Menu item definition
pub enum MenuItem<'a, T: Clone> {
    Action(
        String,
        Box<dyn FnOnce(T, &ClientContext) -> Option<MenuAction<T>> + 'a>,
    ),
    Submenu(String, Vec<MenuItem<'a, T>>),
    SubmenuClosure(
        String,
        Box<dyn Fn(&mut Ui, &T, &ClientContext) -> Option<MenuAction<T>> + 'a>,
    ),
    Separator,
}

/// Response from a menu interaction
pub struct MenuResponse<T: Clone> {
    pub response: Response,
    pub action: Option<MenuAction<T>>,
}

impl<T: Clone> MenuResponse<T> {
    pub fn clicked(&self) -> bool {
        self.response.clicked()
    }

    pub fn deleted(&self) -> Option<&T> {
        if let Some(MenuAction::Delete(item)) = &self.action {
            Some(item)
        } else {
            None
        }
    }

    pub fn pasted(&self) -> Option<&T> {
        if let Some(MenuAction::Paste(item)) = &self.action {
            Some(item)
        } else {
            None
        }
    }

    pub fn custom_action(&self) -> Option<&String> {
        if let Some(MenuAction::Custom(name)) = &self.action {
            Some(name)
        } else {
            None
        }
    }
}

/// Menu composer that wraps another composer and adds a menu button
pub struct MenuComposer<'a, T: Clone, C: Composer<T>> {
    inner: C,
    actions: Vec<MenuItem<'a, T>>,
    dangerous_actions: Vec<MenuItem<'a, T>>,
}

impl<'a, T: Clone, C: Composer<T>> MenuComposer<'a, T, C> {
    pub fn new(inner: C) -> Self {
        Self {
            inner,
            actions: Vec::new(),
            dangerous_actions: Vec::new(),
        }
    }

    pub fn add_action<F>(mut self, name: impl ToString, f: F) -> Self
    where
        F: FnOnce(T, &ClientContext) -> Option<MenuAction<T>> + 'a,
    {
        self.actions
            .push(MenuItem::Action(name.to_string(), Box::new(f)));
        self
    }

    pub fn add_action_empty(mut self, name: impl ToString) -> Self {
        let name = name.to_string();
        self.actions.push(MenuItem::Action(
            name.clone(),
            Box::new(|_, _| Some(MenuAction::Custom(name))),
        ));
        self
    }

    pub fn add_dangerous_action<F>(mut self, name: impl ToString, f: F) -> Self
    where
        F: FnOnce(T, &ClientContext) -> Option<MenuAction<T>> + 'a,
    {
        self.dangerous_actions
            .push(MenuItem::Action(name.to_string(), Box::new(f)));
        self
    }

    pub fn add_separator(mut self) -> Self {
        self.actions.push(MenuItem::Separator);
        self
    }

    pub fn add_dangerous_separator(mut self) -> Self {
        self.dangerous_actions.push(MenuItem::Separator);
        self
    }

    pub fn add_submenu<F>(mut self, name: impl ToString, f: F) -> Self
    where
        F: Fn(&mut Ui, &T, &ClientContext) -> Option<MenuAction<T>> + 'a,
    {
        self.actions
            .push(MenuItem::SubmenuClosure(name.to_string(), Box::new(f)));
        self
    }

    pub fn add_dangerous_submenu<F>(mut self, name: impl ToString, f: F) -> Self
    where
        F: Fn(&mut Ui, &T, &ClientContext) -> Option<MenuAction<T>> + 'a,
    {
        self.dangerous_actions
            .push(MenuItem::SubmenuClosure(name.to_string(), Box::new(f)));
        self
    }

    pub fn add_copy(self) -> Self
    where
        T: StringData,
    {
        self.add_action("📋 Copy".to_string(), |item, _| {
            clipboard_set(item.get_data());
            Some(MenuAction::Copy)
        })
    }

    pub fn add_paste(self) -> Self
    where
        T: StringData + Default,
    {
        self.add_action("📋 Paste".to_string(), |_, _| {
            clipboard_get()
                .and_then(|data| {
                    let mut item = T::default();
                    item.inject_data(&data).ok().map(|_| item)
                })
                .map(MenuAction::Paste)
        })
    }

    pub fn add_delete(self) -> Self {
        self.add_dangerous_action("🗑 Delete".to_string(), |item, _| {
            Some(MenuAction::Delete(item))
        })
    }

    /// Compose with menu - returns MenuResponse instead of Response
    pub fn compose_with_menu(mut self, ctx: &ClientContext, ui: &mut Ui) -> MenuResponse<T> {
        let mut action = None;

        let inner_response = ui
            .horizontal(|ui| {
                action = self.render_menu_button(ctx, ui);
                let inner_response = self.inner.compose(ctx, ui);

                inner_response
            })
            .inner;

        MenuResponse {
            response: inner_response,
            action,
        }
    }

    fn render_menu_button(&mut self, ctx: &ClientContext, ui: &mut Ui) -> Option<MenuAction<T>> {
        let circle_size = 12.0;

        let circle_response = RectButton::new_size(egui::Vec2::splat(circle_size)).ui(
            ui,
            |color, rect, _response, ui| {
                const SIZE: f32 = 0.1;
                ui.painter()
                    .circle_filled(rect.center_top(), rect.width() * SIZE, color);
                ui.painter()
                    .circle_filled(rect.center(), rect.width() * SIZE, color);
                ui.painter()
                    .circle_filled(rect.center_bottom(), rect.width() * SIZE, color);
            },
        );

        // Get data from inner composer
        let data = self.inner.data().clone();

        // Move actions out of self to avoid cloning
        let actions = std::mem::take(&mut self.actions);
        let dangerous_actions = std::mem::take(&mut self.dangerous_actions);

        let mut result = None;
        circle_response.show_menu(ui, |ui| {
            ui.set_min_width(120.0);

            // Normal actions
            for item in actions {
                if let Some(action) = Self::render_menu_item(item, &data, ctx, ui, false) {
                    result = Some(action);
                    break;
                }
            }

            // Separator before dangerous actions
            if !dangerous_actions.is_empty() {
                ui.separator();
            }

            // Dangerous actions
            for item in dangerous_actions {
                if let Some(action) = Self::render_menu_item(item, &data, ctx, ui, true) {
                    result = Some(action);
                    break;
                }
            }
        });
        result
    }

    fn render_menu_item(
        item: MenuItem<'_, T>,
        data: &T,
        ctx: &ClientContext,
        ui: &mut egui::Ui,
        dangerous: bool,
    ) -> Option<MenuAction<T>> {
        match item {
            MenuItem::Action(name, action) => {
                let button = if dangerous {
                    ui.add(
                        egui::Button::new(&name)
                            .fill(ui.visuals().error_fg_color.gamma_multiply(0.2)),
                    )
                } else {
                    ui.button(&name)
                };

                if button.clicked() {
                    ui.close_kind(UiKind::Menu);
                    return action(data.clone(), ctx);
                }
            }
            MenuItem::Submenu(name, sub_items) => {
                ui.menu_button(&name, |ui| {
                    for sub_item in sub_items {
                        if let Some(action) =
                            Self::render_menu_item(sub_item, data, ctx, ui, dangerous)
                        {
                            return Some(action);
                        }
                    }
                    None
                });
            }
            MenuItem::SubmenuClosure(name, closure) => {
                if let Some(inner) = ui.menu_button(&name, |ui| closure(ui, data, ctx)).inner {
                    return inner;
                }
            }
            MenuItem::Separator => {
                ui.separator();
            }
        }
        None
    }
}

impl<'a, T: Clone, C: Composer<T>> Composer<T> for MenuComposer<'a, T, C> {
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
        // For regular compose, just render the inner composer without menu
        self.inner.compose(ctx, ui)
    }
}

/// Extension trait to add menu support to any composer
pub trait WithMenu<T: Clone>: Composer<T> + Sized {
    fn with_menu<'a>(self) -> MenuComposer<'a, T, Self> {
        MenuComposer::new(self)
    }
}

impl<T: Clone, C: Composer<T>> WithMenu<T> for C {}
