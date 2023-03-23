use super::*;

#[derive(Clone, Debug)]
pub struct Context {
    pub owner: legion::Entity,  // entity that has this context component
    pub target: legion::Entity, // any entity
    pub parent: Option<legion::Entity>, // World -> Unit -> Status
    pub vars: Vars,
}

impl Context {
    pub fn merge_mut(&mut self, other: &Context, force: bool) -> &mut Context {
        if force {
            self.owner = other.owner;
            self.target = other.target;
            self.parent = other.parent;
        }
        self.vars.merge_mut(&other.vars, force);
        self
    }

    pub fn merge(&self, other: &Context, force: bool) -> Context {
        let mut context = self.clone();
        context.merge_mut(other, force);
        context
    }

    pub fn add_var(&mut self, var: VarName, value: Var) -> &mut Self {
        self.vars.insert(var, value);
        self
    }
}
