use super::*;

pub struct VfxSystem {}

/// Logic
impl VfxSystem {
    pub fn translate_animated(
        entity: legion::Entity,
        position: vec2<f32>,
        node: &mut Node,
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
            1.0,
            delay,
            VisualEffectType::ShaderAnimation {
                shader: Self::get_show_text_shader(resources, text, font, color, outline_color)
                    .set_uniform("u_position", ShaderUniform::Vec2(position)),
                animation: AnimatedShaderUniforms::empty(EasingType::QuartOut),
            },
            0,
        )
    }

    pub fn vfx_show_parent_text(
        resources: &Resources,
        text: &str,
        color: Rgba<f32>,
        outline_color: Rgba<f32>,
        parent: legion::Entity,
        font: usize,
        delay: Time,
    ) -> VisualEffect {
        VisualEffect::new_delayed(
            1.0,
            delay,
            VisualEffectType::EntityExtraShaderAnimation {
                entity: parent,
                shader: Self::get_show_text_shader(resources, text, font, color, outline_color),
                animation: AnimatedShaderUniforms::empty(EasingType::QuartOut),
            },
            0,
        )
    }

    fn get_show_text_shader(
        resources: &Resources,
        text: &str,
        font: usize,
        color: Rgba<f32>,
        outline_color: Rgba<f32>,
    ) -> Shader {
        resources
            .options
            .shaders
            .text
            .clone()
            .set_uniform("u_offset", ShaderUniform::Vec2(vec2(0.0, 0.5)))
            .set_uniform("u_offset_over_t", ShaderUniform::Vec2(vec2(0.0, 1.8)))
            .set_uniform("u_text", ShaderUniform::String((font, text.to_string())))
            .set_uniform("u_color", ShaderUniform::Color(color))
            .set_uniform("u_outline_color", ShaderUniform::Color(outline_color))
            .set_uniform("u_outline_fade", ShaderUniform::Float(1.0))
            .set_uniform("u_text_border", ShaderUniform::Float(0.1))
            .set_uniform("u_alpha", ShaderUniform::Float(8.0))
            .set_uniform("u_alpha_over_t", ShaderUniform::Float(-8.0))
            .set_uniform("u_size", ShaderUniform::Float(0.6))
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
                animation: default(),
            },
            0,
        )
    }

    pub fn vfx_show_curve(
        resources: &Resources,
        from: legion::Entity,
        to: legion::Entity,
        color: Rgba<f32>,
    ) -> VisualEffect {
        VisualEffect::new(
            1.0,
            VisualEffectType::EntityPairExtraShaderAnimation {
                entity_from: from,
                entity_to: to,
                shader: resources
                    .options
                    .shaders
                    .curve
                    .clone()
                    .set_uniform("u_color", ShaderUniform::Color(color)),
                animation: default(),
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
                animation: AnimatedShaderUniforms::from_to(
                    hashmap! {"u_position" => ShaderUniform::Vec2(from)}.into(),
                    hashmap! {"u_position" => ShaderUniform::Vec2(to)}.into(),
                    easing,
                ),
            },
            0,
        )
    }

    pub fn vfx_battle_team_names(resources: &Resources) -> (Shader, Shader) {
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
            .try_get_vec2(&VarName::Position.uniform())
            .unwrap()
            * vec2(-1.0, 1.0);
        let light_shader = shader
            .clone()
            .set_uniform("u_align", ShaderUniform::Float(-1.0))
            .set_uniform(&VarName::Position.uniform(), ShaderUniform::Vec2(light_pos))
            .set_uniform("u_text", ShaderUniform::String((2, light)));
        (light_shader, dark_shader)
    }
}
