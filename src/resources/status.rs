use super::*;

#[derive(Debug, Deserialize, Clone)]
pub struct Status {
    pub name: String,
    pub triggers: Vec<Trigger>,
}

#[derive(Default)]
pub struct Statuses {
    pub defined_statuses: HashMap<String, Status>, // key = status name
    pub active_statuses: HashMap<legion::Entity, HashMap<String, Context>>, // entity -> status name -> context
}

impl Statuses {}
