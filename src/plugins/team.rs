use super::*;

pub struct TeamPlugin;

#[derive(Resource)]
struct Teams(HashMap<Faction, Entity>);

#[derive(Component)]
pub struct Team;

impl Plugin for TeamPlugin {
    fn build(&self, app: &mut App) {
        let teams =
            HashMap::from_iter(Faction::iter().map(|f| (f, Self::spawn(f, &mut app.world))));
        app.insert_resource(Teams(teams));
    }
}

impl TeamPlugin {
    pub fn unit_faction(entity: Entity, world: &World) -> Faction {
        let team = entity.get_parent(world).unwrap();
        for (faction, entity) in world.resource::<Teams>().0.iter() {
            if team.eq(entity) {
                return *faction;
            }
        }
        panic!("No team found for {entity:?}")
    }
    pub fn despawn(faction: Faction, world: &mut World) {
        world
            .entity_mut(Self::entity(faction, world))
            .despawn_recursive();
        let entity = Self::spawn(faction, world);
        world.resource_mut::<Teams>().0.insert(faction, entity);
    }
    pub fn entity(faction: Faction, world: &World) -> Entity {
        *world
            .resource::<Teams>()
            .0
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
        let state = states.0.entry(ability.clone()).or_default();
        if !state.has_value(var) {
            state.init(var, d);
        }
        state.change_int(var, delta);
    }
}
