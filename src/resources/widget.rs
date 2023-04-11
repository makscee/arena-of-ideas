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
                    "u_ui" => ShaderUniform::Float(1.0),
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
                        EasingType::CubicOut,
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
                    .add_key_frame(0.75, uniforms.clone(), EasingType::CubicOut)
                    .add_key_frame(
                        0.9,
                        uniforms.insert_vec_ref("u_box", vec2(1.0, 0.0)).clone(),
                        EasingType::CubicOut,
                    );
                node.add_effect(VisualEffect::new(
                    3.0,
                    VisualEffectType::ShaderAnimation { shader, animation },
                    0,
                ));

                let mut uniforms: ShaderUniforms = hashmap! {
                    "u_color" => ShaderUniform::Color(color),
                    "u_text" => ShaderUniform::String((1, text)),
                    "u_ui" => ShaderUniform::Float(1.0),
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

                node
            }
        }
    }
}
