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

                let (shader, animation, duration) = {
                    let anim = &options.widgets.battle_over_panel.bg_1;
                    (
                        anim.shader.clone().set_color("u_color", color),
                        anim.animation.clone(),
                        anim.duration,
                    )
                };
                node.add_effect(TimedEffect::new(
                    Some(duration),
                    Animation::ShaderAnimation { shader, animation },
                    0,
                ));

                let (shader, animation, duration) = {
                    let anim = &options.widgets.battle_over_panel.bg_2;
                    (
                        anim.shader
                            .clone()
                            .set_color("u_color", options.colors.background_dark),
                        anim.animation.clone(),
                        anim.duration,
                    )
                };
                node.add_effect(TimedEffect::new(
                    Some(duration),
                    Animation::ShaderAnimation { shader, animation },
                    0,
                ));

                let (shader, animation, duration) = {
                    let anim = &options.widgets.battle_over_panel.title;
                    (
                        anim.shader
                            .clone()
                            .set_color("u_color", color)
                            .set_string("u_text", text, 1),
                        anim.animation.clone(),
                        anim.duration,
                    )
                };
                node.add_effect(TimedEffect::new(
                    Some(duration),
                    Animation::ShaderAnimation { shader, animation },
                    0,
                ));

                let (shader, animation, duration) = {
                    let anim = &options.widgets.battle_over_panel.star;
                    let mut animation = anim.animation.clone();
                    let uniforms = animation.get_uniforms_mut();
                    uniforms.insert_vec2_ref("u_index_offset", vec2(-0.15, 0.0));
                    if *score > 0 {
                        uniforms
                            .insert_float_ref("u_active_scale", 1.5)
                            .insert_color_ref("u_active_color", options.colors.star);
                    } else {
                        uniforms
                            .insert_float_ref("u_active_scale", 1.0)
                            .insert_color_ref("u_active_color", options.colors.inactive);
                    }
                    (
                        anim.shader
                            .clone()
                            .set_color("u_color", options.colors.inactive),
                        animation,
                        anim.duration,
                    )
                };
                node.add_effect(TimedEffect::new(
                    Some(duration),
                    Animation::ShaderAnimation { shader, animation },
                    0,
                ));

                let (shader, animation, duration) = {
                    let anim = &options.widgets.battle_over_panel.star;
                    let mut animation = anim.animation.clone();
                    let uniforms = animation.get_uniforms_mut();
                    uniforms.insert_vec2_ref("u_index_offset", vec2(0.15, 0.0));
                    if *score > 1 {
                        uniforms
                            .insert_float_ref("u_active_scale", 1.5)
                            .insert_color_ref("u_active_color", options.colors.star);
                    } else {
                        uniforms
                            .insert_float_ref("u_active_scale", 1.0)
                            .insert_color_ref("u_active_color", options.colors.inactive);
                    }
                    (
                        anim.shader
                            .clone()
                            .set_color("u_color", options.colors.inactive),
                        animation,
                        anim.duration,
                    )
                };
                let delay = 0.2;
                node.add_effect(TimedEffect::new_delayed(
                    Some(duration - delay),
                    delay,
                    Animation::ShaderAnimation { shader, animation },
                    0,
                ));

                let (shader, animation, duration) = {
                    let anim = &options.widgets.battle_over_panel.star;
                    let mut animation = anim.animation.clone();
                    let uniforms = animation.get_uniforms_mut();
                    uniforms.insert_vec2_ref("u_index_offset", vec2(0.0, -0.1));
                    if *score > 2 {
                        uniforms
                            .insert_float_ref("u_active_scale", 1.5)
                            .insert_color_ref("u_active_color", options.colors.star);
                    } else {
                        uniforms
                            .insert_float_ref("u_active_scale", 1.0)
                            .insert_color_ref("u_active_color", options.colors.inactive);
                    }
                    (
                        anim.shader
                            .clone()
                            .set_color("u_color", options.colors.inactive),
                        animation,
                        anim.duration,
                    )
                };
                let delay = 0.4;
                node.add_effect(TimedEffect::new_delayed(
                    Some(duration - delay),
                    delay,
                    Animation::ShaderAnimation { shader, animation },
                    0,
                ));

                node
            }
            Widget::BonusChoicePanel { bonuses, entity } => {
                let mut node = Node::default();
                let mut position = vec2::ZERO;
                let (shader, animation, duration) = {
                    let anim = &options.widgets.bonus_choice_panel.bg;
                    position = anim
                        .shader
                        .parameters
                        .uniforms
                        .try_get_vec2("u_position")
                        .unwrap();
                    (
                        anim.shader
                            .clone()
                            .set_color("u_color", options.colors.background),
                        anim.animation.clone(),
                        anim.duration,
                    )
                };
                node.add_effect(TimedEffect::new(
                    Some(duration),
                    Animation::ShaderAnimation { shader, animation },
                    0,
                ));

                let (shader, animation, duration) = {
                    let anim = &options.widgets.bonus_choice_panel.option;
                    let mut shader = anim
                        .shader
                        .clone()
                        .set_color("u_color", options.colors.background);
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
                                        let ind = shader
                                            .parameters
                                            .uniforms
                                            .try_get_int("u_index")
                                            .unwrap()
                                            as usize;
                                        debug!("Panel make selection: {ind}");
                                        BonusEffectPool::make_selection(ind, world, resources);
                                    }
                                }
                            }
                            _ => {}
                        });
                    (shader, anim.animation.clone(), anim.duration)
                };

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
                        "u_index" => ShaderUniform::Int(ind as i32),
                        "u_text" => ShaderUniform::String((0, text)),
                        "u_rarity_text" =>ShaderUniform::String((1, rarity.to_string())),
                        "u_text_color" => ShaderUniform::Color(options.colors.text),
                    }
                    .into();
                    let mut shader = shader.clone();
                    shader.merge_uniforms_ref(&initial_uniforms, true);
                    shader.entity = Some(new_entity());
                    node.add_effect(TimedEffect::new(
                        Some(duration),
                        Animation::ShaderAnimation {
                            shader,
                            animation: animation.clone(),
                        },
                        0,
                    ));
                }

                node
            }
        }
    }
}
