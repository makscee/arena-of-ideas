use super::*;

pub struct VfxSystem {}

pub const TEAM_NAMES_KEY: &str = "team_names";

const BATTLE_INTRO_DURATION: Time = 0.6;

/// Logic
impl VfxSystem {
    pub fn translate_animated(
        entity: legion::Entity,
        position: vec2<f32>,
        node: &mut CassetteNode,
        world: &mut legion::World,
        easing: EasingType,
        duration: Time,
    ) {
        world.entry(entity).and_then(|mut x| {
            x.get_component_mut::<AreaComponent>()
                .and_then(|area| {
                    node.add_effect(Self::vfx_move_unit(
                        entity,
                        area.position,
                        position,
                        easing,
                        duration,
                    ));
                    area.position = position;
                    Ok(())
                })
                .ok()
        });
    }
}

/// Effects Collection
impl VfxSystem {
    pub fn vfx_show_text(
        resources: &Resources,
        text: &str,
        color: Rgba<f32>,
        outline_color: Rgba<f32>,
        position: vec2<f32>,
        font: usize,
        delay: Time,
    ) -> VisualEffect {
        VisualEffect::new_delayed(
            1.5,
            delay,
            VisualEffectType::ShaderAnimation {
                shader: resources
                    .options
                    .shaders
                    .text
                    .clone()
                    .set_uniform("u_position", ShaderUniform::Vec2(position))
                    .set_uniform("u_offset", ShaderUniform::Vec2(vec2(0.0, 0.5)))
                    .set_uniform("u_offset_over_t", ShaderUniform::Vec2(vec2(0.0, 1.8)))
                    .set_uniform("u_text", ShaderUniform::String((font, text.to_string())))
                    .set_uniform("u_color", ShaderUniform::Color(color))
                    .set_uniform("u_outline_color", ShaderUniform::Color(outline_color))
                    .set_uniform("u_outline_fade", ShaderUniform::Float(1.0))
                    .set_uniform("u_text_border", ShaderUniform::Float(0.1))
                    .set_uniform("u_alpha", ShaderUniform::Float(8.0))
                    .set_uniform("u_alpha_over_t", ShaderUniform::Float(-8.0))
                    .set_uniform("u_scale", ShaderUniform::Float(0.6)),
                from: default(),
                to: default(),
                easing: EasingType::QuartOut,
            },
            0,
        )
    }

    pub fn vfx_strike(resources: &Resources, position: vec2<f32>) -> VisualEffect {
        VisualEffect::new(
            0.5,
            VisualEffectType::ShaderAnimation {
                shader: resources
                    .options
                    .shaders
                    .strike
                    .clone()
                    .set_uniform("u_position", ShaderUniform::Vec2(position)),
                from: default(),
                to: default(),
                easing: EasingType::Linear,
            },
            0,
        )
    }

    pub fn vfx_show_curve(
        resources: &Resources,
        from: vec2<f32>,
        to: vec2<f32>,
        color: Rgba<f32>,
    ) -> VisualEffect {
        VisualEffect::new(
            1.0,
            VisualEffectType::ShaderAnimation {
                shader: resources
                    .options
                    .shaders
                    .curve
                    .clone()
                    .set_uniform("u_color", ShaderUniform::Color(color))
                    .set_uniform("u_from", ShaderUniform::Vec2(from))
                    .set_uniform("u_to", ShaderUniform::Vec2(to)),
                from: default(),
                to: default(),
                easing: EasingType::Linear,
            },
            0,
        )
    }

    pub fn vfx_move_unit(
        entity: legion::Entity,
        from: vec2<f32>,
        to: vec2<f32>,
        easing: EasingType,
        duration: Time,
    ) -> VisualEffect {
        VisualEffect::new(
            duration,
            VisualEffectType::EntityShaderAnimation {
                entity,
                from: hashmap! {"u_position" => ShaderUniform::Vec2(from)}.into(),
                to: hashmap! {"u_position" => ShaderUniform::Vec2(to)}.into(),
                easing,
            },
            0,
        )
    }

    pub fn vfx_battle_team_names_animation(resources: &Resources) -> Vec<VisualEffect> {
        let light = resources
            .team_states
            .get_team_state(&Faction::Light)
            .name
            .clone();
        let dark = resources
            .team_states
            .get_team_state(&Faction::Dark)
            .name
            .clone();
        let mut effects: Vec<VisualEffect> = default();
        let from = &resources.options.shaders.team_name_intro;
        let to = &resources.options.shaders.team_name;
        let mut from_vars = from.parameters.uniforms.clone();
        let to_vars = to.parameters.uniforms.clone();
        from_vars.merge_mut(&to_vars, false);
        effects.push(VisualEffect::new(
            BATTLE_INTRO_DURATION,
            VisualEffectType::ShaderAnimation {
                shader: from
                    .clone()
                    .set_uniform("u_text", ShaderUniform::String((2, dark.clone()))),
                from: from_vars.clone(),
                to: to_vars.clone(),
                easing: EasingType::QuartInOut,
            },
            0,
        ));
        let key = &VarName::Position.convert_to_uniform();
        let mut from_vars = from.parameters.uniforms.clone();
        let mut to_vars = to.parameters.uniforms.clone();
        to_vars.insert("u_align".to_string(), ShaderUniform::Float(-1.0));
        from_vars.merge_mut(&to_vars, false);
        from_vars.0.insert(
            key.clone(),
            ShaderUniform::Vec2(
                from_vars.get_vec2(key).unwrap() * vec2(-1.0, 1.0) + vec2(0.0, -2.0),
            ),
        );
        to_vars.0.insert(
            key.clone(),
            ShaderUniform::Vec2(to_vars.get_vec2(key).unwrap() * vec2(-1.0, 1.0)),
        );
        effects.push(VisualEffect::new(
            BATTLE_INTRO_DURATION,
            VisualEffectType::ShaderAnimation {
                shader: from
                    .clone()
                    .set_uniform("u_align", ShaderUniform::Float(-1.0))
                    .set_uniform("u_text", ShaderUniform::String((2, light.clone()))),
                from: from_vars,
                to: to_vars,
                easing: EasingType::QuartInOut,
            },
            0,
        ));
        effects
    }

    pub fn vfx_battle_team_names(resources: &Resources) -> Vec<VisualEffect> {
        if resources.transition_state == GameState::Shop {
            return default();
        }
        let light = resources
            .team_states
            .get_team_state(&Faction::Light)
            .name
            .clone();
        let dark = resources
            .team_states
            .get_team_state(&Faction::Dark)
            .name
            .clone();
        let shader = &resources.options.shaders.team_name;
        let dark_shader = shader
            .clone()
            .set_uniform("u_text", ShaderUniform::String((2, dark)));
        let light_pos = shader
            .parameters
            .uniforms
            .get_vec2(&VarName::Position.convert_to_uniform())
            .unwrap()
            * vec2(-1.0, 1.0);
        let light_shader = shader
            .clone()
            .set_uniform("u_align", ShaderUniform::Float(-1.0))
            .set_uniform(
                &VarName::Position.convert_to_uniform(),
                ShaderUniform::Vec2(light_pos),
            )
            .set_uniform("u_text", ShaderUniform::String((2, light)));
        vec![
            VisualEffect::new(
                0.0,
                VisualEffectType::ShaderConst {
                    shader: dark_shader,
                },
                0,
            ),
            VisualEffect::new(
                0.0,
                VisualEffectType::ShaderConst {
                    shader: light_shader,
                },
                0,
            ),
        ]
    }
}
