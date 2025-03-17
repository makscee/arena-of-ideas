use bevy::{
    app::Update,
    ecs::system::{ResMut, Single},
    log::{error, info, warn},
};
use bevy_egui::{egui::Style, EguiContext};
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
        app.init_resource::<NotificationsData>()
            .add_systems(Update, Self::update);
    }
}

impl NotificationsPlugin {
    fn update(mut data: ResMut<NotificationsData>, ctx: Single<&mut EguiContext>) {
        data.toasts.show(ctx.into_inner().get_mut());
    }
}

impl Notification {
    pub fn new_string(text: String) -> Self {
        Self::new(text.cstr_c(VISIBLE_LIGHT))
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
        let Some(ctx) = &egui_context(world) else {
            return;
        };
        let text = self.text.to_colored();
        match self.level {
            ToastLevel::Info => info!("{}", text),
            ToastLevel::Warning => warn!("{}", text),
            ToastLevel::Error => error!("{}", text),
            ToastLevel::Custom(_, _) | ToastLevel::None | ToastLevel::Success => info!("{}", text),
        }
        rm(world).toasts.add(self.to_toast(ctx.style().as_ref()));
    }
}

pub trait NotificationPusher {
    fn to_notification(&self) -> Notification;
    fn notify(&self, world: &mut World) {
        self.to_notification().push(world)
    }
    fn notify_op(&self) {
        self.to_notification().push_op()
    }
    fn notify_error(&self, world: &mut World) {
        self.to_notification().error().push(world)
    }
    fn notify_error_op(&self) {
        self.to_notification().error().push_op();
    }
}

impl NotificationPusher for String {
    fn to_notification(&self) -> Notification {
        Notification::new_string(self.clone())
    }
}
impl NotificationPusher for str {
    fn to_notification(&self) -> Notification {
        Notification::new_string(self.into())
    }
}
