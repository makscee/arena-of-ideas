use super::*;

pub struct Name(pub String);

impl Name {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}
