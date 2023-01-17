use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ShaderConfig {
    pub path: String,
    #[serde(default)]
    pub parameters: ShaderParameters,
    #[serde(default = "default_vertices")]
    pub vertices: usize,
    #[serde(default = "default_instances")]
    pub instances: usize,
}

fn default_vertices() -> usize {
    4
}

fn default_instances() -> usize {
    1
}
