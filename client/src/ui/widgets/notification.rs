use bevy_egui::{EguiContext, egui::Style};
use egui_notify::{Toast, ToastLevel, Toasts};

use super::*;

#[derive(Clone, Debug)]
pub struct Notification {
    text: Cstr,
    level: ToastLevel,
}

pub struct NotificationsPlugin;

#[derive(Resource, Default)]
struct NotificationsData {
    toasts: Toasts,
}

fn rm(world: &mut World) -> Mut<NotificationsData> {
    world.resource_mut::<NotificationsData>()
}

impl Plugin for NotificationsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NotificationsData>();
    }
}

impl NotificationsPlugin {
    pub fn ui(ctx: &egui::Context, world: &mut World) {
        rm(world).toasts.show(ctx);
    }
}

impl Notification {
    pub fn new_string(text: String) -> Self {
        Self::new(text.cstr_c(high_contrast_text()))
    }
    pub fn new(text: Cstr) -> Self {
        Self {
            text,
            level: ToastLevel::Info,
        }
    }
    pub fn error(mut self) -> Self {
        self.level = ToastLevel::Error;
        self
    }
    fn to_toast(self, style: &Style) -> Toast {
        Toast::custom(self.text.widget(1.0, style), self.level)
    }
    pub fn push_op(self) {
        OperationsPlugin::add(|w| self.push(w));
    }
    pub fn push(self, world: &mut World) {
        let text = self.text.to_colored();
        match self.level {
            ToastLevel::Info => info!("{}", text),
            ToastLevel::Warning => warn!("{}", text),
            ToastLevel::Error => error!("{}", text),
            ToastLevel::Custom(_, _) | ToastLevel::None | ToastLevel::Success => info!("{}", text),
        }
        let style = world
            .query::<&EguiContext>()
            .single(world)
            .unwrap()
            .get()
            .style();
        rm(world).toasts.add(self.to_toast(style.as_ref()));
    }
}

pub trait NotificationPusher {
    fn to_notification(&self) -> Option<Notification>;
    fn notify(&self, world: &mut World) {
        if let Some(n) = self.to_notification() {
            n.push(world)
        }
    }
    fn notify_op(&self) {
        if let Some(n) = self.to_notification() {
            n.push_op()
        }
    }
    fn notify_error(&self, world: &mut World) {
        if let Some(n) = self.to_notification() {
            n.error().push(world)
        }
    }
    fn notify_error_op(&self) {
        if let Some(n) = self.to_notification() {
            n.error().push_op()
        }
    }
}

impl NotificationPusher for String {
    fn to_notification(&self) -> Option<Notification> {
        Some(Notification::new_string(self.clone()))
    }
}
impl NotificationPusher for str {
    fn to_notification(&self) -> Option<Notification> {
        Some(Notification::new_string(self.into()))
    }
}
impl<T> NotificationPusher for Result<T, NodeError> {
    fn to_notification(&self) -> Option<Notification> {
        match self {
            Ok(_) => None,
            Err(e) => {
                e.log();
                e.cstr().to_notification()
            }
        }
    }
}

impl<T> NotificationPusher for Result<T, spacetimedb_sdk::error::Error> {
    fn to_notification(&self) -> Option<Notification> {
        match self {
            Ok(_) => None,
            Err(e) => e.to_string().to_notification(),
        }
    }
}
