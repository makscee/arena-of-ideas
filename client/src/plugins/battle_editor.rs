use crate::prelude::*;

pub struct BattleEditorPlugin;

impl Plugin for BattleEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Editor), Self::load_from_client_state);
    }
}

#[derive(Resource, Clone)]
pub struct BattleEditorState {
    pub left_team: NTeam,
    pub right_team: NTeam,
    pub was_playing: bool,
}

impl BattleEditorPlugin {
    pub fn pane_view(ui: &mut Ui, world: &mut World) {
        world.resource_scope(|_, mut battle_data: Mut<BattleData>| {
            BattleCamera::builder()
                .filled_slot_action(
                    "Inspect Unit".to_string(),
                    |team_id, unit_id, _slot, _ctx, ui| {
                        ui.set_inspected_node_for_parent(team_id, unit_id);
                    },
                )
                .filled_slot_action(
                    "Delete Unit".to_string(),
                    |team_id, unit_id, slot_index, _ctx, _ui| {
                        op(move |world| {
                            let mut state = world.resource_mut::<BattleEditorState>();
                            let team = if team_id == state.left_team.id {
                                &mut state.left_team
                            } else {
                                &mut state.right_team
                            };

                            // Remove from slot
                            if let Ok(slots) = team.slots.get_mut() {
                                if let Some(slot) = slots.iter_mut().find(|s| s.index == slot_index)
                                {
                                    slot.unit = Owned::default();
                                }
                            }

                            // Remove from houses
                            if let Ok(houses) = team.houses.get_mut() {
                                for house in houses {
                                    if let RefMultiple::Ids { node_ids, .. } = &mut house.units {
                                        node_ids.retain(|&id| id != unit_id);
                                    }
                                }
                            }

                            BattleEditorPlugin::save_changes_and_reload(world);
                        })
                    },
                )
                .empty_slot_action(
                    "Add Placeholder Unit".to_string(),
                    |team_id, slot_index, _ctx, _ui| {
                        op(move |world| {
                            let mut state = world.resource_mut::<BattleEditorState>();
                            let team = if team_id == state.left_team.id {
                                &mut state.left_team
                            } else {
                                &mut state.right_team
                            };

                            let mut unit = NUnit::placeholder();
                            unit.state
                                .set_loaded(NUnitState::new(next_id(), player_id(), 1, 0));

                            // Add to first house or create one
                            if let Ok(houses) = team.houses.get_mut() {
                                if houses.is_empty() {
                                    let mut house = NHouse::placeholder();
                                    house.units = RefMultiple::Ids {
                                        parent_id: house.id,
                                        node_ids: vec![unit.id],
                                    };
                                    houses.push(house);
                                } else if let Some(house) = houses.first_mut() {
                                    if let RefMultiple::Ids { node_ids, .. } = &mut house.units {
                                        node_ids.push(unit.id);
                                    }
                                }
                            }

                            // Add to slot
                            if let Ok(slots) = team.slots.get_mut() {
                                if let Some(slot) = slots.iter_mut().find(|s| s.index == slot_index)
                                {
                                    slot.unit = Owned::Loaded {
                                        parent_id: team.id,
                                        data: unit,
                                    };
                                } else {
                                    let mut new_slot =
                                        NTeamSlot::new(next_id(), team.id, slot_index);
                                    new_slot.unit = Owned::Loaded {
                                        parent_id: team.id,
                                        data: unit,
                                    };
                                    slots.push(new_slot);
                                }
                            }

                            BattleEditorPlugin::save_changes_and_reload(world);
                        })
                    },
                )
                .on_hover_filled(|unit_id, ctx, ui| {
                    if let Ok(unit) = ctx.load::<NUnit>(unit_id) {
                        unit.as_card().compose(ctx, ui);
                    }
                })
                .on_drag_drop(
                    |dragged_unit_id, from_slot, _target_unit_id, to_slot, _ctx| {
                        if from_slot == to_slot {
                            return false;
                        }
                        op(move |world| {
                            let mut state = world.resource_mut::<BattleEditorState>();
                            // Determine which team(s) are involved
                            let left_has_from =
                                state.left_team.slots.get().unwrap().iter().any(|s| {
                                    if s.index == from_slot {
                                        match &s.unit {
                                            Owned::Loaded { data: u, .. } => {
                                                u.id == dragged_unit_id
                                            }
                                            _ => false,
                                        }
                                    } else {
                                        false
                                    }
                                });
                            let left_has_to = state
                                .left_team
                                .slots
                                .get()
                                .unwrap()
                                .iter()
                                .any(|s| s.index == to_slot);

                            // Perform swap within the appropriate team
                            let team = if left_has_from || left_has_to {
                                &mut state.left_team
                            } else {
                                &mut state.right_team
                            };

                            if let Ok(slots) = team.slots.get_mut() {
                                let from_unit = slots
                                    .iter()
                                    .find(|s| s.index == from_slot)
                                    .and_then(|s| match &s.unit {
                                        Owned::Loaded { data: unit, .. } => Some(unit.clone()),
                                        _ => None,
                                    });
                                let to_unit = slots.iter().find(|s| s.index == to_slot).and_then(
                                    |s| match &s.unit {
                                        Owned::Loaded { data: unit, .. } => Some(unit.clone()),
                                        _ => None,
                                    },
                                );

                                // Swap units
                                if let Some(from_slot_mut) =
                                    slots.iter_mut().find(|s| s.index == from_slot)
                                {
                                    if let Some(to_unit) = to_unit {
                                        from_slot_mut.unit = Owned::Loaded {
                                            parent_id: team.id,
                                            data: to_unit,
                                        };
                                    } else {
                                        from_slot_mut.unit = Owned::default();
                                    }
                                }

                                if let Some(from_unit) = from_unit {
                                    if let Some(to_slot_mut) =
                                        slots.iter_mut().find(|s| s.index == to_slot)
                                    {
                                        to_slot_mut.unit = Owned::Loaded {
                                            parent_id: team.id,
                                            data: from_unit,
                                        };
                                    } else {
                                        // Create new slot if it doesn't exist
                                        let mut new_slot =
                                            NTeamSlot::new(next_id(), team.id, to_slot);
                                        new_slot.unit = Owned::Loaded {
                                            parent_id: team.id,
                                            data: from_unit,
                                        };
                                        slots.push(new_slot);
                                    }
                                }
                            }

                            BattleEditorPlugin::save_changes_and_reload(world);
                        });
                        true
                    },
                )
                .on_battle_end(|ui, result| {
                    ui.vertical_centered_justified(|ui| {
                        if result {
                            "Victory".cstr_cs(GREEN, CstrStyle::Heading).label(ui);
                        } else {
                            "Defeat".cstr_cs(RED, CstrStyle::Heading).label(ui);
                        }
                    });
                })
                .show(&mut battle_data, ui);
        });
    }

