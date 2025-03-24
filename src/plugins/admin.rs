use rand::seq::SliceRandom;

use super::*;

pub struct AdminPlugin;

impl Plugin for AdminPlugin {
    fn build(&self, _: &mut App) {}
}

impl AdminPlugin {
    fn setup_battle(world: &mut World) {
        let (left, right) = client_state().get_battle_test_teams();
        dbg!(&left);
        let mut left = Team::from_tnodes(left[0].id, &left).unwrap_or_default();
        let mut right = Team::from_tnodes(right[0].id, &right).unwrap_or_default();
        let mut battle_world = World::new();
        left.houses = all(world).core.clone();
        right.houses = all(world).core.clone();
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
                        .stroke(Stroke::new(
                            1.0,
                            if resp.hovered() {
                                tokens_global().hovered_ui_element_border()
                            } else {
                                tokens_global().subtle_borders_and_separators()
                            },
                        ))
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
    pub fn pane(ui: &mut Ui, world: &mut World) {
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
        if "Incubator Merge".cstr().button(ui).clicked() {
            cn().reducers.incubator_merge().unwrap();
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
        if "Load Battle".cstr().button(ui).clicked() {
            BattlePlugin::load_incubator(world).log();
        }
        if "Add Battle Panes".cstr().button(ui).clicked() {
            BattlePlugin::add_panes();
        }
        if "Add Node Graph Pane".cstr().button(ui).clicked() {
            TilePlugin::op(|tree| {
                let id = tree.tiles.insert_pane(Pane::NodeGraph);
                tree.add_to_root(id).unwrap();
            });
        }
        if "Add Team Editor Panes".cstr().button(ui).clicked() {
            TeamEditorPlugin::load_team(default(), world);
            TeamEditorPlugin::add_panes();
        }
        if "Add Unit".cstr().button(ui).clicked() {
            let unit = dbg!(Context::new_world(world)
                .children_components_recursive::<Unit>(all(world).entity())
                .choose(&mut thread_rng()))
            .unwrap()
            .entity();
            TeamEditorPlugin::add_unit(unit, world).log();
        }
        if "Notification Test".cstr().button(ui).clicked() {
            "notify test".notify(world);
            "notify error test".notify_error(world);
        }
        if "Incubator".cstr().button(ui).clicked() {
            GameState::Incubator.set_next(world);
        }
    }
}
