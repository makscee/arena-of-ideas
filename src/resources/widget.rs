use geng::prelude::itertools::Itertools;

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
    BattleChoicePanel {
        unit: &'a PackedUnit,
        difficulty: usize,
        name: String,
        panel_entity: legion::Entity,
        resources: &'a Resources,
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
    CardChoicePanel {
        title: String,
        cards: Vec<Shader>,
        entity: legion::Entity,
        input_handler: Handler,
        update_handler: Option<Handler>,
        resources: &'a Resources,
    },
}

impl<'a> Widget<'_> {
    pub fn generate_node(self) -> Node {
        match self {
            Self::CardChoicePanel {
                title,
                cards,
                input_handler,
                update_handler,
                resources,
                entity,
            } => {
                debug!(
                    "Card choice panel {}",
                    cards
                        .iter()
                        .map(|x| x.parameters.uniforms.try_get_string("u_name").unwrap())
                        .join(" ")
                );
                let mut node: Node = default();

                // card
                let (shader, animation, duration) = {
                    let anim = &resources.options.widgets.card_choice_panel.card_bg;
                    let shader = anim.shader.clone();
                    let animation = anim.animation.clone();
                    let duration = anim.duration;
                    (shader, animation, duration)
                };
                for (i, card) in cards.into_iter().enumerate() {
                    let mut shader = shader.clone();
                    let rarity = HeroPool::rarity_by_name(
                        &card.parameters.uniforms.try_get_string("u_name").unwrap(),
                        resources,
                    )
                    .color(resources);
                    shader.chain_after.push(card);
                    shader.set_int_ref("u_index", i as i32);
                    shader.set_color_ref("u_rarity_color", rarity);
                    shader.entity = Some(new_entity());
                    shader.parent = Some(entity);

                    ButtonSystem::add_button_handlers(&mut shader);
                    shader.input_handlers.push(input_handler);
                    if let Some(update_handler) = update_handler {
                        shader.update_handlers.push(update_handler);
                    }
                    node.add_effect(TimedEffect::new(
                        Some(duration),
                        Animation::ShaderAnimation {
                            shader,
                            animation: animation.clone(),
                        },
                        0,
                    ));
                }

                // panel
                let (shader, animation, duration) = {
                    let anim = &resources.options.widgets.card_choice_panel.panel;
                    let shader = anim.shader.clone().set_string("u_title", title, 1);
                    let animation = anim.animation.clone();
                    let duration = anim.duration;
                    (shader, animation, duration)
                };
                node.add_effect(TimedEffect::new(
                    Some(duration),
                    Animation::ShaderAnimation {
                        shader,
                        animation: animation.clone(),
                    },
                    0,
                ));

                // reroll button
                let shader = resources
                    .options
                    .widgets
                    .card_choice_panel
                    .reroll_btn
                    .clone();
                fn reroll_handler(
                    event: HandleEvent,
                    _: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            if ShopSystem::is_reroll_affordable(world)
                                && resources
                                    .tape_player
                                    .tape
                                    .close_panels(shader.parent.unwrap(), 0.0)
                            {
                                ShopSystem::reroll_hero_panel(world, resources);
                                debug!("Reroll hero buy panel");
                            }
                        }
                        _ => {}
                    }
                }
                fn reroll_update_handler(
                    _: HandleEvent,
                    _: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    _: &mut Resources,
                ) {
                    shader.set_active(ShopSystem::is_reroll_affordable(world));
                }
                let mut shader = ButtonSystem::create_button(
                    None,
                    None,
                    reroll_handler,
                    Some(reroll_update_handler),
                    new_entity(),
                    Some(shader),
                    &resources.options,
                );
                shader.parent = Some(entity);
                node.add_effect(TimedEffect::new(
                    Some(duration),
                    Animation::ShaderAnimation {
                        shader,
                        animation: animation.clone(),
                    },
                    0,
                ));

                node.lock(NodeLockType::Empty)
            }
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
            Self::BattleChoicePanel {
                unit,
                difficulty,
                resources,
                name,
                panel_entity,
            } => {
                let mut node = Node::default();
                let rarity_color = *resources
                    .options
                    .colors
                    .rarities
                    .get(&enum_iterator::all::<Rarity>().collect_vec()[difficulty])
                    .unwrap();
                let (mut shader, animation, duration) = {
                    let anim = &resources.options.widgets.battle_choice_panel.bg;

                    let mut shader = anim
                        .shader
                        .clone()
                        .set_color("u_color", rarity_color)
                        .set_color("u_outline_color", rarity_color)
                        .set_int("u_difficulty", difficulty as i32);
                    ButtonSystem::add_button_handlers(&mut shader);
                    shader
                        .input_handlers
                        .push(|event, _, shader, world, resources| match event {
                            HandleEvent::Click => {
                                if resources.tape_player.tape.close_panels(
                                    shader.parent.unwrap(),
                                    resources.tape_player.head,
                                ) {
                                    let difficulty = shader
                                        .parameters
                                        .uniforms
                                        .try_get_int("u_difficulty")
                                        .unwrap()
                                        as usize;
                                    debug!("Battle choice make selection: {difficulty}");
                                    let dark =
                                        Ladder::get_current_teams(resources)[difficulty].clone();
                                    let light = PackedTeam::pack(&Faction::Team, world, resources);
                                    resources.battle_data.last_difficulty = difficulty;
                                    TeamSystem::get_state_mut(&Faction::Team, world)
                                        .vars
                                        .change_int(&VarName::Stars, difficulty as i32 + 1);
                                    BattleSystem::init_ladder_battle(
                                        &light, dark, world, resources,
                                    );
                                    GameStateSystem::set_transition(GameState::Battle, resources);
                                }
                            }
                            _ => {}
                        });
                    shader.parent = Some(panel_entity);
                    shader.entity = Some(new_entity());
                    let position = shader
                        .parameters
                        .uniforms
                        .try_get_vec2(&VarName::Position.uniform())
                        .unwrap()
                        + vec2(
                            shader
                                .parameters
                                .uniforms
                                .try_get_float("u_offset_each")
                                .unwrap()
                                * difficulty as f32,
                            0.0,
                        );
                    let mut animation = anim.animation.clone();
                    animation
                        .get_uniforms_mut()
                        .insert_vec2_ref("u_open_position", position);
                    let mut unit_shader = unit
                        .get_ui_shader(Faction::Dark, resources)
                        .set_float("u_scale", 0.35);
                    shader.chain_after.push(unit_shader);
                    shader.order += difficulty as i32 * 3;
                    (shader, animation, anim.duration)
                };

                let name_shader = resources
                    .options
                    .widgets
                    .battle_choice_panel
                    .name
                    .clone()
                    .set_string("u_text", name, 1)
                    .set_color("u_mid_border_color", rarity_color);
                shader.chain_after.push(name_shader);

                let star_shader = &resources.options.widgets.battle_choice_panel.star;
                for i in 0..3 {
                    let color = if difficulty >= i {
                        resources.options.colors.star
                    } else {
                        resources.options.colors.inactive
                    };
                    let star_shader = star_shader
                        .clone()
                        .set_color(&VarName::Color.uniform(), color)
                        .set_int("u_index", i as i32);
                    shader.chain_after.push(star_shader);
                }
                node.add_effect(TimedEffect::new(
                    Some(duration),
                    Animation::ShaderAnimation {
                        shader,
                        animation: animation.clone(),
                    },
                    0,
                ));

                node.lock(NodeLockType::Empty)
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
                    if score > 0 {
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
                    if score > 1 {
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
                    if score > 2 {
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
                            .set_color("u_color", options.colors.background)
                            .set_string("u_value_text", value.to_string(), 1),
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
