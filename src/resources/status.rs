use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Debug, Deserialize, Clone)]
pub struct Status {
    pub name: String,
    pub trigger: Trigger,
    pub shader: Option<Shader>,
}

#[derive(Default)]
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
}
