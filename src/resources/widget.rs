use super::*;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Widget {
    BattleOverPanel { score: usize },
}

impl Widget {
    pub fn generate_node(&self, options: &Options) -> Node {
        match self {
            Widget::BattleOverPanel { score } => {
                let mut node = Node::default();
                let (text, color) = match score {
                    0 => (String::from("Defeat"), options.colors.defeat),
                    _ => (String::from("Victory"), options.colors.victory),
                };
                let mut uniforms: ShaderUniforms = hashmap! {
                    "u_color" => ShaderUniform::Color(color),
                }
                .into();

                let shader = options
                    .shaders
                    .battle_over_panel_bg
                    .clone()
                    .merge_uniforms(&uniforms, false);
                let animation = AnimatedShaderUniforms::empty(EasingType::Linear)
                    .add_key_frame(
                        0.0,
                        uniforms.clone().insert_vec("u_box", vec2(1.0, 0.0)),
                        EasingType::Linear,
                    )
                    .add_key_frame(
                        0.2,
                        uniforms.clone().insert_vec("u_box", vec2(1.0, 0.5)),
                        EasingType::CubicOut,
                    )
                    .add_key_frame(
                        0.8,
                        uniforms.clone().insert_vec("u_box", vec2(1.0, 0.5)),
                        EasingType::CubicOut,
                    )
                    .add_key_frame(
                        1.0,
                        uniforms.clone().insert_vec("u_box", vec2(1.0, 0.0)),
                        EasingType::CubicIn,
                    );
                node.add_effect(VisualEffect::new(
                    3.0,
                    VisualEffectType::ShaderAnimation {
                        shader: shader.clone(),
                        animation,
                    },
                    0,
                ));

                uniforms.insert_color_ref("u_color", Rgba::WHITE);
                let animation = AnimatedShaderUniforms::empty(EasingType::Linear)
                    .add_key_frame(
                        0.05,
                        uniforms.insert_vec_ref("u_box", vec2(1.0, 0.0)).clone(),
                        EasingType::Linear,
                    )
                    .add_key_frame(
                        0.15,
                        uniforms.insert_vec_ref("u_box", vec2(1.0, 0.45)).clone(),
                        EasingType::CubicOut,
                    )
                    .add_key_frame(0.65, uniforms.clone(), EasingType::CubicOut)
                    .add_key_frame(
                        0.85,
                        uniforms.insert_vec_ref("u_box", vec2(1.0, 0.0)).clone(),
                        EasingType::CubicIn,
                    );
                node.add_effect(VisualEffect::new(
                    3.0,
                    VisualEffectType::ShaderAnimation { shader, animation },
                    0,
                ));

                let mut uniforms: ShaderUniforms = hashmap! {
                    "u_color" => ShaderUniform::Color(color),
                    "u_text" => ShaderUniform::String((1, text)),
                }
                .into();
                let shader = options
                    .shaders
                    .battle_over_panel_title
                    .clone()
                    .merge_uniforms(&uniforms, true);
                let animation = AnimatedShaderUniforms::empty(EasingType::Linear)
                    .add_key_frame(
                        0.1,
                        uniforms
                            .insert_vec_ref("u_position", vec2(3.0, 0.0))
                            .clone(),
                        EasingType::Linear,
                    )
                    .add_key_frame(
                        0.2,
                        uniforms
                            .insert_vec_ref("u_position", vec2(0.0, 0.0))
                            .clone(),
                        EasingType::CubicOut,
                    )
                    .add_key_frame(0.6, uniforms.clone(), EasingType::CubicOut)
                    .add_key_frame(
                        0.8,
                        uniforms
                            .insert_vec_ref("u_position", vec2(-2.0, 0.0))
                            .clone(),
                        EasingType::CubicIn,
                    );
                node.add_effect(VisualEffect::new(
                    3.0,
                    VisualEffectType::ShaderAnimation { shader, animation },
                    0,
                ));

                let initial_uniforms: ShaderUniforms = hashmap! {
                    "u_color" => ShaderUniform::Color(options.colors.inactive),
                    "u_position" => ShaderUniform::Vec2(vec2(0.0,-0.1)),
                    "u_scale" => ShaderUniform::Float(0.0),
                    "u_offset" => ShaderUniform::Vec2(vec2::ZERO)
                }
                .into();
                let shader = options
                    .shaders
                    .battle_over_panel_star
                    .clone()
                    .merge_uniforms(&initial_uniforms, true);
                let mut uniforms = initial_uniforms.clone();
                let mut animation = AnimatedShaderUniforms::empty(EasingType::Linear)
                    .add_key_frame(0.2, uniforms.clone(), EasingType::Linear)
                    .add_key_frame(
                        0.4,
                        uniforms
                            .insert_vec_ref("u_offset", vec2(-0.15, 0.0))
                            .insert_float_ref("u_scale", 1.0)
                            .clone(),
                        EasingType::CubicOut,
                    );
                if *score > 0 {
                    animation = animation.add_key_frame(
                        0.42,
                        uniforms
                            .insert_color_ref("u_color", options.colors.star)
                            .insert_float_ref("u_scale", 1.5)
                            .clone(),
                        EasingType::CubicIn,
                    );
                }
                animation = animation
                    .add_key_frame(
                        0.65,
                        uniforms.insert_float_ref("u_scale", 1.0).clone(),
                        EasingType::Linear,
                    )
                    .add_key_frame(0.75, uniforms.clone(), EasingType::Linear)
                    .add_key_frame(
                        0.9,
                        uniforms
                            .insert_float_ref("u_scale", 0.0)
                            .insert_vec_ref("u_offset", vec2(-1.0, 0.0))
                            .clone(),
                        EasingType::CubicIn,
                    );
                node.add_effect(VisualEffect::new(
                    3.0,
                    VisualEffectType::ShaderAnimation {
                        shader: shader.clone(),
                        animation,
                    },
                    0,
                ));

                let mut uniforms = initial_uniforms.clone();
                let mut animation = AnimatedShaderUniforms::empty(EasingType::Linear)
                    .add_key_frame(0.2, uniforms.clone(), EasingType::Linear)
                    .add_key_frame(
                        0.45,
                        uniforms
                            .insert_vec_ref("u_offset", vec2(0.15, 0.0))
                            .insert_float_ref("u_scale", 1.0)
                            .clone(),
                        EasingType::CubicOut,
                    );
                if *score > 1 {
                    animation = animation.add_key_frame(
                        0.47,
                        uniforms
                            .insert_color_ref("u_color", options.colors.star)
                            .insert_float_ref("u_scale", 1.5)
                            .clone(),
                        EasingType::CubicIn,
                    );
                }
                animation = animation
                    .add_key_frame(
                        0.65,
                        uniforms.insert_float_ref("u_scale", 1.0).clone(),
                        EasingType::Linear,
                    )
                    .add_key_frame(0.75, uniforms.clone(), EasingType::Linear)
                    .add_key_frame(
                        0.9,
                        uniforms
                            .insert_float_ref("u_scale", 0.0)
                            .insert_vec_ref("u_offset", vec2(1.0, 0.0))
                            .clone(),
                        EasingType::CubicIn,
                    );
                node.add_effect(VisualEffect::new(
                    3.0,
                    VisualEffectType::ShaderAnimation {
                        shader: shader.clone(),
                        animation,
                    },
                    0,
                ));

                let mut uniforms = initial_uniforms.clone();
                let mut animation = AnimatedShaderUniforms::empty(EasingType::Linear)
                    .add_key_frame(0.2, uniforms.clone(), EasingType::Linear)
                    .add_key_frame(
                        0.5,
                        uniforms
                            .insert_vec_ref("u_offset", vec2(0.0, -0.1))
                            .insert_float_ref("u_scale", 1.0)
                            .clone(),
                        EasingType::CubicOut,
                    );
                if *score > 2 {
                    animation = animation.add_key_frame(
                        0.52,
                        uniforms
                            .insert_color_ref("u_color", options.colors.star)
                            .insert_float_ref("u_scale", 1.5)
                            .clone(),
                        EasingType::CubicIn,
                    );
                }
                animation = animation
                    .add_key_frame(
                        0.78,
                        uniforms.insert_float_ref("u_scale", 1.0).clone(),
                        EasingType::Linear,
                    )
                    .add_key_frame(0.8, uniforms.clone(), EasingType::Linear)
                    .add_key_frame(
                        0.9,
                        uniforms
                            .insert_float_ref("u_scale", 0.0)
                            .insert_vec_ref("u_offset", vec2(0.0, -1.0))
                            .clone(),
                        EasingType::CubicIn,
                    );
                node.add_effect(VisualEffect::new(
                    3.0,
                    VisualEffectType::ShaderAnimation {
                        shader: shader.clone(),
                        animation,
                    },
                    0,
                ));

                node
            }
        }
    }
}
