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
    fn extend_vars(&self, vars: &mut Vars, resources: &Resources) {
        vars.insert(VarName::Description, Var::String((1, self.text.clone())));
    }
}
