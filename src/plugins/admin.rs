use super::*;

pub struct AdminPlugin;

impl Plugin for AdminPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Admin), Self::on_enter)
            .add_systems(Update, Self::update);
    }
}

impl AdminPlugin {
    fn on_enter(world: &mut World) {
        let house = houses().get("holy").unwrap().clone();
        dbg!(&house);
        house.unpack(world.spawn_empty().id(), &mut world.commands());
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
        for r in world.query::<&Representation>().iter(world) {
            dbg!(r);
        }
        Tile::new(Side::Left, |ui, world| {})
            .pinned()
            .transparent()
            .no_expand()
            .push(world);
    }

    fn update(world: &mut World) {
        let egui_context = world
            .query_filtered::<&mut EguiContext, With<bevy::window::PrimaryWindow>>()
            .get_single(world);

        let Ok(egui_context) = egui_context else {
            return;
        };
        let mut egui_context = egui_context.clone();

        egui::Window::new("World Inspector")
            .default_size(egui::vec2(300.0, 300.0))
            .show(egui_context.get_mut(), |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);
                    ui.allocate_space(ui.available_size());
                });
            });
        for r in world
            .query::<&mut Representation>()
            .iter(world)
            .cloned()
            .collect_vec()
        {
            r.material.update(r.entity.unwrap(), world);
        }
    }
}
