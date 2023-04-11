use super::*;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct AnimatedShaderUniforms {
    key_frames: Vec<KeyFrame>,
    #[serde(default)]
    pub easing: EasingType,
}

impl AnimatedShaderUniforms {
    pub fn empty(easing: EasingType) -> Self {
        Self {
            key_frames: default(),
            easing,
        }
    }

    pub fn from_to(from: ShaderUniforms, to: ShaderUniforms, easing: EasingType) -> Self {
        let key_frames = vec![
            KeyFrame::new(0.0, from, default()),
            KeyFrame::new(1.0, to, default()),
        ];
        Self { key_frames, easing }
    }

    pub fn add_key_frame(mut self, t: Time, uniforms: ShaderUniforms, easing: EasingType) -> Self {
        self.key_frames.push(KeyFrame::new(t, uniforms, easing));
        self.key_frames
            .sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        self
    }

    pub fn get_mixed(&self, t: Time) -> ShaderUniforms {
        if t < 0.0 || t > 1.0 {
            panic!("Wrong t range {}", t);
        }
        let mut t = self.easing.f(t);

        let mut result = default();
        for (i, left) in self.key_frames.iter().enumerate().rev() {
            if left.t < t {
                if let Some(right) = self.key_frames.get(i + 1) {
                    t = (t - left.t) / (right.t - left.t);
                    t = right.easing.f(t);
                    result = ShaderUniforms::mix(&left.uniforms, &right.uniforms, t);
                } else {
                    result = left.uniforms.clone();
                    t = (t - left.t) / (1.0 - left.t);
                }
                break;
            }
            if i == 0 {
                result = left.uniforms.clone();
            }
        }
        result.insert_ref("u_t", ShaderUniform::Float(t));
        result
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct KeyFrame {
    pub t: Time,
    pub uniforms: ShaderUniforms,
    #[serde(default)]
    pub easing: EasingType,
}

impl KeyFrame {
    pub fn new(t: Time, uniforms: ShaderUniforms, easing: EasingType) -> Self {
        Self {
            t,
            uniforms,
            easing,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum EasingType {
    Linear,
    QuartOut,
    QuartIn,
    QuartInOut,
    QuadOut,
    QuadIn,
    QuadInOut,
    CubicIn,
    CubicOut,
    BackIn,
}

impl Default for EasingType {
    fn default() -> Self {
        Self::Linear
    }
}

impl EasingType {
    pub fn f(&self, t: f32) -> f32 {
        match self {
            EasingType::Linear => tween::Tweener::linear(0.0, 1.0, 1.0).move_to(t),
            EasingType::QuartOut => tween::Tweener::quart_out(0.0, 1.0, 1.0).move_to(t),
            EasingType::QuartIn => tween::Tweener::quart_in(0.0, 1.0, 1.0).move_to(t),
            EasingType::QuartInOut => tween::Tweener::quart_in_out(0.0, 1.0, 1.0).move_to(t),
            EasingType::QuadOut => tween::Tweener::quad_out(0.0, 1.0, 1.0).move_to(t),
            EasingType::QuadIn => tween::Tweener::quad_in(0.0, 1.0, 1.0).move_to(t),
            EasingType::QuadInOut => tween::Tweener::quad_in_out(0.0, 1.0, 1.0).move_to(t),
            EasingType::CubicIn => tween::Tweener::cubic_in(0.0, 1.0, 1.0).move_to(t),
            EasingType::CubicOut => tween::Tweener::cubic_out(0.0, 1.0, 1.0).move_to(t),
            EasingType::BackIn => tween::Tweener::back_in(0.0, 1.0, 1.0).move_to(t),
        }
    }
}
