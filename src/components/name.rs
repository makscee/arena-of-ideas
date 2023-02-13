use super::*;

#[derive(Clone)]
pub struct NameComponent(pub String);

impl NameComponent {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}
