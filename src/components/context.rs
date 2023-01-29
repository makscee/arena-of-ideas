use super::*;

#[derive(Clone, Debug)]
pub struct Context {
    pub owner: legion::Entity,
    pub target: legion::Entity,
    pub creator: legion::Entity,
    pub vars: Vars,
    pub status: Option<(String, legion::Entity)>,
}
