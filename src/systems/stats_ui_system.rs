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
        let hp_modified = -damage.signum();
        let attack_value = vars.get_int(&VarName::AttackValue);
        vec![
            options
                .shaders
                .stats
                .clone()
                .set_uniform("u_angle_offset", ShaderUniform::Float(1.0))
                .set_uniform("u_text", ShaderUniform::String((0, hp_value.to_string())))
                .set_uniform("u_value_modified", ShaderUniform::Int(hp_modified))
                .set_uniform(
                    "u_circle_color",
                    ShaderUniform::Color(options.colors.stats_hp_color),
                ),
            options
                .shaders
                .stats
                .clone()
                .set_uniform("u_angle_offset", ShaderUniform::Float(-1.0))
                .set_uniform(
                    "u_text",
                    ShaderUniform::String((0, attack_value.to_string())),
                )
                .set_uniform(
                    "u_circle_color",
                    ShaderUniform::Color(options.colors.stats_attack_color),
                ),
        ]
    }
}
