use super::*;

#[derive(Clone)]
pub struct DescriptionComponent {
    pub text: String,
}

impl DescriptionComponent {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

impl VarsProvider for DescriptionComponent {
    fn extend_vars(&self, vars: &mut Vars) {
        vars.insert(VarName::Description, Var::String(self.text.clone()));
        vars.insert(VarName::Font, Var::Int(1));
    }
}