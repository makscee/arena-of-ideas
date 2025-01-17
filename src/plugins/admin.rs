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
        let unit = Unit {
            name: String::new(),
            stats: Some(UnitStats {
                pwr: 1,
                hp: 10,
                ..default()
            }),
            description: Some(UnitDescription {
                description: "battle start test".into(),
                trigger: Some(UnitTrigger {
                    trigger: Trigger::BattleStart,
                    target: Expression::Owner,
                    effect: Effect::ChangeStatus,
                    ..default()
                }),
                ..default()
            }),
            ..default()
        };
        let b = Battle {
            left: [unit.clone(), unit.clone()].into(),
            right: [unit.clone(), unit.clone()].into(),
        };
        let mut bs = BattleSimulation::new(&b).start();
        let mut t = 0.0;
        let mut playing = false;
        Window::new("Battle", move |ui, _| {
            ui.set_min_size(egui::vec2(800.0, 400.0));
            Slider::new("ts").full_width().ui(&mut t, 0.0..=bs.t, ui);
            Checkbox::new(&mut playing, "play").ui(ui);
            if "+1".cstr().button(ui).clicked() {
                bs.run();
            }
            if "+10".cstr().button(ui).clicked() {
                for _ in 0..10 {
                    bs.run();
                }
            }
            if "+100".cstr().button(ui).clicked() {
                for _ in 0..100 {
                    bs.run();
                }
            }
            if playing {
                t += gt().last_delta();
                t = t.at_most(bs.t);
            }
            bs.show_at(t, ui);
            if t >= bs.t && !bs.ended() {
                bs.run();
            }
        })
        .push(world);
    }
    fn show_anim_editor(w: &mut World) {
        let mut cs = client_state().clone();
        if cs.edit_anim.is_none() {
            let mut anim = Anim::default();
            anim.push(AnimAction::Spawn(Box::new(Material(
                [
                    Box::new(PainterAction::Rectangle(Box::new(Expression::V2(0.5, 0.3)))),
                    Box::new(PainterAction::Rotate(Box::new(Expression::Var(VarName::t)))),
                ]
                .into(),
            ))));
            cs.edit_anim = Some(anim);
        }
        let mut anim = cs.edit_anim.clone().unwrap();
        let mut t = 0.0;
        let mut world = World::new();
        fn respawn(anim: &Anim, world: &mut World) -> Result<f32, ExpressionError> {
            world.clear_all();
            anim.apply(&mut 0.0, Context::default().set_t(0.0).take(), world)
        }
        let mut end_t = respawn(&anim, &mut world).unwrap();
        let mut size = 300.0;
        Window::new("Anim Editor", move |ui, _| {
            let mut reload = false;
            ui.horizontal(|ui| {
                DragValue::new(&mut size).prefix("size: ").ui(ui);
            });
            let mut query = world.query::<(Entity, &Representation)>();
            let context = Context::new_world(&world).set_t(t).take();
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
                for (entity, r) in query.iter(&world) {
                    match RepresentationPlugin::paint_rect(
                        rect,
                        context.clone().set_owner(entity),
                        &r.material,
                        ui,
                    ) {
                        Ok(_) => {}
                        Err(e) => error!("Paint error: {e} {context:?}"),
                    }
                }
                ui.set_clip_rect(cr);
                ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            if anim.show_mut(None, ui) {
                                reload = true;
                                let mut cs = client_state().clone();
                                cs.edit_anim = Some(anim.clone());
                                cs.save();
                            }
                        });
                    });
            });
            reload |= t > end_t;
            if reload {
                if let Ok(end) = respawn(&anim, &mut world) {
                    t = 0.0;
                    end_t = end;
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
                if "Anim Editor".cstr().button(ui).clicked() {
                    Self::show_anim_editor(world);
                }
                if "Spawn Hero".cstr().button(ui).clicked() {
                    match cn().reducers.node_spawn_hero("SpawnedHero".into()) {
                        Ok(_) => {
                            info!("Hero spawned")
                        }
                        Err(e) => {
                            error!("{e}")
                        }
                    };
                }
                // ui.horizontal(|ui| {
                //     e.show_mut(Some("Expr"), ui);
                //     e.show(Some("Prefix"), &Context::default(), ui);
                // });
            })
            .transparent()
            .pinned()
            .push(world);
        });
        // let house = houses().get("holy").unwrap().clone();
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
        // let mut egui_context = egui_context.clone();

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
