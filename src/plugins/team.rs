use egui::TextBuffer;

use super::*;

pub struct TeamPlugin;

#[derive(Resource)]
struct TeamResource {
    entities: HashMap<Faction, Entity>,
    table: Vec<TTeam>,
    new_team_name: String,
    team: u64,
    unit_to_add: u64,
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
            unit_to_add: 0,
        });
    }
}

impl TeamPlugin {
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
        let d = GameAssets::ability_default(&ability, var, world);
        let entity = Self::entity(faction, world);
        let mut states = world.get_mut::<AbilityStates>(entity).unwrap();
        let state = states.0.entry(ability).or_default();
        if !state.has_value(var) {
            state.init(var, d);
        }
        state.change_int(var, delta);
    }

    fn load_teams_table(world: &mut World) {
        world.resource_mut::<TeamResource>().table = TTeam::filter_by_owner(user_id())
            .filter(|t| t.pool.eq(&TeamPool::Owned))
            .collect_vec();
        TableState::reset_cache(&egui_context(world).unwrap());
    }
    fn load_editor_team(world: &mut World) {
        Self::despawn(Faction::Team, world);
        let team = world.resource::<TeamResource>().team;
        if team == 0 {
            return;
        }
        TeamSyncPlugin::subscribe(team, Faction::Team, world);
    }
    fn open_new_team_popup(world: &mut World) {
        let ctx = egui_context(world).unwrap();
        Confirmation::new(
            "New Team".cstr_cs(VISIBLE_LIGHT, CstrStyle::Heading2),
            |world| {
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
            },
        )
        .content(|ui, world| {
            Input::new("name").ui(&mut world.resource_mut::<TeamResource>().new_team_name, ui);
        })
        .add(&ctx);
    }

    fn teams_tiles(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            title("Team Manager", ui);
            if Button::click("New Team".into()).ui(ui).clicked() {
                Self::open_new_team_popup(world);
            }
            let data = mem::take(&mut world.resource_mut::<TeamResource>().table);
            data.show_modified_table("Teams", ui, world, |t| {
                t.column_btn("edit", |d, _, world| {
                    let mut tr = world.resource_mut::<TeamResource>();
                    tr.team = d.id;
                    GameState::TeamEditor.proceed_to_target(world);
                })
            });
            world.resource_mut::<TeamResource>().table = data;
        })
        .pinned()
        .push(world);
    }

    fn editor_tiles(world: &mut World) {
        Self::load_editor_team(world);
        Tile::new(Side::Top, |ui, world| {
            let team = world.resource::<TeamResource>().team;
            if team == 0 {
                return;
            }
            let Some(team) = TTeam::find_by_id(team) else {
                return;
            };
            title(&team.name, ui);
            TeamContainer::new(Faction::Team)
                .top_content(|ui, world| {
                    ui.horizontal(|ui| {
                        if Button::click("Add Unit".into()).ui(ui).clicked() {
                            Confirmation::new("Add unit to team".cstr(), |world| {
                                let mut tr = world.resource_mut::<TeamResource>();
                                let id = tr.unit_to_add;
                                if id != 0 {
                                    tr.unit_to_add = 0;
                                    team_add_unit(tr.team, id);
                                    once_on_team_add_unit(|_, _, status, _, _| {
                                        status.on_success(|world| {
                                            TableState::reset_cache(&egui_context(world).unwrap());
                                        })
                                    });
                                }
                            })
                            .content(|ui, world| {
                                let units = TUnitItem::filter_by_owner(user_id())
                                    .map(|u| u.unit)
                                    .collect_vec();
                                let selected = units
                                    .show_modified_table("Units", ui, world, |t| t.selectable())
                                    .selected_row;
                                if let Some(selected) = selected {
                                    world.resource_mut::<TeamResource>().unit_to_add =
                                        units[selected].id;
                                }
                            })
                            .add(ui.ctx());
                        }
                        if Button::click("Disband".into()).red(ui).ui(ui).clicked() {
                            Confirmation::new("Disband team?".cstr_c(VISIBLE_LIGHT), |world| {
                                let tr = world.resource_mut::<TeamResource>();
                                team_disband(tr.team);
                                once_on_team_disband(|_, _, status, id| {
                                    let id = *id;
                                    status.on_success(move |w| {
                                        format!("Team#{id} disbanded").notify(w);
                                        GameState::Teams.proceed_to_target(w);
                                    })
                                });
                            })
                            .add(ui.ctx());
                        }
                    });
                })
                .slot_content(|slot, entity, ui, world| {
                    if entity.is_none() {
                        return;
                    }
                    ui.vertical_centered_justified(|ui| {
                        if Button::click("Remove".into()).ui(ui).clicked() {
                            let tr = world.resource::<TeamResource>();
                            team_remove_unit(tr.team, tr.team.get_team().units[slot].id);
                            once_on_team_remove_unit(|_, _, status, _, _| {
                                TableState::reset_cache_op();
                                status.on_success(|w| "Unit removed".notify(w));
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
        .min_space(egui::vec2(200.0, 200.0))
        .push_as_content(world);
    }

    pub fn add_tiles(state: GameState, world: &mut World) {
        TeamSyncPlugin::unsubscribe_all(world);
        Tile::new(Side::Top, |ui, world| {
            let mut r = world.resource_mut::<TeamResource>();
            SubstateMenu::show(&[GameState::Teams, GameState::TeamEditor], ui, world);
        })
        .pinned()
        .transparent()
        .non_focusable()
        .push(world);
        Self::load_teams_table(world);
        match state {
            GameState::Teams => Self::teams_tiles(world),
            GameState::TeamEditor => Self::editor_tiles(world),
            _ => panic!("Unsupported state"),
        }
    }
}
