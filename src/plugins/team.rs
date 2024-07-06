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
}
