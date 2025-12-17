use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Material(pub RhaiScript<PainterAction>);

impl Material {
    pub fn new(code: String) -> Self {
        Self(RhaiScript::new(code))
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.0 = self.0.with_description(description);
        self
    }
}
