use super::*;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Widget {
    BattleOverPanel {
        score: usize,
    },
    BonusChoicePanel {
        bonuses: Vec<BonusEffect>,
        entity: legion::Entity,
    },
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
                let animation = AnimatedShaderUniforms::empty()
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
                node.add_effect(TimedEffect::new(
                    Some(3.0),
                    Animation::ShaderAnimation {
                        shader: shader.clone(),
                        animation,
                    },
                    0,
                ));

                uniforms.insert_color_ref("u_color", options.colors.background_dark);
                let animation = AnimatedShaderUniforms::empty()
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
                node.add_effect(TimedEffect::new(
                    Some(3.0),
                    Animation::ShaderAnimation { shader, animation },
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
                let animation = AnimatedShaderUniforms::empty()
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
                node.add_effect(TimedEffect::new(
                    Some(3.0),
                    Animation::ShaderAnimation { shader, animation },
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
                let mut animation = AnimatedShaderUniforms::empty()
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
                node.add_effect(TimedEffect::new(
                    Some(3.0),
                    Animation::ShaderAnimation {
                        shader: shader.clone(),
                        animation,
                    },
                    0,
                ));

                let mut uniforms = initial_uniforms.clone();
                let mut animation = AnimatedShaderUniforms::empty()
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
                node.add_effect(TimedEffect::new(
                    Some(3.0),
                    Animation::ShaderAnimation {
                        shader: shader.clone(),
                        animation,
                    },
                    0,
                ));

                let mut uniforms = initial_uniforms.clone();
                let mut animation = AnimatedShaderUniforms::empty()
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
                node.add_effect(TimedEffect::new(
                    Some(3.0),
                    Animation::ShaderAnimation {
                        shader: shader.clone(),
                        animation,
                    },
                    0,
                ));

                node
            }
            Widget::BonusChoicePanel { bonuses, entity } => {
                let duration = Some(1.0);
                let mut node = Node::default();
                let mut bg = options.shaders.choice_panel.clone();
                let position = bg.parameters.uniforms.try_get_vec2("u_position").unwrap();
                bg.entity = Some(new_entity());
                let initial_uniforms: ShaderUniforms = hashmap! {
                    "u_color" => ShaderUniform::Color(options.colors.background),
                    "u_box" => ShaderUniform::Vec2(vec2(1.0, 0.0)),
                }
                .into();

                let animation = AnimatedShaderUniforms::empty()
                    .add_key_frame(0.0, initial_uniforms.clone(), default())
                    .add_key_frame(
                        0.5,
                        initial_uniforms
                            .clone()
                            .insert_vec("u_box", vec2(1.0, 0.55)),
                        EasingType::QuadOut,
                    )
                    .add_key_frame(1.0, initial_uniforms.clone(), EasingType::BackIn);

                node.add_effect(TimedEffect::new(
                    duration,
                    Animation::ShaderAnimation {
                        shader: bg,
                        animation,
                    },
                    0,
                ));

                let mut shader = options.shaders.choice_panel_option.clone();
                shader.parent = Some(*entity);

                shader.input_handlers.push(ButtonSystem::button_handler);
                shader
                    .input_handlers
                    .push(|event, _, shader, world, resources| match event {
                        InputEvent::Click => {
                            if let Some(panel) = resources
                                .tape_player
                                .tape
                                .panels
                                .get_mut(&shader.parent.unwrap())
                            {
                                if panel.set_open(false, resources.tape_player.head) {
                                    let ind =
                                        shader.parameters.uniforms.try_get_int("u_index").unwrap()
                                            as usize;
                                    debug!("Panel make selection: {ind}");
                                    BonusEffectPool::make_selection(ind, world, resources);
                                }
                            }
                        }
                        _ => {}
                    });
                for (ind, bonus) in bonuses.iter().enumerate() {
                    let color = *options.colors.rarities.get(&bonus.rarity).unwrap();
                    let rarity = format!("{:?}", bonus.rarity);
                    let mut text = bonus.description.clone();
                    if let Some((_, target)) = &bonus.target {
                        text += format!(" to {target}").as_str();
                    }
                    let initial_uniforms: ShaderUniforms = hashmap! {
                        "u_color" => ShaderUniform::Color(color),
                        "u_position" => ShaderUniform::Vec2(position + vec2((ind as f32) * 0.1, ((bonuses.len() as f32 - 1.0) * 0.5 - ind as f32) * 0.25)),
                        "u_box" => ShaderUniform::Vec2(vec2(2.0, 0.0)),
                        "u_index" => ShaderUniform::Int(ind as i32),
                        "u_open" => ShaderUniform::Float(0.0),
                        "u_text" => ShaderUniform::String((0, text)),
                        "u_rarity_text" =>ShaderUniform::String((1, rarity.to_string())),
                        "u_text_color" => ShaderUniform::Color(options.colors.text),
                    }
                    .into();
                    let time_offset = ind as f32 * 0.1;
                    let animation = AnimatedShaderUniforms::empty()
                        .add_key_frame(time_offset.min(0.4), initial_uniforms.clone(), default())
                        .add_key_frame(
                            0.5,
                            initial_uniforms
                                .clone()
                                .insert_vec("u_box", vec2(2.0, 0.1))
                                .insert_float("u_open", 1.0),
                            EasingType::QuadOut,
                        )
                        .add_key_frame(
                            1.0 - time_offset.min(0.4),
                            initial_uniforms.clone(),
                            EasingType::QuadIn,
                        );

                    shader.entity = Some(new_entity());

                    node.add_effect(TimedEffect::new(
                        duration,
                        Animation::ShaderAnimation {
                            shader: shader.clone(),
                            animation,
                        },
                        0,
                    ));
                }

                node
            }
        }
    }
}
