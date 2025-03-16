use bevy::ecs::system::Single;
use egui_notify::{Toast, Toasts};

use super::*;

pub struct NotificationsPlugin;

#[derive(Resource, Default)]
struct NotificationsData {
    toasts: Toasts,
}

impl Plugin for NotificationsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NotificationsData>()
            .add_systems(Update, Self::update);
    }
}

fn rm(world: &mut World) -> Mut<NotificationsData> {
    world.resource_mut::<NotificationsData>()
}

impl NotificationsPlugin {
    pub fn test(world: &mut World) {
        rm(world)
            .toasts
            .basic("Error")
            .level(egui_notify::ToastLevel::Error);
        rm(world)
            .toasts
            .basic("Info")
            .level(egui_notify::ToastLevel::Info);
        rm(world)
            .toasts
            .basic("Success aspdoaspd sapdoj aspodj aspodj as")
            .level(egui_notify::ToastLevel::Success);
        rm(world)
            .toasts
            .basic("Warning")
            .level(egui_notify::ToastLevel::Warning);
        rm(world)
            .toasts
            .basic("Custom")
            .level(egui_notify::ToastLevel::Custom("C".into(), PURPLE));
    }
    fn update(mut data: ResMut<NotificationsData>, ctx: Single<&mut EguiContext>) {
        data.toasts.show(ctx.into_inner().get_mut());
    }
}