    pub fn pane_edit_graph(left: bool, ui: &mut Ui, world: &mut World) {
        ui.horizontal(|ui| {
            let saved_teams = pd().client_state.saved_teams.clone();

            for (name, left_data, right_data) in saved_teams.iter() {
                ui.menu_button(name, |ui| {
                    if ui.button("Load").clicked() {
                        let mut state = world.resource_mut::<BattleEditorState>();
                        let left_team = match NTeam::unpack(left_data) {
                            Ok(v) => v,
                            Err(e) => {
                                e.cstr().notify_error(world);
                                return;
                            }
                        };
                        let right_team = match NTeam::unpack(right_data) {
                            Ok(v) => v,
                            Err(e) => {
                                e.cstr().notify_error(world);
                                return;
                            }
                        };
                        state.left_team = left_team;
                        state.right_team = right_team;
                        BattleEditorPlugin::save_changes_and_reload(world);
                    }
                    if ui.button("Update").clicked() {
                        let state = world.resource_mut::<BattleEditorState>();
                        pd_mut(|pd| {
                            pd.client_state.save_team(
                                name.clone(),
                                &state.left_team,
                                &state.right_team,
                            );
                        });
                    }
                    if "[red Delete]".cstr().button(ui).clicked() {
                        pd_mut(|pd| {
                            pd.client_state.delete_team(name);
                        });
                    }
                });
            }

            if ui.button("+ Add").clicked() {
                let mut name = "name".to_owned();
                let confirmation = Confirmation::new("Save Team")
                    .accept_name("Save")
                    .cancel_name("Cancel")
                    .content(move |ui, world, result| {
                        Input::new("Team Name").ui_string(&mut name, ui);
                        if result == Some(true) {
                            if !name.is_empty() {
                                let state = world.resource::<BattleEditorState>();
                                let name = name.clone();
                                pd_mut(|pd| {
                                    pd.client_state.save_team(
                                        name,
                                        &state.left_team,
                                        &state.right_team,
                                    );
                                });
                            }
                        }
                    });
                confirmation.push(world);
            }
        });

        if let Some(mut state) = world.get_resource_mut::<BattleEditorState>() {
            let needs_reload = if left {
                state.left_team.render_recursive_edit(ui)
            } else {
                state.right_team.render_recursive_edit(ui)
            };

            if needs_reload {
                BattleEditorPlugin::save_changes_and_reload(world);
            }
        } else {
            "[red [b BattleEditorState] not found]".cstr().label(ui);
        }
    }

    pub fn load_from_client_state(world: &mut World) {
        let (left_team, right_team) =
            if let Some((left, right)) = pd().client_state.get_battle_test_teams() {
                (left, right)
            } else {
                (NTeam::placeholder(), NTeam::placeholder())
            };
        world.insert_resource(BattleEditorState {
            left_team,
            right_team,
            was_playing: true,
        });
        Self::save_changes_and_reload(world);
    }

    pub fn save_changes_and_reload(world: &mut World) {
        let was_playing = world
            .get_resource::<BattleData>()
            .map(|bd| bd.playing)
            .or_else(|| {
                world
                    .get_resource::<BattleEditorState>()
                    .map(|s| s.was_playing)
            })
            .unwrap_or(true);

        let state = world.resource_mut::<BattleEditorState>();
        pd_mut(|pd| {
            pd.client_state
                .set_battle_test_teams(&state.left_team, &state.right_team)
        });
        BattlePlugin::load_teams(0, state.left_team.clone(), state.right_team.clone(), world);

        if let Some(mut battle_data) = world.get_resource_mut::<BattleData>() {
            battle_data.playing = was_playing;
        }
        if let Some(mut editor_state) = world.get_resource_mut::<BattleEditorState>() {
            editor_state.was_playing = was_playing;
        }
    }
}
