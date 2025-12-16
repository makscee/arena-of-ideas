use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Material {
    pub script: RhaiScript<PainterAction>,
}

impl Material {
    pub fn new(code: String) -> Self {
        Self {
            script: RhaiScript::new(code),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.script = self.script.with_description(description);
        self
    }
}
