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
    pub fn add_hints(shader: &mut Shader, definitions: HashSet<String>, resources: &Resources) {
        if !definitions.is_empty() {
            for title in definitions.into_iter() {
                let data = resources.definitions.get(&title).unwrap();
                let (color, text) = (data.color.clone(), data.description.clone());
                shader.hover_hints.push((color, title, text));
            }
        }
    }
}

pub struct Definition {
    pub description: String,
    pub color: Rgba<f32>,
}
