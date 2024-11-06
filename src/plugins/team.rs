use egui::TextBuffer;

use super::*;

pub struct TeamPlugin;

#[derive(Resource)]
struct TeamResource {
    entities: HashMap<Faction, Entity>,
    table: Vec<TTeam>,
    new_team_name: String,
    team: u64,
}

#[derive(Component)]
pub struct Team;

impl Plugin for TeamPlugin {
    fn build(&self, app: &mut App) {
        let teams =
            HashMap::from_iter(Faction::iter().map(|f| (f, Self::spawn(f, &mut app.world_mut()))));
        app.insert_resource(TeamResource {
            entities: teams,
            table: default(),
            new_team_name: default(),
            team: 0,
        });
    }
}

impl TeamPlugin {
    pub fn team_entity_faction(entity: Entity, world: &World) -> Option<Faction> {
        world
            .resource::<TeamResource>()
            .entities
            .iter()
            .find_map(|(k, v)| match entity.eq(v) {
                true => Some(*k),
                false => None,
            })
    }
    pub fn unit_faction(entity: Entity, world: &World) -> Faction {
        let team = entity.get_parent(world).unwrap();
        for (faction, entity) in world.resource::<TeamResource>().entities.iter() {
            if team.eq(entity) {
                return *faction;
            }
        }
        panic!("No team found for {entity}")
    }
    pub fn despawn(faction: Faction, world: &mut World) {
        debug!("Despawn team {faction}");
        world
            .entity_mut(Self::entity(faction, world))
            .despawn_recursive();
        let entity = Self::spawn(faction, world);
        world
            .resource_mut::<TeamResource>()
            .entities
            .insert(faction, entity);
    }
    pub fn entity(faction: Faction, world: &World) -> Entity {
        *world
            .resource::<TeamResource>()
            .entities
            .get(&faction)
            .with_context(|| format!("Team not spawned {faction}"))
            .unwrap()
    }
    fn spawn(faction: Faction, world: &mut World) -> Entity {
        let team = world
            .spawn((
                VarState::default()
                    .init(VarName::Faction, VarValue::Faction(faction))
                    .init(VarName::Name, VarValue::String(format!("Team {faction}")))
                    .take(),
                Team,
                TransformBundle::default(),
                VisibilityBundle::default(),
                AbilityStates::default(),
            ))
            .id();
        team
    }
    pub fn get_ability_state<'a>(
        ability: &str,
        faction: Faction,
        world: &'a World,
    ) -> Option<&'a VarState> {
        let entity = Self::entity(faction, world);
        world.get::<AbilityStates>(entity).unwrap().0.get(ability)
    }
    pub fn change_ability_var_int(
        ability: String,
        var: VarName,
        delta: i32,
        faction: Faction,
        world: &mut World,
    ) {
        let d = GameAssets::ability_default(&ability, var);
        let entity = Self::entity(faction, world);
        let mut states = world.get_mut::<AbilityStates>(entity).unwrap();
        let state = states.0.entry(ability).or_default();
        if !state.has_value(var) {
            state.init(var, d);
        }
        state.change_int(var, delta);
    }

    fn load_teams_table(world: &mut World) {
        world.resource_mut::<TeamResource>().table = TTeam::filter_by_owner(player_id())
            .filter(|t| t.pool.eq(&TeamPool::Owned))
            .collect_vec();
    }
    fn load_editor_team(id: u64, world: &mut World) {
        Self::despawn(Faction::Team, world);
        if id == 0 {
            error!("Wrong team id");
            return;
        }
        world.resource_mut::<TeamResource>().team = id;
        TeamSyncPlugin::unsubscribe_all(world);
        TeamSyncPlugin::subscribe(id, Faction::Team, world);
    }
    fn open_new_team_popup(world: &mut World) {
        Confirmation::new("New team".cstr_cs(VISIBLE_LIGHT, CstrStyle::Heading2))
            .accept(|world| {
                let name = world.resource_mut::<TeamResource>().new_team_name.take();
                if name.is_empty() {
                    Notification::new("Empty name".cstr()).error().push(world);
                    Self::open_new_team_popup(world);
                } else {
                    team_create(name);
                    once_on_team_create(|_, _, status, _| {
                        status.on_success(|world| Self::load_teams_table(world));
                    });
                }
            })
            .cancel(|_| {})
            .content(|ui, world| {
                ui.vertical_centered_justified(|ui| {
                    Input::new("name")
                        .char_limit(20)
                        .ui_string(&mut world.resource_mut::<TeamResource>().new_team_name, ui);
                });
            })
            .push(world);
    }

    pub fn teams_tiles(world: &mut World) {
        Self::load_teams_table(world);
        Tile::new(Side::Left, |ui, world| {
            title("Team Manager", ui);
            let cost = GlobalSettings::current().create_team_cost;
            ui.vertical_centered_justified(|ui| {
                if Button::click("New Team")
                    .credits_cost(cost)
                    .ui(ui)
                    .clicked()
                {
                    Self::open_new_team_popup(world);
                }
            });
            let data = mem::take(&mut world.resource_mut::<TeamResource>().table);
            data.show_modified_table("Teams", ui, world, |t| {
                t.column_btn("edit", |d, _, world| {
                    Self::edit_team(d.id, world);
                })
            });
            world.resource_mut::<TeamResource>().table = data;
        })
        .pinned()
        .push(world);
    }

    fn edit_team(id: u64, world: &mut World) {
        Self::load_editor_team(id, world);
        const EDITOR_TILE: &str = "team_editor";
        TilePlugin::close(EDITOR_TILE, world);
        Tile::new(Side::Top, move |ui, world| {
            if id == 0 {
                return;
            }
            let Some(team) = TTeam::find_by_id(id) else {
                return;
            };
            title(&team.name, ui);
            TeamContainer::new(Faction::Team)
                .empty_slot_text("+1/+1\nto all".cstr_cs(VISIBLE_DARK, CstrStyle::Bold))
                .top_content(|ui, world| {
                    ui.horizontal(|ui| {
                        if Button::click("Rename").ui(ui).clicked() {
                            Confirmation::new(
                                "Rename team".cstr_cs(VISIBLE_LIGHT, CstrStyle::Heading2),
                            )
                            .accept(|world| {
                                let mut tr = world.resource_mut::<TeamResource>();
                                let name = tr.new_team_name.take();
                                team_rename(tr.team, name);
                                once_on_team_rename(|_, _, status, _, _| {
                                    status.on_success(|world| Self::load_teams_table(world));
                                });
                            })
                            .cancel(|_| {})
                            .content(|ui, world| {
                                ui.vertical_centered_justified(|ui| {
                                    Input::new("name").char_limit(20).ui_string(
                                        &mut world.resource_mut::<TeamResource>().new_team_name,
                                        ui,
                                    );
                                });
                            })
                            .push(world);
                        }
                        if Button::click("Disband").red(ui).ui(ui).clicked() {
                            Confirmation::new("Disband team?".cstr_c(VISIBLE_LIGHT))
                                .accept(|world| {
                                    let tr = world.resource_mut::<TeamResource>();
                                    team_disband(tr.team);
                                    once_on_team_disband(|_, _, status, id| {
                                        let id = *id;
                                        status.on_success(move |world| {
                                            format!("Team#{id} disbanded").notify(world);
                                            MetaPlugin::clear(world);
                                            MetaPlugin::load_mode(MetaMode::Teams, world);
                                        })
                                    });
                                })
                                .cancel(|_| {})
                                .push(world);
                        }
                    });
                })
                .on_click(|_, e, world| {
                    if e.is_some() {
                        return;
                    }
                    Confirmation::new("Add unit to team".cstr())
                        .cancel(|_| {})
                        .content(|ui, world| {
                            let units = TUnitItem::filter_by_owner(player_id())
                                .map(|u| u.unit)
                                .collect_vec();
                            units.show_modified_table("Units", ui, world, |t| {
                                t.column_btn("select", |u, _, world| {
                                    let tr = world.resource::<TeamResource>();
                                    team_add_unit(tr.team, u.id);
                                    once_on_team_add_unit(|_, _, status, _, _| {
                                        status.on_success(|world| {
                                            Self::load_teams_table(world);
                                            Confirmation::close_current(world);
                                        })
                                    });
                                })
                            });
                        })
                        .push(world);
                })
                .slot_content(|slot, entity, ui, world| {
                    if entity.is_none() {
                        return;
                    }
                    ui.vertical_centered_justified(|ui| {
                        if Button::click("Remove").ui(ui).clicked() {
                            let tr = world.resource::<TeamResource>();
                            team_remove_unit(tr.team, tr.team.get_team(ctx).units[slot].id);
                            once_on_team_remove_unit(|_, _, status, _, _| {
                                status.on_success(|w| {
                                    "Unit removed".notify(w);
                                    Self::load_teams_table(w);
                                });
                            });
                        }
                    });
                })
                .on_swap(|from, to, world| {
                    let team = world.resource::<TeamResource>().team;
                    team_swap_units(team, from as u8, to as u8);
                    once_on_team_swap_units(|_, _, status, _, _, _| status.notify_error());
                })
                .ui(ui, world);
        })
        .pinned()
        .transparent()
        .with_id(EDITOR_TILE.into())
        .min_space(egui::vec2(200.0, 200.0))
        .push(world);
    }
}
