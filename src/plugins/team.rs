use egui::TextBuffer;

use super::*;

pub struct TeamPlugin;

#[derive(Resource)]
struct TeamResource {
    entities: HashMap<Faction, Entity>,
    table: Vec<TTeam>,
    new_team_name: String,
    substate: SubState,
}

#[derive(Component)]
pub struct Team;

#[derive(PartialEq, Copy, Clone, EnumIter, Display, Default)]
enum SubState {
    #[default]
    Teams,
    TeamEditor,
}

impl Plugin for TeamPlugin {
    fn build(&self, app: &mut App) {
        let teams =
            HashMap::from_iter(Faction::iter().map(|f| (f, Self::spawn(f, &mut app.world_mut()))));
        app.insert_resource(TeamResource {
            entities: teams,
            table: default(),
            new_team_name: default(),
            substate: default(),
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

    fn load_teams(world: &mut World) {
        world.resource_mut::<TeamResource>().table =
            TTeam::filter_by_owner(user_id()).collect_vec();
        TableState::reset_cache(&egui_context(world).unwrap());
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
                    new_team(name);
                    once_on_new_team(|_, _, status, _| {
                        status.on_success(|world| Self::load_teams(world));
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
        })
        .sticky()
        .push(world);
        Tile::new(Side::Right, |ui, world| {
            world.resource_scope(|world, data: Mut<TeamResource>| {
                data.table.show_table("Teams", ui, world);
            });
        })
        .sticky()
        .push(world);
    }

    fn editor_tiles(world: &mut World) {
        Tile::new(Side::Top, |ui, world| {
            UnitContainer::new(Faction::Team)
                .slot_content(|i, e, ui, world| {
                    if e.is_none() {
                        ui.vertical_centered_justified(|ui| {
                            if Button::click("Add".into()).ui(ui).clicked() {
                                debug!("Add");
                            }
                        });
                    }
                })
                .ui(ui, world);
        })
        .sticky()
        .min_size(300.0)
        .transparent()
        .push_as_content(world);
    }

    pub fn add_tiles(state: GameState, world: &mut World) {
        Tile::new(Side::Top, |ui, world| {
            let mut r = world.resource_mut::<TeamResource>();
            let state = SubsectionMenu::new(r.substate).show(ui);
            if r.substate != state {
                r.substate = state;
                match state {
                    SubState::Teams => GameState::Teams.proceed_to_target(world),
                    SubState::TeamEditor => GameState::TeamEditor.proceed_to_target(world),
                }
            }
        })
        .min_size(0.0)
        .sticky()
        .transparent()
        .non_focusable()
        .push(world);
        Self::load_teams(world);
        match state {
            GameState::Teams => Self::teams_tiles(world),
            GameState::TeamEditor => Self::editor_tiles(world),
            _ => panic!("Unsupported state"),
        }
    }
}
