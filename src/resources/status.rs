use super::*;

#[derive(Debug, Deserialize, Clone)]
pub struct Status {
    pub name: String,
    pub triggers: Vec<Trigger>,
}
