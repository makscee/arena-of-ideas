use super::*;

pub struct AdminPlugin;

impl Plugin for AdminPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::update);
    }
}

impl AdminPlugin {
    fn setup_battle(world: &mut World) {
        let (left, right) = client_state().get_battle_test_teams();
        dbg!(&left);
        let mut left = Team::from_tnodes(left[0].id, &left).unwrap_or_default();
        let mut right = Team::from_tnodes(right[0].id, &right).unwrap_or_default();
        let mut battle_world = World::new();
        left.houses = all().core.clone();
        right.houses = all().core.clone();
        let entity_left = battle_world.spawn_empty().id();
        let entity_right = battle_world.spawn_empty().id();
        left.unpack(entity_left, &mut battle_world);
        right.unpack(entity_right, &mut battle_world);
        let mut slot_entities: [Vec<Option<Entity>>; 2] = default();
        slot_entities[0] = (0..global_settings().team_slots).map(|_| None).collect();
        slot_entities[1] = slot_entities[0].clone();
        for fusion in battle_world.query::<&Fusion>().iter(&battle_world) {
            let team = fusion.find_up::<Team>(&battle_world).unwrap();
            slot_entities[(team.entity() == entity_left) as usize][fusion.slot as usize] =
                Some(fusion.entity());
        }
        let mut editing_entity = None;
        Window::new("Edit Teams", move |ui, world| {
            ui.horizontal(|ui| {
                if "Save".cstr_s(CstrStyle::Heading2).button(ui).clicked() {
                    let mut cs = client_state().clone();
                    cs.battle_test_teams.0 = Team::pack(entity_left, &battle_world)
                        .unwrap()
                        .to_tnodes()
                        .into_iter()
                        .map(|n| n.to_ron())
                        .collect();
                    cs.battle_test_teams.1 = Team::pack(entity_right, &battle_world)
                        .unwrap()
                        .to_tnodes()
                        .into_iter()
                        .map(|n| n.to_ron())
                        .collect();
                    cs.save();
                    WindowPlugin::close_current(world);
                    return;
                }
            });
            let slots = global_settings().team_slots as usize;
            #[derive(Resource)]
            struct EditedFusion {
                fusion: Fusion,
            }
            if let Some(ef) = world.remove_resource::<EditedFusion>() {
                ef.fusion.unpack(editing_entity.unwrap(), &mut battle_world);
            }
            for side in [true, false] {
                for i in 0..slots {
                    let entity = slot_entities[side as usize][i];
                    let r = show_battle_slot(i + 1, slots, side, ui);
                    if r.clicked() {
                        todo!();
                    }
                    if let Some(entity) = entity {
                        let rect = r.rect;
                        let fusion = battle_world.get::<Fusion>(entity).unwrap();
                        fusion.paint(rect, ui, &battle_world).log();
                        let context = &Context::new_world(&battle_world).set_owner(entity).take();
                        let rep = battle_world.get::<Representation>(entity).unwrap();
                        rep.paint(rect, context, ui).log();
                    }
                }
            }
        })
        .default_width(800.0)
        .default_height(200.0)
        .push(world);
    }
    fn show_battle(world: &mut World) {
        let (left, right) = client_state().get_battle_test_teams();
        let left = Team::from_tnodes(left[0].id, &left).unwrap_or_default();
        let right = Team::from_tnodes(right[0].id, &right).unwrap_or_default();
        let b = Battle { left, right };
        b.open_window(world);
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
        let mut vars: Vec<(VarName, Expression)> = default();
        Window::new("Anim Editor", move |ui, _| {
            let mut reload = false;
            ui.horizontal(|ui| {
                DragValue::new(&mut size).prefix("size: ").ui(ui);
            });
            if "+".cstr().button(ui).clicked() {
                vars.push(default());
            }
            ui.horizontal(|ui| {
                for (var, value) in &mut vars {
                    var.show_mut(None, ui);
                    value.show_mut(None, ui);
                }
            });
            let mut query = world.query::<(Entity, &Representation)>();
            let mut context = Context::new_world(&world)
                .set_t(t)
                .set_var(VarName::position, default())
                .set_var(VarName::extra_position, vec2(1.0, 0.0).into())
                .take();
            for (var, value) in &vars {
                context.set_var(*var, value.get_value(&context).unwrap_or_default());
            }
            ui.horizontal_centered(|ui| {
                let (rect, resp) = ui.allocate_exact_size(egui::Vec2::splat(size), Sense::hover());
                gt().pause(resp.hovered());
                t += gt().last_delta();
                ui.painter().add(
                    Frame::new()
                        .stroke(if resp.hovered() {
                            STROKE_YELLOW
                        } else {
                            STROKE_BG_DARK
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
    pub fn tab(ui: &mut Ui, world: &mut World) {
        if "Open Battle".cstr().button(ui).clicked() {
            Self::show_battle(world);
        }
        if "Setup Battle".cstr().button(ui).clicked() {
            Self::setup_battle(world);
        }
        if "Anim Editor".cstr().button(ui).clicked() {
            Self::show_anim_editor(world);
        }
        if "Insert Match".cstr().button(ui).clicked() {
            cn().reducers.match_insert().unwrap();
        }
        if "Houses Editor".cstr().button(ui).clicked() {
            GameAssetsEditor::open_houses_window(world);
        }
        if "Update Core".cstr().button(ui).clicked() {
            cn().reducers.incubator_update_core().unwrap();
        }
        if "Export All".cstr().button(ui).clicked() {
            let all = All::pack(world.get_name_link("all").unwrap(), world).unwrap();
            dbg!(&all);
            let path = "./assets/";
            let dir = all.to_dir("ron".into());
            let dir = dir.as_dir().unwrap();
            std::fs::create_dir_all(format!("{path}{}", dir.path().to_str().unwrap())).unwrap();
            dir.extract(path).unwrap();
        }
        if "Export Incubator Data".cstr().button(ui).clicked() {
            GameAssets::update_files();
        }
        let r = "Context Test".cstr().button(ui);
        ContextMenu::new(r)
            .add("test1", |ui, _| {
                debug!("test1");
            })
            .add("test2", |ui, _| {
                debug!("test2");
            })
            .add("test3", |ui, _| {
                debug!("test3");
            })
            .ui(ui, world);
        if "Notification Test ❌".cstr().button(ui).clicked() {
            NotificationsPlugin::test(world);
        }
        if "Incubator".cstr().button(ui).clicked() {
            GameState::Incubator.set_next(world);
        }
    }
    fn update(world: &mut World) {
        // let egui_context = world
        //     .query_filtered::<&mut EguiContext, With<bevy::window::PrimaryWindow>>()
        //     .get_single(world);

        // let Ok(egui_context) = egui_context else {
        //     return;
        // };
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
