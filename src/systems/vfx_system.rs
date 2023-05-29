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
        resources: &Resources,
        text: &str,
        color: Rgba<f32>,
        outline_color: Rgba<f32>,
        position: vec2<f32>,
        font: usize,
        delay: Time,
    ) -> TimedEffect {
        TimedEffect::new_delayed(
            Some(1.0),
            delay,
            Animation::ShaderAnimation {
                shader: Self::get_show_text_shader(resources, text, font, color, outline_color)
                    .set_uniform("u_position", ShaderUniform::Vec2(position)),
                animation: AnimatedShaderUniforms::empty(),
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
    ) -> TimedEffect {
        TimedEffect::new_delayed(
            Some(1.0),
            delay,
            Animation::EntityExtraShaderAnimation {
                entity: parent,
                shader: Self::get_show_text_shader(resources, text, font, color, outline_color),
                animation: AnimatedShaderUniforms::empty(),
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
            .set_uniform("u_position_over_t", ShaderUniform::Vec2(vec2(0.0, 1.8)))
            .set_uniform("u_text", ShaderUniform::String((font, text.to_string())))
            .set_uniform("u_mid_border_color", ShaderUniform::Color(color))
            .set_uniform("u_outline_color", ShaderUniform::Color(outline_color))
            .set_uniform("u_outline_fade", ShaderUniform::Float(1.0))
            .set_uniform("u_text_border", ShaderUniform::Float(0.1))
            .set_uniform("u_alpha", ShaderUniform::Float(8.0))
            .set_uniform("u_alpha_over_t", ShaderUniform::Float(-8.0))
            .set_vec2("u_box", vec2(3.0, 1.0))
            .set_float("u_size", 1.0)
            .set_color("u_color", resources.options.colors.text)
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

    pub fn vfx_battle_team_names(world: &legion::World, resources: &Resources) -> (Shader, Shader) {
        let light = TeamSystem::get_state(&Faction::Light, world).name.clone();
        let dark = TeamSystem::get_state(&Faction::Dark, world).name.clone();
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
            .set_uniform("u_align", ShaderUniform::Vec2(vec2(-1.0, 0.0)))
            .set_uniform(&VarName::Position.uniform(), ShaderUniform::Vec2(light_pos))
            .set_uniform("u_text", ShaderUniform::String((2, light)));
        (light_shader, dark_shader)
    }

    pub fn vfx_show_stars_indicator_panel(resources: &mut Resources) {
        fn update_handler(
            _: HandleEvent,
            _: legion::Entity,
            shader: &mut Shader,
            world: &mut legion::World,
            _: &mut Resources,
        ) {
            let value =
                TeamSystem::get_state(&Faction::Team, world).get_int(&VarName::Stars, world);
            shader.set_string_ref("u_text", format!("x{value}"), 1);
        }
        let mut shader = resources
            .options
            .shaders
            .count_indicator
            .clone()
            .set_float("u_star_enabled", 1.0)
            .set_float("u_g_enabled", 0.0)
            .set_int("u_index", 0);
        shader.entity = Some(new_entity());
        shader.update_handlers.push(update_handler);
        Node::new_panel_scaled(shader)
            .lock(NodeLockType::Empty)
            .push_as_panel(new_entity(), resources);
    }

    pub fn vfx_show_g_indicator_panel(resources: &mut Resources) {
        fn update_handler(
            _: HandleEvent,
            _: legion::Entity,
            shader: &mut Shader,
            world: &mut legion::World,
            _: &mut Resources,
        ) {
            let value = ShopSystem::get_g(world);
            shader.set_string_ref("u_text", format!("x{value}"), 1);
        }
        let mut shader = resources
            .options
            .shaders
            .count_indicator
            .clone()
            .set_float("u_star_enabled", 0.0)
            .set_float("u_g_enabled", 1.0)
            .set_int("u_index", 1);
        shader.entity = Some(new_entity());
        shader.update_handlers.push(update_handler);
        Node::new_panel_scaled(shader)
            .lock(NodeLockType::Empty)
            .push_as_panel(new_entity(), resources);
    }
}
