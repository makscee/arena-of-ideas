use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Debug, Deserialize, Clone)]
pub struct Status {
    pub trigger: Trigger,
    pub shader: Option<Shader>,
}

#[derive(Default, Debug)]
pub struct StatusPool {
    pub defined_statuses: HashMap<String, Status>, // key = status name
    pub active_statuses: HashMap<legion::Entity, HashMap<String, Context>>, // entity -> status name -> context
}

impl StatusPool {
    pub fn get_entity_shaders(&self, entity: &legion::Entity) -> Vec<Shader> {
        self.active_statuses
            .get(entity)
            .unwrap_or(&default())
            .keys()
            .filter_map(|key| self.defined_statuses.get(key).unwrap().shader.clone())
            .collect_vec()
    }

    pub fn add_entity_status(
        entity: &legion::Entity,
        status_name: &str,
        context: Context,
        resources: &mut Resources,
    ) {
        let mut statuses = resources
            .status_pool
            .active_statuses
            .remove(entity)
            .unwrap_or_default();
        statuses.insert(
            status_name.to_string(),
            Context {
                status: Some((status_name.to_string(), entity.clone())),
                ..context.clone()
            },
        );
        resources
            .status_pool
            .active_statuses
            .insert(*entity, statuses);

        {
            Event::Init {
                status: status_name.to_string(),
            }
        }
        .send(&context, resources);
    }

    pub fn define_status(&mut self, name: String, status: Status) {
        self.defined_statuses.insert(name, status);
    }
}
