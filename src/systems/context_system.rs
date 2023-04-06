use super::*;

pub struct ContextSystem {}

impl System for ContextSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        Self::refresh_all(world, resources);
    }
}

impl ContextSystem {
    pub fn new() -> Self {
        Self {}
    }

    /// Merge data from other entity components
    pub fn refresh_entity(
        entity: legion::Entity,
        world: &mut legion::World,
        resources: &Resources,
    ) -> Context {
        let entry = world.entry(entity).expect("Unit entity not found");
        let mut context = Context {
            vars: default(),
            ..entry.get_component::<Context>().unwrap().clone()
        };

        if let Some(component) = entry.get_component::<AreaComponent>().ok() {
            component.extend_vars(&mut context.vars, resources);
        }
        if let Some(component) = entry.get_component::<HealthComponent>().ok() {
            component.extend_vars(&mut context.vars, resources);
        }
        if let Some(component) = entry.get_component::<AttackComponent>().ok() {
            component.extend_vars(&mut context.vars, resources);
        }
        if let Some(component) = entry.get_component::<DescriptionComponent>().ok() {
            component.extend_vars(&mut context.vars, resources);
        }
        if let Some(component) = entry.get_component::<HouseComponent>().ok() {
            component.extend_vars(&mut context.vars, resources);
        }
        if let Some(component) = entry.get_component::<UnitComponent>().ok() {
            component.extend_vars(&mut context.vars, resources);
        }

        // apply changes from active statuses
        context = Event::ModifyContext { context }.calculate(world, resources);

        world.entry(entity).unwrap().add_component(context.clone());
        context
    }

    pub fn try_get_context(
        entity: legion::Entity,
        world: &legion::World,
    ) -> Result<Context, Error> {
        let mut context = world.entry_ref(entity)?.get_component::<Context>()?.clone();
        if let Some(parent) = context.parent {
            context.merge_mut(&ContextSystem::get_context(parent, world), false);
        }
        Ok(context)
    }

    pub fn get_context(entity: legion::Entity, world: &legion::World) -> Context {
        Self::try_get_context(entity, world).expect(&format!(
            "Failed to get Context component from entity#{:?}",
            entity
        ))
    }

    pub fn try_get_position(entity: legion::Entity, world: &legion::World) -> Option<vec2<f32>> {
        Self::try_get_context(entity, world)
            .ok()
            .and_then(|x| x.vars.try_get_vec2(&VarName::Position))
    }

    pub fn refresh_factions(
        factions: &HashSet<Faction>,
        world: &mut legion::World,
        resources: &Resources,
    ) {
        UnitSystem::collect_factions(world, resources, factions, false)
            .into_iter()
            .for_each(|entity| {
                Self::refresh_entity(entity, world, resources);
            });
    }

    pub fn refresh_all(world: &mut legion::World, resources: &Resources) {
        <&EntityComponent>::query()
            .filter(!component::<WorldComponent>() & component::<Context>())
            .iter(world)
            .map(|entity| entity.entity)
            .collect_vec()
            .into_iter()
            .for_each(|entity| {
                Self::refresh_entity(entity, world, resources);
            });
    }
}
