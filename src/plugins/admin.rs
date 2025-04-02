use super::*;

pub struct AdminPlugin;

impl Plugin for AdminPlugin {
    fn build(&self, _: &mut App) {}
}

impl AdminPlugin {
    pub fn pane(ui: &mut Ui, world: &mut World) {
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
        if "Add Node Graph Pane".cstr().button(ui).clicked() {
            TilePlugin::add_to_current(|tree| tree.tiles.insert_pane(Pane::NodeGraph));
        }
        if "Add Team Editor Panes".cstr().button(ui).clicked() {
            TeamEditorPlugin::load_team(default(), world);
            TeamEditorPlugin::add_panes();
            TeamEditorPlugin::unit_add_from_core(world).notify(world);
        }
        if "Notification Test".cstr().button(ui).clicked() {
            "notify test".notify(world);
            "notify error test".notify_error(world);
        }
        if "Incubator".cstr().button(ui).clicked() {
            GameState::Incubator.set_next(world);
        }
    }
    fn show_anim_editor(w: &mut World) {
        let mut cs = pd().client_state.clone();
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
                .set_var_any(VarName::position, default())
                .set_var_any(VarName::extra_position, vec2(1.0, 0.0).into())
                .take();
            for (var, value) in &vars {
                context.set_var_any(*var, value.get_value(&context).unwrap_or_default());
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
                                let mut cs = pd().client_state.clone();
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
}
