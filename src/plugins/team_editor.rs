use super::*;

pub struct TeamEditorPlugin;

#[derive(Resource)]
struct TeamEditorData {
    world: World,
    add_unit:
        HashMap<String, Box<dyn Fn(&mut Ui, &mut World) -> Option<House> + 'static + Send + Sync>>,
    on_save: Option<Box<dyn Fn(Team, &mut World) + 'static + Send + Sync>>,
}

fn rm(world: &mut World) -> Result<Mut<TeamEditorData>, ExpressionError> {
    world
        .get_resource_mut::<TeamEditorData>()
        .to_e("Team not loaded")
}
impl TeamEditorPlugin {
    pub fn unit_add_fn(
        name: String,
        f: impl Fn(&mut Ui, &mut World) -> Option<House> + 'static + Send + Sync,
        world: &mut World,
    ) -> Result<(), ExpressionError> {
        rm(world)?.add_unit.insert(name, Box::new(f));
        Ok(())
    }
    pub fn unit_add_from_core(world: &mut World) -> Result<(), ExpressionError> {
        Self::unit_add_fn(
            "from core".into(),
            |ui, world| {
                let context = &Context::new(world);
                for unit in context.children_components_recursive::<Unit>(core(context).entity()) {
                    let color = context
                        .clone()
                        .set_owner(unit.entity())
                        .get_color(VarName::color)
                        .ok_log()?;
                    if unit.unit_name.cstr_c(color).button(ui).clicked() {
                        return unit.clone().to_house(context).ok_log();
                    }
                }
                None
            },
            world,
        )
    }
    pub fn on_save_fn(
        f: impl Fn(Team, &mut World) + 'static + Send + Sync,
        world: &mut World,
    ) -> Result<(), ExpressionError> {
        rm(world)?.on_save = Some(Box::new(f));
        Ok(())
    }
    pub fn add_panes() {
        TilePlugin::add_to_current(|tree| {
            if let Some(id) = tree.tiles.find_pane(&Pane::Team(TeamPane::Roster)) {
                tree.tiles.remove(id);
            }
            if let Some(id) = tree.tiles.find_pane(&Pane::Team(TeamPane::Slots)) {
                tree.tiles.remove(id);
            }
            let roster = tree.tiles.insert_pane(Pane::Team(TeamPane::Roster));
            let slots = tree.tiles.insert_pane(Pane::Team(TeamPane::Slots));
            tree.tiles.insert_vertical_tile([slots, roster].into())
        });
    }
    pub fn load_team(team: Team, world: &mut World) {
        let mut team_world = World::new();
        team.unpack(team_world.spawn_empty().id(), &mut team_world);
        world.insert_resource(TeamEditorData {
            world: team_world,
            add_unit: default(),
            on_save: None,
        });
    }
    pub fn add_roster_unit(mut house: House, world: &mut World) -> Result<(), ExpressionError> {
        let world = &mut rm(world)?.world;
        let team = world.query::<&Team>().single(world).entity();
        if house.units.is_empty() {
            return Err("No units in House".into());
        }
        let context = Context::new(world);
        if let Some(team_house) = context
            .children_components::<House>(team)
            .into_iter()
            .find(|h| h.house_name == house.house_name)
        {
            let entity = team_house.entity();
            let mut unit = house.units.remove(0);
            unit.clear_ids();
            unit.unpack(world.spawn_empty().set_parent(entity).id(), world);
        } else {
            house.clear_ids();
            house.unpack(world.spawn_empty().set_parent(team).id(), world);
        }
        Ok(())
    }
    fn add_slot_unit(entity: Entity, slot: i32, world: &mut World) -> Result<(), ExpressionError> {
        let world = &mut rm(world)?.world;
        let team = world.query::<&Team>().single(world).entity();
        let context = Context::new(world);
        let unit = world
            .get::<Unit>(entity)
            .to_e("Failed to find Unit")?
            .unit_name
            .clone();
        let fusion = context
            .children_components::<Fusion>(team)
            .into_iter()
            .find(|f| f.slot == slot)
            .map(|f| f.entity())
            .unwrap_or_else(|| {
                let f = Fusion { slot, ..default() };
                let entity = world.spawn_empty().set_parent(team).id();
                f.unpack(entity, world);
                entity
            });
        let mut fusion = world.get_mut::<Fusion>(fusion).unwrap();
        fusion.units.push(unit);
        Ok(())
    }
    pub fn pane_roster(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let Some(mut ed) = world.remove_resource::<TeamEditorData>() else {
            "No team loaded".cstr_c(RED).label(ui);
            return Ok(());
        };

        let team_world = &mut ed.world;
        let team = team_world.query::<&Team>().single(team_world).entity();
        if !ed.add_unit.is_empty() {
            ui.menu_button("add unit", |ui| {
                for (btn, f) in &ed.add_unit {
                    ui.menu_button(btn, |ui| {
                        if let Some(house) = f(ui, world) {
                            op(move |world| {
                                Self::add_roster_unit(house, world).notify(world);
                            });
                            ui.close_menu();
                        }
                    });
                }
            });
        }
        let context = &team_world.into();
        if let Some(f) = &ed.on_save {
            if "save".cstr().button(ui).clicked() {
                let team = Team::pack(team, context).to_e("Failed to pack team")?;
                f(team, world);
            }
        }
        Team::get(team, context)
            .unwrap()
            .view(ViewContext::new(ui), context, ui);
        for house in context.children_components::<House>(team) {
            let color = house
                .color_load(&context)
                .to_e("House color not found")?
                .color
                .c32();
            ui.collapsing(
                house
                    .house_name
                    .cstr_cs(color, CstrStyle::Bold)
                    .widget(1.0, ui.style()),
                |ui| {
                    for unit in context.children_components::<Unit>(house.entity()) {
                        unit.view(ViewContext::new(ui), &context, ui);
                    }
                },
            );
        }

        world.insert_resource(ed);
        Ok(())
    }
    pub fn pane_slots(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let Some(mut ed) = world.remove_resource::<TeamEditorData>() else {
            "No team loaded".cstr_c(RED).label(ui);
            return Ok(());
        };

        let team_world = &mut ed.world;
        let team = team_world.query::<&Team>().single(team_world).entity();
        let slots = global_settings().team_slots as usize;
        for slot in 0..slots {
            let resp = show_slot(slot, slots, false, ui);
            let slot = slot as i32;
            let fusion = Fusion::find_by_slot(slot, team_world);
            let context = Context::new(team_world);
            resp.bar_menu(|ui| {
                ui.menu_button("add unit", |ui| {
                    let units = context.children_components_recursive::<Unit>(team);
                    for unit in units {
                        let entity = unit.entity();
                        ui.horizontal(|ui| {
                            if let Err(e) = unit.show_tag(context.clone().set_owner(entity), ui) {
                                e.cstr().label(ui);
                            } else {
                                if "add".cstr().button(ui).clicked() {
                                    op(move |world| {
                                        Self::add_slot_unit(entity, slot, world).notify(world);
                                    });
                                    ui.close_menu();
                                }
                            }
                        });
                    }
                });
                if let Some(fusion) = &fusion {
                    ui.menu_button("edit", |ui| {
                        let mut fusion = fusion.clone();
                        let entity = fusion.entity();

                        // if fusion.show_editor(&context, ui).unwrap_or_default() {
                        //     op(move |world| {
                        //         *world.get_mut::<Fusion>(entity).unwrap() = fusion;
                        //     });
                        // }
                    });
                }
            });
            if let Some(fusion) = &fusion {
                if let Err(e) = fusion.paint(resp.rect, &context, ui) {
                    let ui = &mut ui.new_child(UiBuilder::new().max_rect(resp.rect));
                    ui.horizontal_centered(|ui| {
                        e.cstr_s(CstrStyle::Bold).label(ui);
                    });
                }
            }
        }

        world.insert_resource(ed);
        Ok(())
    }
}
