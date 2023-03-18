use super::*;

#[derive(Clone, Debug)]
pub struct Context {
    pub owner: legion::Entity,  // entity that has this context component
    pub target: legion::Entity, // any entity
    pub parent: Option<legion::Entity>, // World -> Unit -> Status
    pub vars: Vars,
}

impl Context {
    pub fn merge_mut(mut self, other: &Context, force: bool) -> Context {
        if force {
            self.owner = other.owner;
            self.target = other.target;
            self.parent = other.parent;
        }
        self.vars.merge(&other.vars, force);
        self
    }

    pub fn merge(&self, other: &Context, force: bool) -> Context {
        let context = self.clone();
        context.merge_mut(other, force)
    }

    pub fn add_var(&mut self, name: VarName, var: Var) -> &mut Self {
        self.vars.insert(name, var);
        self
    }
}
