use super::*;

pub enum Widget<'a> {
    BattleOverPanel {
        score: usize,
        options: &'a Options,
    },
    BonusChoicePanel {
        value: usize,
        bonuses: Vec<BonusEffect>,
        panel_entity: legion::Entity,
        options: &'a Options,
    },
    Button {
        text: String,
        input_handler: Handler,
        update_handler: Option<Handler>,
        options: &'a Options,
        uniforms: ShaderUniforms,
        shader: Option<Shader>,
        entity: legion::Entity,
    },
}

impl<'a> Widget<'_> {
    pub fn generate_node(self) -> Node {
        match self {
            Self::Button {
                text,
                input_handler,
                update_handler,
                options,
                uniforms,
                shader,
                entity,
            } => {
                let button = ButtonSystem::create_button(
                    Some(&text),
                    None,
                    input_handler,
                    update_handler,
                    entity,
                    shader,
                    options,
                )
                .merge_uniforms(&uniforms, true);

                Node::new_panel_scaled(button)
            }
            Self::BattleOverPanel { score, options } => {
                let mut node = Node::default();
                let (text, color) = match score {
                    0 => (String::from("Defeat"), options.colors.defeat),
                    _ => (String::from("Victory"), options.colors.victory),
                };

                let (shader, animation, duration) = {
                    let anim = &options.widgets.battle_over_panel.bg_1;
                    (
                        anim.shader.clone().set_color("u_color".to_owned(), color),
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
                            .set_color("u_color".to_owned(), options.colors.background_dark),
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
                            .set_color("u_color".to_owned(), color)
                            .set_string("u_text".to_owned(), text, 1),
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
                    uniforms.insert_vec2_ref("u_index_offset".to_owned(), vec2(-0.15, 0.0));
                    if score > 0 {
                        uniforms
                            .insert_float_ref("u_active_scale".to_owned(), 1.5)
                            .insert_color_ref("u_active_color".to_owned(), options.colors.star);
                    } else {
                        uniforms
                            .insert_float_ref("u_active_scale".to_owned(), 1.0)
                            .insert_color_ref("u_active_color".to_owned(), options.colors.inactive);
                    }
                    (
                        anim.shader
                            .clone()
                            .set_color("u_color".to_owned(), options.colors.inactive),
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
                    uniforms.insert_vec2_ref("u_index_offset".to_owned(), vec2(0.15, 0.0));
                    if score > 1 {
                        uniforms
                            .insert_float_ref("u_active_scale".to_owned(), 1.5)
                            .insert_color_ref("u_active_color".to_owned(), options.colors.star);
                    } else {
                        uniforms
                            .insert_float_ref("u_active_scale".to_owned(), 1.0)
                            .insert_color_ref("u_active_color".to_owned(), options.colors.inactive);
                    }
                    (
                        anim.shader
                            .clone()
                            .set_color("u_color".to_owned(), options.colors.inactive),
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
                    uniforms.insert_vec2_ref("u_index_offset".to_owned(), vec2(0.0, -0.1));
                    if score > 2 {
                        uniforms
                            .insert_float_ref("u_active_scale".to_owned(), 1.5)
                            .insert_color_ref("u_active_color".to_owned(), options.colors.star);
                    } else {
                        uniforms
                            .insert_float_ref("u_active_scale".to_owned(), 1.0)
                            .insert_color_ref("u_active_color".to_owned(), options.colors.inactive);
                    }
                    (
                        anim.shader
                            .clone()
                            .set_color("u_color".to_owned(), options.colors.inactive),
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
            Self::BonusChoicePanel {
                bonuses,
                panel_entity: entity,
                options,
                value,
            } => {
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
                            .set_color("u_color".to_owned(), options.colors.background)
                            .set_string("u_value_text".to_owned(), value.to_string(), 1),
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
                    let mut shader = anim.shader.clone();
                    shader.parent = Some(entity);
                    ButtonSystem::add_button_handlers(&mut shader);
                    shader
                        .input_handlers
                        .push(|event, _, shader, world, resources| match event {
                            HandleEvent::Click => {
                                if resources.tape_player.tape.close_panels(
                                    shader.parent.unwrap(),
                                    resources.tape_player.head,
                                ) {
                                    let ind =
                                        shader.parameters.uniforms.try_get_int("u_index").unwrap()
                                            as usize;
                                    debug!("Panel make selection: {ind}");
                                    BonusEffectPool::make_selection(ind, world, resources);
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
                        "u_rarity_color" => ShaderUniform::Color(color),
                        "u_position" => ShaderUniform::Vec2(position + vec2(0.0, ((bonuses.len() as f32 - 1.0) * 0.5 - ind as f32) * 0.2)),
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
