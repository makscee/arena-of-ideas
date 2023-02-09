use legion::EntityStore;

use super::*;

#[derive(Clone, Debug)]
pub struct Context {
    pub owner: legion::Entity,
    pub target: legion::Entity,
    pub creator: legion::Entity,
    pub vars: Vars,
    pub status: Option<(String, legion::Entity)>,
}

impl Context {
    /// Merge data from other entity components
    pub fn construct_context(entity: &legion::Entity, world: &legion::World) -> Context {
        let entry = world.entry_ref(*entity).expect("Unit entity not found");
        let mut context = entry
            .get_component::<Context>()
            .unwrap_or(&Context {
                owner: *entity,
                target: *entity,
                creator: *entity,
                vars: default(),
                status: None,
            })
            .clone();
        if let Some(component) = entry.get_component::<UnitComponent>().ok() {
            component.extend_vars(&mut context.vars);
        }
        if let Some(component) = entry.get_component::<Position>().ok() {
            component.extend_vars(&mut context.vars);
        }
        if let Some(component) = entry.get_component::<HpComponent>().ok() {
            component.extend_vars(&mut context.vars);
        }

        context
    }
}
