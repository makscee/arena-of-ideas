use super::*;

pub struct BattlePlugin;

#[derive(Resource)]
struct BattleData {
    battle: Battle,
    simulation: BattleSimulation,
    t: f32,
    playing: bool,
}

fn rm(world: &mut World) -> Mut<BattleData> {
    world.resource_mut::<BattleData>()
}

impl BattlePlugin {
    pub fn load_incubator(world: &mut World) -> Result<(), ExpressionError> {
        let mut team = Team::default();
        team.houses = IncubatorPlugin::world_op(world, |world| {
            world
                .query::<&House>()
                .iter(world)
                .filter_map(|h| House::pack(h.entity(), world))
                .collect_vec()
        });
        let battle = Battle {
            left: team.clone(),
            right: team.clone(),
        };
        world.insert_resource(BattleData {
            simulation: BattleSimulation::new(battle.clone()),
            battle,
            t: 0.0,
            playing: false,
        });
        Ok(())
    }
    fn reload_simulation(world: &mut World) {
        let mut data = rm(world);
        data.simulation = BattleSimulation::new(data.battle.clone());
    }
    pub fn add_panes() {
        TilePlugin::op(|tree| {
            if let Some(id) = tree.tiles.find_pane(&Pane::BattleView) {
                tree.tiles.remove(id);
            }
            if let Some(id) = tree.tiles.find_pane(&Pane::BattleControls) {
                tree.tiles.remove(id);
            }
            let view = tree.tiles.insert_pane(Pane::BattleView);
            let controls = tree.tiles.insert_pane(Pane::BattleControls);
            let id = tree.tiles.insert_vertical_tile([view, controls].into());
            tree.add_to_root(id).log();
        });
    }
    pub fn pane_view(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let Some(mut data) = world.remove_resource::<BattleData>() else {
            "[red No battle loaded]".cstr().label(ui);
            return Ok(());
        };

        let t = data.t;
        data.simulation.show_at(
            t,
            ui,
            world,
            |slot, player_side, resp, simulation, ui, world| {
                resp.bar_menu(|ui| {
                    ui.menu_button("add unit", |ui| {
                        IncubatorPlugin::world_op(world, |world| {
                            let team = if player_side {
                                simulation.team_left
                            } else {
                                simulation.team_right
                            };
                            let context =
                                Context::new_world(&simulation.world).set_owner(team).take();
                            let units = context.children_components_recursive::<Unit>(team);
                            for (entity, _) in units {
                                let context = Context::new_world(&simulation.world)
                                    .set_owner(entity)
                                    .take();
                                if let Ok(name) =
                                    context.get_string(VarName::name).and_then(|name| {
                                        context.get_color(VarName::color).map(|c| (name.cstr_c(c)))
                                    })
                                {
                                    if name.cstr().button(ui).clicked() {
                                        OperationsPlugin::add(move |world| {
                                            Self::add_incubator_unit(
                                                slot,
                                                player_side,
                                                name.get_text(),
                                                world,
                                            );
                                        });
                                        ui.close_menu();
                                    }
                                }
                            }
                        });
                        Ok(())
                    })
                    .inner
                    .unwrap_or(Ok(()))
                    .log();
                });
            },
        );

        world.insert_resource(data);
        Ok(())
    }
    pub fn pane_controls(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let Some(mut data) = world.get_resource_mut::<BattleData>() else {
            "[red No battle loaded]".cstr().label(ui);
            return Ok(());
        };
        let BattleData {
            battle: _,
            simulation,
            t,
            playing,
        } = data.as_mut();
        if simulation.duration > 0.0 {
            Slider::new("ts")
                .full_width()
                .ui(t, 0.0..=simulation.duration, ui);
        }
        ui.horizontal(|ui| {
            Checkbox::new(playing, "play").ui(ui);
            if "+1".cstr().button(ui).clicked() {
                simulation.run();
            }
            if "+10".cstr().button(ui).clicked() {
                for _ in 0..10 {
                    simulation.run();
                }
            }
            if "+100".cstr().button(ui).clicked() {
                for _ in 0..100 {
                    simulation.run();
                }
            }
        });
        if *playing {
            *t += gt().last_delta();
            *t = t.at_most(simulation.duration);
        }
        if *t >= simulation.duration && !simulation.ended() {
            simulation.run();
        }
        Ok(())
    }
    fn add_incubator_unit(mut slot: usize, player_side: bool, unit: String, world: &mut World) {
        let mut data = rm(world);
        let team = if player_side {
            &mut data.battle.left
        } else {
            &mut data.battle.right
        };
        if slot > team.fusions.len() {
            slot = team.fusions.len();
            team.fusions.push(Fusion {
                slot: slot as i32,
                ..default()
            });
        } else {
            slot = slot - 1;
        }
        team.fusions[slot].units.push(unit);
        Self::reload_simulation(world);
    }
}
