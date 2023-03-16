use super::*;

pub struct Definitions(HashMap<String, String>);

impl Definitions {
    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }
    pub fn insert(&mut self, key: String, value: String) {
        self.insert(key, value);
    }
}
