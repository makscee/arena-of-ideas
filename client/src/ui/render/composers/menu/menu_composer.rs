use super::*;
use crate::ui::render::builder::{RenderBuilder, RenderDataRef};
use crate::ui::widgets::RectButton;
use crate::{clipboard_get, clipboard_set};
use egui::{Response, Ui};
use schema::StringData;

/// Result of a menu action
#[derive(Debug, Clone)]
pub enum MenuAction<T: Clone> {
    Delete(T),
    Paste(T),
    Copy,
    Custom(Box<T>),
}

/// Menu item definition
pub enum MenuItem<'a, T: Clone> {
    Action(
        String,
        Box<dyn FnOnce(T, &Context) -> Option<MenuAction<T>> + 'a>,
    ),
    Submenu(String, Vec<MenuItem<'a, T>>),
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
        if let Some(MenuAction::Delete(ref item)) = self.action {
            Some(item)
        } else {
            None
        }
    }

    pub fn pasted(&self) -> Option<&T> {
        if let Some(MenuAction::Paste(ref item)) = self.action {
            Some(item)
        } else {
            None
        }
    }

    pub fn custom_action(&self) -> Option<&T> {
        if let Some(MenuAction::Custom(ref item)) = self.action {
            Some(item.as_ref())
        } else {
            None
        }
    }
}

/// Builder for adding a menu to any RenderBuilder
pub struct MenuBuilder<'a, T: Clone> {
    render_builder: RenderBuilder<'a, T>,
    actions: Vec<MenuItem<'a, T>>,
    dangerous_actions: Vec<MenuItem<'a, T>>,
}

impl<'a, T: Clone> MenuBuilder<'a, T> {
    pub fn new(render_builder: RenderBuilder<'a, T>) -> Self {
        Self {
            render_builder,
            actions: Vec::new(),
            dangerous_actions: Vec::new(),
        }
    }

    pub fn add_action<F>(mut self, name: String, f: F) -> Self
    where
        F: FnOnce(T, &Context) -> Option<MenuAction<T>> + 'a,
    {
        self.actions.push(MenuItem::Action(name, Box::new(f)));
        self
    }

    pub fn add_dangerous_action<F>(mut self, name: String, f: F) -> Self
    where
        F: FnOnce(T, &Context) -> Option<MenuAction<T>> + 'a,
    {
        self.dangerous_actions
            .push(MenuItem::Action(name, Box::new(f)));
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

    /// Show content with menu using a custom composer function
    pub fn show_with<F>(mut self, ui: &mut Ui, composer: F) -> MenuResponse<T>
    where
        F: FnOnce(&mut RenderBuilder<'a, T>, &mut Ui) -> Response,
    {
        let mut action = None;

        let inner_response = ui
            .horizontal(|ui| {
                // Render the content using the provided composer
                let inner_response = composer(&mut self.render_builder, ui);

                // Render menu button
                action = self.render_menu_button(ui);
                inner_response
            })
            .inner;

        // Handle paste action if data is mutable
        if let Some(MenuAction::Paste(ref new_data)) = action {
            if let RenderDataRef::Mutable(data) = &mut self.render_builder.data {
                **data = new_data.clone();
            }
        }

        MenuResponse {
            response: inner_response,
            action,
        }
    }

    /// Show content with menu using the default compose method
    pub fn show(self, ui: &mut Ui) -> MenuResponse<T> {
        self.show_with(ui, |builder, ui| builder.compose(ui))
    }

    /// Edit selector with menu
    pub fn edit_selector(mut self, ui: &mut Ui) -> MenuResponse<T>
    where
        T: ToCstr + AsRef<str> + IntoEnumIterator + PartialEq,
    {
        let mut changed = false;
        let mut action = None;
        let mut selector_response = None;

        ui.horizontal(|ui| {
            // Edit selector
            if let RenderDataRef::Mutable(data) = &mut self.render_builder.data {
                let (old_value, response) = Selector::ui_enum(*data, ui);
                selector_response = Some(response);
                if old_value.is_some() {
                    changed = true;
                }
            }

            // Render menu button
            action = self.render_menu_button(ui);
        });

        // Handle paste action
        if let Some(MenuAction::Paste(ref new_data)) = action {
            if let RenderDataRef::Mutable(data) = &mut self.render_builder.data {
                **data = new_data.clone();
                changed = true;
            }
        }

        MenuResponse {
            response: selector_response.unwrap_or_else(|| ui.label("")),
            action: action.or_else(|| {
                if changed {
                    Some(MenuAction::Custom(Box::new(
                        self.render_builder.data().clone(),
                    )))
                } else {
                    None
                }
            }),
        }
    }

    /// Edit selector with field moving for FRecursive types
    pub fn edit_selector_recursive(mut self, ui: &mut Ui) -> MenuResponse<T>
    where
        T: ToCstr + AsRef<str> + IntoEnumIterator + PartialEq + FRecursive,
    {
        let mut changed = false;
        let mut action = None;
        let mut selector_response = None;

        ui.horizontal(|ui| {
            // Edit selector with field moving
            if let RenderDataRef::Mutable(data) = &mut self.render_builder.data {
                let mut old_value = (*data).clone();
                let (old_val, response) = crate::ui::widgets::Selector::ui_enum(*data, ui);
                selector_response = Some(response);
                if old_val.is_some() {
                    // Move inner fields from old value to new value
                    data.move_inner_fields_from(&mut old_value);
                    changed = true;
                }
            }

            // Render menu button
            action = self.render_menu_button(ui);
        });

        // Handle paste action
        if let Some(MenuAction::Paste(ref new_data)) = action {
            if let RenderDataRef::Mutable(data) = &mut self.render_builder.data {
                **data = new_data.clone();
                changed = true;
            }
        }

        MenuResponse {
            response: selector_response.unwrap_or_else(|| ui.label("")),
            action: action.or_else(|| {
                if changed {
                    Some(MenuAction::Custom(Box::new(
                        self.render_builder.data().clone(),
                    )))
                } else {
                    None
                }
            }),
        }
    }

    fn render_menu_button(&mut self, ui: &mut Ui) -> Option<MenuAction<T>> {
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

        // Store data and context to use in closure
        let data = self.render_builder.data().clone();
        let context = self.render_builder.context();

        // Move actions out of self to avoid cloning
        let actions = std::mem::take(&mut self.actions);
        let dangerous_actions = std::mem::take(&mut self.dangerous_actions);

        let mut result = None;
        circle_response.bar_menu(|ui| {
            ui.set_min_width(120.0);

            // Normal actions
            for item in actions {
                if let Some(action) = Self::render_menu_item(item, &data, context, ui, false) {
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
                if let Some(action) = Self::render_menu_item(item, &data, context, ui, true) {
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
        context: &Context,
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
                    ui.close_menu();
                    return action(data.clone(), context);
                }
            }
            MenuItem::Submenu(name, sub_items) => {
                ui.menu_button(&name, |ui| {
                    for sub_item in sub_items {
                        if let Some(action) =
                            Self::render_menu_item(sub_item, data, context, ui, dangerous)
                        {
                            return Some(action);
                        }
                    }
                    None
                });
            }
            MenuItem::Separator => {
                ui.separator();
            }
        }
        None
    }
}
