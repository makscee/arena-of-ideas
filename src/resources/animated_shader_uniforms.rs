use super::*;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct AnimatedShaderUniforms {
    key_frames: Vec<KeyFrame>,
    #[serde(default)]
    easing: EasingType,
}

impl AnimatedShaderUniforms {
    pub fn empty() -> Self {
        Self {
            key_frames: default(),
            easing: EasingType::Linear,
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
        let mut uniforms = ShaderUniforms::default();
        if t < 0.0 {
            return uniforms;
        }
        let mut t = self.easing.f(t);
        let mut prev_t = 0.0;

        for frame in self.key_frames.iter() {
            if frame.t < t {
                for (key, value) in frame.uniforms.iter_data() {
                    uniforms.insert_ref(key, value.clone());
                }
            } else {
                t = (t - prev_t) / (frame.t - prev_t);
                t = frame.easing.f(t);
                let mixed = ShaderUniforms::mix(&uniforms, &frame.uniforms, t, &uniforms);
                uniforms.merge_mut(&mixed, true);
                break;
            }
            prev_t = frame.t;
        }

        uniforms.insert_ref("u_t".to_owned(), ShaderUniform::Float(t));
        uniforms
    }

    pub fn get_uniforms_mut<'a>(&'a mut self) -> &'a mut ShaderUniforms {
        &mut self.key_frames.get_mut(0).unwrap().uniforms
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct KeyFrame {
    pub t: Time,
    #[serde(default)]
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
