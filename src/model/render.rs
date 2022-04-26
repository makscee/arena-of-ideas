use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum RenderConfig {
    Circle {
        color: Color<f32>,
    },
    Texture {
        path: String,
    },
    Shader {
        path: String,
        #[serde(default)]
        parameters: ShaderParameters,
    },
}

#[derive(Clone)]
pub enum RenderMode {
    Circle {
        color: Color<f32>,
    },
    Texture {
        texture: Rc<ugli::Texture>,
    },
    Shader {
        program: Rc<ugli::Program>,
        parameters: ShaderParameters,
    },
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self::Circle {
            color: Color::BLACK,
        }
    }
}

impl Default for RenderMode {
    fn default() -> Self {
        Self::Circle {
            color: Color::BLACK,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShaderParameters(HashMap<String, f32>);

impl ugli::Uniforms for ShaderParameters {
    fn walk_uniforms<C>(&self, visitor: &mut C)
    where
        C: ugli::UniformVisitor,
    {
        for (name, value) in &self.0 {
            visitor.visit(name, value);
        }
    }
}
