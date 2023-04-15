use super::*;

pub struct ContextSystem;
impl ContextSystem {
    /// Merge data from other entity components
    pub fn refresh_entity(
        entity: legion::Entity,
        world: &mut legion::World,
        resources: &Resources,
    ) -> Context {
        let entry = world.entry(entity).expect("Context entity not found");
        let context = entry.get_component::<Context>().unwrap().clone();
        if entry.get_component::<WorldComponent>().is_ok() {
            return context;
        }
        let mut context = Context {
            vars: default(),
            ..context
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

    pub fn refresh_by_context(context: &Context, world: &mut legion::World, resources: &Resources) {
        Self::refresh_entity(context.owner, world, resources);
        Self::refresh_entity(context.target, world, resources);
    }
}
