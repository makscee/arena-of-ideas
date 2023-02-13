use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Debug, Deserialize, Clone)]
pub struct Status {
    pub name: String,
    pub trigger: Trigger,
    pub shader: Option<Shader>,
}

#[derive(Default, Debug)]
pub struct Statuses {
    pub defined_statuses: HashMap<String, Status>, // key = status name
    pub active_statuses: HashMap<legion::Entity, HashMap<String, Context>>, // entity -> status name -> context
}

impl Statuses {
    pub fn get_entity_shaders(&self, entity: &legion::Entity) -> Vec<Shader> {
        self.active_statuses
            .get(entity)
            .unwrap_or(&default())
            .keys()
            .filter_map(|key| self.defined_statuses.get(key).unwrap().shader.clone())
            .collect_vec()
    }

    pub fn add_entity_status(
        &mut self,
        entity: &legion::Entity,
        status_name: &str,
        context: Context,
    ) {
        let mut statuses = self.active_statuses.remove(entity).unwrap_or_default();
        statuses.insert(
            status_name.to_string(),
            Context {
                status: Some((status_name.to_string(), entity.clone())),
                ..context
            },
        );
        self.active_statuses.insert(*entity, statuses);
    }

    pub fn add_and_define_entity_status(
        &mut self,
        entity: &legion::Entity,
        status: &Status,
        context: Context,
    ) {
        self.add_entity_status(entity, &status.name, context);
        self.defined_statuses
            .insert(status.name.clone(), status.clone());
    }
}
