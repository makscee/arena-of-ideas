use super::*;

pub struct StatsUiSystem {}

#[derive(Clone, Deserialize, Eq, PartialEq, Debug)]
pub enum StatType {
    Hp,
    Attack,
}

impl StatsUiSystem {
    pub fn get_entity_shaders(vars: &Vars, options: &Options) -> Vec<Shader> {
        let damage = vars.get_int(&VarName::HpDamage);
        let hp_value = vars.get_int(&VarName::HpValue) - damage;
        let hp_original_value = vars.get_int(&VarName::HpOriginalValue);
        let hp_modified = if damage > 0 {
            -1
        } else {
            if hp_value > hp_original_value {
                1
            } else {
                0
            }
        };
        let attack_value = vars.get_int(&VarName::AttackValue);
        let attack_original_value = vars.get_int(&VarName::AttackOriginalValue);
        let attack_modified = match attack_value > attack_original_value {
            true => 1,
            false => 0,
        };
        vec![
            options
                .shaders
                .stats
                .clone()
                .set_uniform("u_angle_offset", ShaderUniform::Float(1.0))
                .set_uniform("u_text", ShaderUniform::String((1, hp_value.to_string())))
                .set_uniform("u_value_modified", ShaderUniform::Int(hp_modified))
                .set_uniform("u_animate_on_damage", ShaderUniform::Float(1.0))
                .set_uniform(
                    "u_circle_color",
                    ShaderUniform::Color(options.colors.stats_health),
                ),
            options
                .shaders
                .stats
                .clone()
                .set_uniform("u_angle_offset", ShaderUniform::Float(-1.0))
                .set_uniform("u_value_modified", ShaderUniform::Int(attack_modified))
                .set_uniform(
                    "u_text",
                    ShaderUniform::String((1, attack_value.to_string())),
                )
                .set_uniform(
                    "u_circle_color",
                    ShaderUniform::Color(options.colors.stats_attack),
                ),
        ]
    }
}
