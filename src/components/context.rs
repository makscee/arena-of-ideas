use super::*;

#[derive(Clone)]
pub struct Context {
    pub owner: legion::Entity,
    pub target: legion::Entity,
    pub creator: legion::Entity,
}
