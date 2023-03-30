use super::*;

#[derive(Default)]
pub struct Definitions(HashMap<String, Definition>);

impl Definitions {
    pub fn get(&self, key: &str) -> Option<&Definition> {
        self.0.get(key)
    }
    pub fn insert(&mut self, key: String, color: Rgba<f32>, description: String) {
        self.0.insert(key, Definition { description, color });
    }
    pub fn contains(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }
}

pub struct Definition {
    pub description: String,
    pub color: Rgba<f32>,
}
