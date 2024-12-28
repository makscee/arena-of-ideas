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
        let mut playing = false;
        Window::new("Battle", move |ui, world| {
            ui.set_min_size(egui::vec2(800.0, 400.0));
            Slider::new("ts").full_width().ui(&mut t, 0.0..=bs.t, ui);
            Checkbox::new(&mut playing, "play").ui(ui);
            if playing {
                t += gt().last_delta();
                t = t.at_most(bs.t);
            }
            bs.show_at(t, ui);
        })
        .push(world);
    }
    fn show_vfx_editor(w: &mut World) {
        let mut vfx = Vfx::default();
        vfx.representation = Representation {
            material: Material(vec![Box::new(PainterAction::Rectangle(Box::new(
                Expression::V2(1.0, 0.7),
            )))]),
            children: Vec::new(),
            entity: None,
        };
        vfx.duration = 1.0;
        vfx.timeframe = 1.0;
        let mut t = 0.0;
        let mut world = World::new();
        fn respawn(vfx: &Vfx, world: &mut World) -> Result<Entity, ExpressionError> {
            world.clear_all();
            match vfx.spawn(&mut 0.0, world) {
                Ok(e) => Ok(e),
                Err(e) => Err(e),
            }
        }
        let mut entity = respawn(&vfx, &mut world).unwrap();
        let mut size = 100.0;
        Window::new("Vfx Editor", move |ui, _| {
            let mut reload = false;
            ui.horizontal(|ui| {
                DragValue::new(&mut size).prefix("size: ").ui(ui);
                reload |= DragValue::new(&mut vfx.duration)
                    .prefix("duration: ")
                    .ui(ui)
                    .changed();
                reload |= DragValue::new(&mut vfx.timeframe)
                    .prefix("timeframe: ")
                    .ui(ui)
                    .changed();
            });
            let mut query = world.query::<&Representation>();
            let context = Context::new_world(&world).set_owner(entity).set_t(t).take();
            ui.horizontal_centered(|ui| {
                let (rect, resp) = ui.allocate_exact_size(egui::Vec2::splat(size), Sense::hover());
                gt().pause(resp.hovered());
                t += gt().last_delta();
                ui.painter().add(
                    Frame::none()
                        .stroke(if resp.hovered() {
                            STROKE_YELLOW
                        } else {
                            STROKE_DARK
                        })
                        .paint(rect),
                );
                let cr = ui.clip_rect();
                ui.set_clip_rect(rect.expand(6.0).intersect(cr));
                for r in query.iter(&world) {
                    match RepresentationPlugin::paint_rect(rect, &context, &r.material, ui) {
                        Ok(_) => {}
                        Err(e) => error!("Paint error: {e}"),
                    }
                }
                ui.set_clip_rect(cr);
                ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            if vfx.show_mut(None, ui) {
                                reload = true;
                            }
                        });
                    });
            });
            reload |= t >= vfx.duration;
            if reload {
                if let Ok(e) = respawn(&vfx, &mut world) {
                    t = 0.0;
                    entity = e;
                }
            }
        })
        .push(w);
    }
    fn setup(mut commands: Commands) {
        commands.add(|world: &mut World| {
            let mut e = Expression::F(1.0);
            e.inject_data(
                r#"
Abs(Equals(F(51.0),Abs(Equals(F(1.0),Or(Equals(F(1.0),One),Abs(Or(Target,Abs(One))))))))"#,
            );
            Tile::new(Side::Left, move |ui, world| {
                let scale = &mut world.resource_mut::<CameraData>().need_scale;
                Slider::new("Cam Scale").ui(scale, 1.0..=50.0, ui);
                if "Open Battle".cstr().button(ui).clicked() {
                    Self::show_battle(world);
                }
                if "Vfx Editor".cstr().button(ui).clicked() {
                    Self::show_vfx_editor(world);
                }
                ui.horizontal(|ui| {
                    e.show_mut(Some("Expr"), ui);
                    e.show(Some("Prefix"), &Context::default(), ui);
                });
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
