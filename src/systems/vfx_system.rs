use super::*;

const TEXT_KEY: &str = "text";
const TEXT_DELAY_PER: Time = 0.2;
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
        let state = ContextState::get_mut(entity, world);
        let cur_pos = state.vars.get_vec2(&VarName::Position);
        node.add_effect(Self::vfx_move_unit(
            entity, cur_pos, position, easing, duration,
        ));
        state.vars.set_vec2(&VarName::Position, position);
    }
}

/// Effects Collection
impl VfxSystem {
    pub fn add_show_text_effect(
        text: &str,
        color: Rgba<f32>,
        position: vec2<f32>,
        world: &legion::World,
        resources: &mut Resources,
    ) {
        Effect::ShowText {
            text: text.to_owned(),
            color: Some(color),
            entity: None,
            font: 0,
        }
        .wrap()
        .push(
            Context::new(
                ContextLayer::Var {
                    var: VarName::Position,
                    value: Var::Vec2(position),
                },
                world,
                resources,
            ),
            resources,
        );
    }

    pub fn vfx_show_text(
        node: &mut Node,
        text: &str,
        color: Rgba<f32>,
        outline_color: Rgba<f32>,
        position: vec2<f32>,
        font: usize,
        resources: &Resources,
    ) {
        let delay = TEXT_DELAY_PER * node.key_effects_count(TEXT_KEY) as f32;
        let effect = TimedEffect::new_delayed(
            Some(1.0),
            delay,
            Animation::ShaderAnimation {
                shader: Self::get_show_text_shader(resources, text, font, color, outline_color)
                    .insert_uniform("u_position".to_owned(), ShaderUniform::Vec2(position)),
                animation: AnimatedShaderUniforms::empty(),
            },
            0,
        );
        node.add_effect_by_key(TEXT_KEY.to_owned(), effect);
    }

    pub fn vfx_show_parent_text(
        node: &mut Node,
        text: &str,
        color: Rgba<f32>,
        outline_color: Rgba<f32>,
        parent: legion::Entity,
        font: usize,
        sound: Option<SoundType>,
        resources: &Resources,
    ) {
        let delay = TEXT_DELAY_PER * node.key_effects_count(TEXT_KEY) as f32;
        let effect = TimedEffect::new_delayed(
            Some(1.0),
            delay,
            Animation::EntityExtraShaderAnimation {
                entity: parent,
                shader: Self::get_show_text_shader(resources, text, font, color, outline_color),
                animation: AnimatedShaderUniforms::empty(),
            },
            0,
        );
        node.add_effect_by_key(TEXT_KEY.to_owned(), effect);
        if let Some(sound) = sound {
            node.add_sound(sound, delay);
        }
    }

    fn get_show_text_shader(
        resources: &Resources,
        text: &str,
        font: usize,
        color: Rgba<f32>,
        outline_color: Rgba<f32>,
    ) -> ShaderChain {
        resources
            .options
            .shaders
            .text
            .clone()
            .insert_uniform("u_offset".to_owned(), ShaderUniform::Vec2(vec2(0.0, 0.5)))
            .insert_uniform(
                "u_position_over_t".to_owned(),
                ShaderUniform::Vec2(vec2(0.0, 1.8)),
            )
            .insert_uniform(
                "u_text".to_owned(),
                ShaderUniform::String((font, text.to_string())),
            )
            .insert_uniform(
                "u_mid_border_color".to_owned(),
                ShaderUniform::Color(resources.options.colors.outline),
            )
            .insert_float("u_outline_fade".to_owned(), 1.0)
            .insert_float("u_text_border".to_owned(), 0.1)
            .insert_float("u_alpha".to_owned(), 4.0)
            .insert_float("u_alpha_over_t".to_owned(), -4.0)
            .insert_float("u_size".to_owned(), 2.0)
            .insert_color("u_color".to_owned(), resources.options.colors.text)
            .insert_color("u_outline_color".to_owned(), outline_color)
            .remove_mapping("u_outline_color")
    }

    pub fn vfx_strike(resources: &Resources, position: vec2<f32>) -> TimedEffect {
        TimedEffect::new(
            Some(0.5),
            Animation::ShaderAnimation {
                shader: resources
                    .options
                    .shaders
                    .strike
                    .clone()
                    .insert_uniform("u_position".to_owned(), ShaderUniform::Vec2(position)),
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
    ) -> TimedEffect {
        TimedEffect::new(
            Some(1.0),
            Animation::EntityPairExtraShaderAnimation {
                entity_from: from,
                entity_to: to,
                shader: resources
                    .options
                    .shaders
                    .curve
                    .clone()
                    .insert_uniform("u_color".to_owned(), ShaderUniform::Color(color)),
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
    ) -> TimedEffect {
        TimedEffect::new(
            Some(duration),
            Animation::EntityShaderAnimation {
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

    pub fn vfx_battle_team_names(
        world: &legion::World,
        resources: &Resources,
    ) -> (ShaderChain, ShaderChain) {
        let light = TeamSystem::get_state(Faction::Light, world).name.clone();
        let dark = TeamSystem::get_state(Faction::Dark, world).name.clone();
        let shader = &resources.options.shaders.team_name;
        let dark_shader = shader
            .clone()
            .insert_uniform("u_text".to_owned(), ShaderUniform::String((2, dark)));
        let light_pos = shader
            .middle
            .parameters
            .uniforms
            .try_get_vec2(&VarName::Position.uniform())
            .unwrap()
            * vec2(-1.0, 1.0);
        let light_shader = shader
            .clone()
            .insert_uniform("u_align".to_owned(), ShaderUniform::Vec2(vec2(-1.0, 0.0)))
            .insert_uniform("u_position".to_owned(), ShaderUniform::Vec2(light_pos))
            .insert_uniform("u_text".to_owned(), ShaderUniform::String((2, light)));
        (light_shader, dark_shader)
    }
}
