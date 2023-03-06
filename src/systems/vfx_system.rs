use super::*;

pub struct VfxSystem {}

impl VfxSystem {
    pub fn vfx_show_text(
        resources: &Resources,
        text: &str,
        color: Rgba<f32>,
        position: vec2<f32>,
        font: usize,
    ) -> VisualEffect {
        VisualEffect::new(
            3.0,
            VisualEffectType::ShaderAnimation {
                shader: resources
                    .options
                    .text
                    .clone()
                    .set_uniform("u_position", ShaderUniform::Vec2(position))
                    .set_uniform("u_text", ShaderUniform::String((font, text.to_string())))
                    .set_uniform("u_outline_color", ShaderUniform::Color(color))
                    .set_uniform("u_alpha_over_t", ShaderUniform::Float(-0.8))
                    .set_uniform("u_scale", ShaderUniform::Float(0.6))
                    .set_uniform("u_position_over_t", ShaderUniform::Vec2(vec2(0.0, 4.5))),
                from: hashmap! {
                    "u_time" => ShaderUniform::Float(0.0),
                }
                .into(),
                to: hashmap! {
                    "u_time" => ShaderUniform::Float(1.0),
                }
                .into(),
                easing: EasingType::Linear,
            },
            0,
        )
    }

    pub fn vfx_strike_charge(entity: legion::Entity, faction: &Faction) -> VisualEffect {
        let faction_mul = match faction {
            Faction::Light => -1.0,
            Faction::Dark => 1.0,
            _ => panic!("Wrong faction"),
        };
        VisualEffect::new(
            1.5,
            VisualEffectType::EntityShaderAnimation {
                entity,
                from: hashmap! {
                    "u_position" => ShaderUniform::Vec2(vec2(1.5, 0.0) * faction_mul),
                }
                .into(),
                to: hashmap! {
                    "u_position" => ShaderUniform::Vec2(vec2(4.5, 0.0) * faction_mul),
                }
                .into(),
                easing: EasingType::QuartInOut,
            },
            20,
        )
    }

    pub fn vfx_strike_release(entity: legion::Entity, faction: &Faction) -> VisualEffect {
        let faction_mul = match faction {
            Faction::Light => -1.0,
            Faction::Dark => 1.0,
            _ => panic!("Wrong faction"),
        };
        VisualEffect::new(
            0.1,
            VisualEffectType::EntityShaderAnimation {
                entity,
                from: hashmap! {
                    "u_position" => ShaderUniform::Vec2(vec2(4.5, 0.0) * faction_mul),
                }
                .into(),
                to: hashmap! {
                    "u_position" => ShaderUniform::Vec2(vec2(1.0, 0.0) * faction_mul),
                }
                .into(),
                easing: EasingType::Linear,
            },
            20,
        )
    }

    pub fn vfx_strike_retract(entity: legion::Entity, faction: &Faction) -> VisualEffect {
        let faction_mul = match faction {
            Faction::Light => -1.0,
            Faction::Dark => 1.0,
            _ => panic!("Wrong faction"),
        };
        VisualEffect::new(
            0.25,
            VisualEffectType::EntityShaderAnimation {
                entity,
                from: hashmap! {
                    "u_position" => ShaderUniform::Vec2(vec2(1.0, 0.0) * faction_mul),
                }
                .into(),
                to: hashmap! {
                    "u_position" => ShaderUniform::Vec2(vec2(1.5, 0.0) * faction_mul),
                }
                .into(),
                easing: EasingType::QuartOut,
            },
            20,
        )
    }

    pub fn vfx_strike(resources: &Resources, position: vec2<f32>) -> VisualEffect {
        VisualEffect::new(
            0.5,
            VisualEffectType::ShaderAnimation {
                shader: resources.options.strike.clone(),
                from: hashmap! {
                    "u_time" => ShaderUniform::Float(0.0),
                    "u_position" => ShaderUniform::Vec2(position),
                }
                .into(),
                to: hashmap! {
                    "u_time" => ShaderUniform::Float(1.0),
                    "u_position" => ShaderUniform::Vec2(position),
                }
                .into(),
                easing: EasingType::Linear,
            },
            0,
        )
    }

    pub fn vfx_move_unit(entity: legion::Entity, from: vec2<f32>, to: vec2<f32>) -> VisualEffect {
        VisualEffect::new(
            0.2,
            VisualEffectType::EntityShaderAnimation {
                entity,
                from: hashmap! {"u_position" => ShaderUniform::Vec2(from)}.into(),
                to: hashmap! {"u_position" => ShaderUniform::Vec2(to)}.into(),
                easing: EasingType::QuartIn,
            },
            0,
        )
    }
}
