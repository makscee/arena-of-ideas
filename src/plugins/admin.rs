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
        Window::new("UnitCard", |ui, _| {
            UnitCard {
                name: "Apprentice".into(),
                description: "[yellow Battle Start]: deal [vb [b 1]] damage to random enemy".into(),
                house: "wizards".into(),
                house_color: Color32::from_hex("#039BE5").unwrap(),
                rarity: Rarity::legendary,
                vars: HashMap::from_iter([
                    (VarName::pwr, 1.into()),
                    (VarName::hp, 3.into()),
                    (VarName::lvl, 3.into()),
                    (VarName::tier, 2.into()),
                ]),
                expanded: false,
                reaction: Reaction::new(
                    [(
                        Trigger::BattleStart,
                        [
                            Action::AddTarget(Box::new(Expression::RandomUnit(Box::new(
                                Expression::AllEnemyUnits,
                            )))),
                            Action::UseAbility,
                        ]
                        .to_vec()
                        .into(),
                    )]
                    .to_vec(),
                ),
            }
            .show(ui)
        })
        .no_frame()
        .center_anchor()
        .push(world);
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
    fn test_team(units: Vec<String>) -> Team {
        todo!()
    }
    fn setup_battle(world: &mut World) {
        let (left, right) = client_state().battle_test_teams.clone();
        let mut left = Team::from_strings(0, &left).unwrap_or_default();
        let mut right = Team::from_strings(0, &right).unwrap_or_default();
        let mut battle_world = World::new();
        left.houses = houses().values().cloned().collect_vec();
        let entity_left = battle_world.spawn_empty().id();
        left.unpack(entity_left, &mut battle_world);
        let mut editing_entity = None;
        Window::new("Edit Teams", move |ui, world| {
            let slots = global_settings().team_slots as usize;
            #[derive(Resource)]
            struct EditedFusion {
                fusion: Fusion,
            }
            if let Some(ef) = world.remove_resource::<EditedFusion>() {
                ef.fusion.unpack(editing_entity.unwrap(), &mut battle_world);
                Fusion::init_all(&mut battle_world).log();
            }
            let mut slot_fusions: Vec<Option<Fusion>> = (0..5).map(|_| None).collect_vec();
            for (fusion, slot) in battle_world
                .query::<(&Fusion, &UnitSlot)>()
                .iter(&battle_world)
            {
                slot_fusions[slot.slot as usize] = Some(fusion.clone());
            }
            for i in 0..slots {
                let r = show_battle_slot(i + 1, slots, true, ui);
                if r.clicked() {
                    let entity = if let Some(fusion) = &slot_fusions[i] {
                        fusion.entity()
                    } else {
                        let entity = battle_world.spawn_empty().set_parent(entity_left).id();
                        Fusion::new_full(default(), default(), UnitSlot::new(i as i32))
                            .unpack(entity, &mut battle_world);
                        Fusion::init_all(&mut battle_world).log();
                        entity
                    };
                    editing_entity = Some(entity);
                    Fusion::open_editor_window(entity, world, &battle_world, |fusion, world| {
                        world.insert_resource(EditedFusion { fusion });
                    })
                    .unwrap();
                }
                if let Some(fusion) = &slot_fusions[i] {
                    let rect = r.rect;
                    fusion.paint(rect, ui, &battle_world).log();
                    let context = &Context::new_world(&battle_world)
                        .set_owner(fusion.entity())
                        .take();
                    let rep = battle_world.get::<Representation>(fusion.entity()).unwrap();
                    rep.paint(rect, context, ui).log();
                }
            }
        })
        .default_width(600.0)
        .push(world);
    }
    fn show_battle(world: &mut World) {
        let b = Battle {
            left: Self::test_team(["fortifier".into(), "priest".into()].into()),
            right: Self::test_team(["chrism".into(), "priest".into()].into()),
        };
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
                if "Setup Battle".cstr().button(ui).clicked() {
                    Self::setup_battle(world);
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
                if "Push Battle".cstr().button(ui).clicked() {
                    let team = Self::test_team(vec!["priest".into()]);
                    let team = team.to_strings_root();
                    cn().reducers.battle_insert(team.clone(), team).unwrap();
                }
                if "Insert Match".cstr().button(ui).clicked() {
                    cn().reducers.match_insert().unwrap();
                }
                if "Open Match".cstr().button(ui).clicked() {
                    if let Some(m) = NodeDomain::Match.filter_by_kind(NodeKind::Match).get(0) {
                        MatchPlugin::load_match_data(m.id, world);
                        MatchPlugin::open_shop_window(world);
                    } else {
                        error!("No matches found");
                    }
                }
                if "Houses Editor".cstr().button(ui).clicked() {
                    GameAssetsEditor::open_houses_window(world);
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
