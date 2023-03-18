use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Debug, Deserialize, Clone)]
pub struct Status {
    pub trigger: Trigger,
    pub color: Option<Rgba<f32>>,
    pub shader: Option<Shader>,
}

#[derive(Default, Debug)]
pub struct StatusPool {
    pub defined_statuses: HashMap<String, Status>, // status name -> status
    pub active_statuses: HashMap<legion::Entity, HashMap<String, i32>>, // entity -> status name -> charges
    pub status_changes: VecDeque<(legion::Entity, String, i32)>, // added or removed status charges
}

impl StatusPool {
    /// Send event to all active statuses
    pub fn notify_all(event: &Event, resources: &mut Resources, world: &legion::World) {
        resources
            .status_pool
            .active_statuses
            .keys()
            .cloned()
            .collect_vec()
            .into_iter()
            .for_each(|entity| Self::notify_entity(event, entity, resources, world, None))
    }

    /// Trigger all active statuses on entity
    pub fn notify_entity(
        event: &Event,
        entity: legion::Entity,
        resources: &mut Resources,
        world: &legion::World,
        context: Option<Context>,
    ) {
        if let Some(statuses) = resources.status_pool.active_statuses.get(&entity) {
            let context = context.unwrap_or(ContextSystem::get_context(entity, world));
            statuses.iter().for_each(|(status_name, _)| {
                resources
                    .status_pool
                    .defined_statuses
                    .get(status_name)
                    .unwrap()
                    .trigger
                    .catch_event(
                        event,
                        &mut resources.action_queue,
                        context
                            .clone()
                            .add_var(VarName::StatusName, Var::String((0, status_name.clone())))
                            .to_owned(),
                        &resources.logger,
                    );
            });
        }
    }

    pub fn collect_triggers(&self, entity: &legion::Entity) -> Vec<(String, Trigger, i32)> {
        self.collect_statuses(entity)
            .into_iter()
            .map(|(name, status, charges)| (name, status.trigger, charges))
            .collect_vec()
    }

    pub fn collect_statuses(&self, entity: &legion::Entity) -> Vec<(String, Status, i32)> {
        self.active_statuses
            .get(entity)
            .unwrap_or(&default())
            .iter()
            .map(|(name, charges)| {
                (
                    name.clone(),
                    self.defined_statuses
                        .get(name)
                        .expect(&format!("Status undefined: {}", name))
                        .clone(),
                    *charges,
                )
            })
            .collect_vec()
    }

    pub fn get_entity_shaders(&self, entity: &legion::Entity) -> Vec<Shader> {
        self.collect_statuses(entity)
            .into_iter()
            .filter_map(|(_, status, _)| match status.shader {
                Some(shader) => Some(shader),
                None => None,
            })
            .collect_vec()
    }

    pub fn change_entity_status(
        entity: legion::Entity,
        status_name: &str,
        resources: &mut Resources,
        charges_change: i32,
    ) {
        if charges_change != 0 {
            resources.status_pool.status_changes.push_back((
                entity,
                status_name.to_string(),
                charges_change,
            ))
        }
    }

    fn add_status_charge(
        entity: legion::Entity,
        status: &str,
        resources: &mut Resources,
        world: &legion::World,
    ) {
        let mut entity_statuses = resources
            .status_pool
            .active_statuses
            .remove(&entity)
            .unwrap_or_default();
        let charges = *entity_statuses.get(status).unwrap_or(&0);
        if charges == 0 {
            Event::StatusAdd {
                status: status.to_string(),
                owner: entity,
            }
            .send(resources, world);
        }
        Event::StatusChargeAdd {
            status: status.to_string(),
            owner: entity,
        }
        .send(resources, world);
        entity_statuses.insert(status.to_string(), charges + 1);
        resources
            .status_pool
            .active_statuses
            .insert(entity, entity_statuses);
    }

    fn remove_status_charge(
        entity: legion::Entity,
        status: &str,
        resources: &mut Resources,
        world: &legion::World,
    ) {
        let mut entity_statuses = resources
            .status_pool
            .active_statuses
            .remove(&entity)
            .unwrap_or_default();
        let charges = *entity_statuses.get(status).unwrap_or(&0);
        if charges == 1 {
            Event::StatusRemove {
                status: status.to_string(),
                owner: entity,
            }
            .send(resources, world);
        }
        Event::StatusChargeRemove {
            status: status.to_string(),
            owner: entity,
        }
        .send(resources, world);
        if charges > 1 {
            entity_statuses.insert(status.to_string(), charges - 1);
        } else {
            entity_statuses.remove(status);
        }
        resources
            .status_pool
            .active_statuses
            .insert(entity, entity_statuses);
    }

    pub fn process_status_changes(world: &legion::World, resources: &mut Resources) {
        while let Some((entity, status_name, charges_delta)) =
            resources.status_pool.status_changes.pop_front()
        {
            for _ in 0..charges_delta.abs() {
                if charges_delta > 0 {
                    Self::add_status_charge(entity, &status_name, resources, world);
                } else {
                    Self::remove_status_charge(entity, &status_name, resources, world);
                }
            }
        }
    }

    pub fn define_status(&mut self, name: String, mut status: Status) {
        if status.shader.is_some() && status.color.is_some() {
            if let Some(ref mut shader) = status.shader {
                shader.parameters.uniforms.insert(
                    "u_color".to_string(),
                    ShaderUniform::Color(status.color.unwrap()),
                );
            }
        }
        self.defined_statuses.insert(name, status);
    }

    pub fn clear_all_active(&mut self) {
        self.active_statuses.clear();
    }

    pub fn clear_entity(&mut self, entity: &legion::Entity) {
        self.active_statuses.remove(entity);
    }
}
