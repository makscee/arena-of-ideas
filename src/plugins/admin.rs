use super::*;

pub struct AdminPlugin;

impl Plugin for AdminPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Admin), (Self::setup, Self::on_enter))
            .add_systems(Update, Self::update);
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
    fn show_battle(world: &mut World) {
        let b = Battle {
            left: [
                Unit {
                    stats: Some(UnitStats {
                        pwr: 1,
                        hp: 3,
                        ..default()
                    }),
                    ..default()
                },
                Unit {
                    stats: Some(UnitStats {
                        pwr: 1,
                        hp: 3,
                        ..default()
                    }),
                    ..default()
                },
            ]
            .into(),
            right: [
                Unit {
                    stats: Some(UnitStats {
                        pwr: 1,
                        hp: 4,
                        ..default()
                    }),
                    ..default()
                },
                Unit {
                    stats: Some(UnitStats {
                        pwr: 1,
                        hp: 4,
                        ..default()
                    }),
                    ..default()
                },
            ]
            .into(),
        };
        let mut bs = BattleSimulation::new(&b).run();
        let mut t = 0.0;
        Window::new("Battle", move |ui, world| {
            ui.set_min_size(egui::vec2(800.0, 400.0));
            Slider::new("ts").full_width().ui(&mut t, 0.0..=bs.t, ui);
            bs.show_at(t, ui);
        })
        .push(world);
    }
    fn setup(mut commands: Commands) {
        commands.add(|world: &mut World| {
            Tile::new(Side::Left, |ui, world| {
                let scale = &mut world.resource_mut::<CameraData>().need_scale;
                Slider::new("Cam Scale").ui(scale, 1.0..=50.0, ui);
                if "Open Battle".cstr().button(ui).clicked() {
                    Self::show_battle(world);
                }
            })
            .transparent()
            .pinned()
            .push(world);
        });
        let house = houses().get("holy").unwrap().clone();
        // dbg!(&house);
        // house.unpack(commands.spawn_empty().id(), &mut commands);

        // commands.add(|world: &mut World| {
        //     for e in world
        //         .query_filtered::<Entity, With<House>>()
        //         .iter(world)
        //         .collect_vec()
        //     {
        //         Window::new("Inspector", move |ui, world| {
        //             world.get::<House>(e).unwrap().ui(0, ui, world);
        //         })
        //         .no_frame()
        //         .push(world);
        //     }
        // });
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
