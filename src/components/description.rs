use super::*;

pub struct Description {
    pub text: String,
}

impl Description {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

impl VarsProvider for Description {
    fn extend_vars(&self, vars: &mut Vars) {
        vars.insert(VarName::Description, Var::String(self.text.clone()));
        vars.insert(VarName::Font, Var::Int(1));
    }
}
