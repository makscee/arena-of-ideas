use super::*;

pub struct AdminPlugin;

impl Plugin for AdminPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Admin), (Self::setup, Self::on_enter))
            .add_systems(Update, (Self::update, Self::ui));
    }
}

impl AdminPlugin {
    fn on_enter(world: &mut World) {
        world.flush();
        for u in world.query::<&Unit>().iter(world) {
            debug!("Unit {}", u.name);
            debug!(
                "House {}",
                u.find_up::<House>(world)
                    .unwrap()
                    .get_var(VarName::name)
                    .unwrap()
            );
        }
    }
    fn setup(mut commands: Commands) {
        let house = houses().get("holy").unwrap().clone();
        dbg!(&house);
        house.unpack(commands.spawn_empty().id(), &mut commands);
    }
    fn ui(query: Query<&Unit>, mut ctx: Query<&mut EguiContext>) {
        let ctx = ctx.single_mut().into_inner().get_mut();
        for unit in query.iter() {
            Window::new("Unit").show(ctx, |ui| {
                unit.show(ui);
            });
        }
    }
    fn update(world: &mut World) {
        let egui_context = world
            .query_filtered::<&mut EguiContext, With<bevy::window::PrimaryWindow>>()
            .get_single(world);

        let Ok(egui_context) = egui_context else {
            return;
        };
        let mut egui_context = egui_context.clone();

        // egui::Window::new("World Inspector")
        //     .default_size(egui::vec2(300.0, 300.0))
        //     .show(egui_context.get_mut(), |ui| {
        //         egui::ScrollArea::both().show(ui, |ui| {
        //             bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);
        //             ui.allocate_space(ui.available_size());
        //         });
        //     });
    }
}
