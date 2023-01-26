use super::*;

#[derive(Clone)]
pub struct ContextComponent {
    pub owner: legion::Entity,
    pub target: legion::Entity,
    pub creator: legion::Entity,
}
