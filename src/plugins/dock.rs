use super::*;

pub struct DockPlugin;

#[derive(Resource)]
struct DockResource {
    dock: Dock,
}

impl Plugin for DockPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DockResource { dock: Dock::new() });
        app.add_systems(Update, Self::ui);
    }
}

impl DockPlugin {
    fn ui(world: &mut World) {
        let Some(ctx) = &egui_context(world) else {
            return;
        };
        world.resource_mut::<DockResource>().dock.ui(ctx);
    }
}
