use super::*;

#[derive(Clone, Debug)]
pub struct Context {
    pub owner: legion::Entity,
    pub target: legion::Entity,
    pub creator: legion::Entity,
}
