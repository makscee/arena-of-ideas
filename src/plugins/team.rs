use super::*;

pub struct TeamPlugin;

#[derive(Resource, Default)]
struct Teams {
    entities: HashMap<Faction, Entity>,
}

impl Plugin for TeamPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Teams::default());
    }
}

impl TeamPlugin {
    pub fn get_ability_state<'a>(
        faction: Faction,
        ability: &str,
        world: &'a World,
    ) -> Option<&'a VarState> {
        let team = Self::get_entity(faction, world)?;
        world
            .get::<AbilityStates>(team)?
            .0
            .get(ability)
            .or(Pools::get_default_ability_state(ability, world))
    }
    pub fn add_ability_var(
        faction: Faction,
        ability: &str,
        var: VarName,
        value: VarValue,
        world: &mut World,
    ) -> Result<()> {
        let team = Self::get_entity(faction, world).context("No team entity")?;

        let states = world.get_mut::<AbilityStates>(team).context("No states")?;
        if !states.0.contains_key(ability) {
            let d = Pools::get_default_ability_state(ability, world)
                .cloned()
                .unwrap_or_default();
            let mut states = world.get_mut::<AbilityStates>(team).unwrap();
            states.0.insert(ability.to_owned(), d);
        }
        let mut states = world.get_mut::<AbilityStates>(team).unwrap();
        let state = states.0.get_mut(ability).unwrap();
        let prev = state.get_value_last(var).unwrap_or(VarValue::Int(0));
        state.init(var, VarValue::sum(&prev, &value)?);
        Ok(())
    }
    pub fn inject_ability_state(
        faction: Faction,
        ability: &str,
        context: &mut Context,
        world: &mut World,
    ) {
        if let Some(ability_state) = Self::get_ability_state(faction, ability, world) {
            for (var, history) in ability_state.history.iter() {
                if let Some(value) = history.get_last() {
                    context.set_ability_var(ability.to_owned(), *var, value);
                }
            }
        }
    }

    pub fn spawn(faction: Faction, world: &mut World) -> Entity {
        Self::despawn(faction, world);
        let team = world
            .spawn((
                VarState::new_with(VarName::Faction, VarValue::Faction(faction))
                    .init(VarName::Name, VarValue::String(format!("Team {faction}")))
                    .take(),
                Team,
                Transform::default(),
                GlobalTransform::default(),
                VisibilityBundle::default(),
                AbilityStates::default(),
            ))
            .id();
        if faction == Faction::Team {
            for slot in 1..=TEAM_SLOTS {
                UnitPlugin::spawn_slot(slot, Faction::Team, world);
            }
        }
        world.resource_mut::<Teams>().entities.insert(faction, team);
        team
    }
    pub fn despawn(faction: Faction, world: &mut World) {
        if let Some(team) = Self::find_entity(faction, world) {
            world.entity_mut(team).despawn_recursive();
            world.resource_mut::<Teams>().entities.remove(&faction);
        }
    }
    pub fn get_entity(faction: Faction, world: &World) -> Option<Entity> {
        world.resource::<Teams>().entities.get(&faction).copied()
    }
    pub fn find_entity(faction: Faction, world: &mut World) -> Option<Entity> {
        world
            .query_filtered::<(Entity, &VarState), With<Team>>()
            .iter(world)
            .find_map(
                |(e, s)| match s.get_faction(VarName::Faction).unwrap().eq(&faction) {
                    true => Some(e),
                    false => None,
                },
            )
    }
    pub fn state(faction: Faction, world: &mut World) -> Option<&VarState> {
        Self::find_entity(faction, world).map(|e| VarState::get(e, world))
    }
    pub fn state_mut(faction: Faction, world: &mut World) -> Option<Mut<VarState>> {
        Self::find_entity(faction, world).map(|e| VarState::get_mut(e, world))
    }
}

#[derive(Component)]
pub struct Team;
