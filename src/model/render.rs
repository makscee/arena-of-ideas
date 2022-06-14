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
pub struct ShaderParameters(HashMap<String, ShaderParameter>);

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ShaderParameter {
    Int(i32),
    Float(f32),
    Vec2(Vec2<f32>),
    Vec3(Vec3<f32>),
    Vec4(Vec4<f32>),
    Color(Color<f32>),
}

impl ugli::Uniform for ShaderParameter {
    fn apply(&self, gl: &ugli::raw::Context, info: &ugli::UniformInfo) {
        match self {
            Self::Int(value) => value.apply(gl, info),
            Self::Float(value) => value.apply(gl, info),
            Self::Vec2(value) => value.apply(gl, info),
            Self::Vec3(value) => value.apply(gl, info),
            Self::Vec4(value) => value.apply(gl, info),
            Self::Color(value) => value.apply(gl, info),
        }
    }
}
