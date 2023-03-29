use super::*;

pub struct StatusSystem {}

const PANELS_KEY: &str = "panels";
impl StatusSystem {
    fn get_status_names_shaders(names: &Vec<(String, i32)>, resources: &Resources) -> Vec<Shader> {
        names
            .iter()
            .filter_map(|(name, charges)| match resources.definitions.get(name) {
                Some(def) => Some(
                    resources
                        .options
                        .shaders
                        .status_panel_text
                        .clone()
                        .set_uniform(
                            "u_text",
                            ShaderUniform::String((
                                1,
                                match *charges > 1 {
                                    true => format!("{} ({})", name, charges),
                                    false => name.clone(),
                                },
                            )),
                        )
                        .set_uniform("u_outline_color", ShaderUniform::Color(def.color)),
                ),
                None => None,
            })
            .collect_vec()
    }
    fn get_definitions_shaders(names: &Vec<(String, i32)>, resources: &Resources) -> Vec<Shader> {
        names
            .iter()
            .filter_map(|(name, _)| match resources.definitions.get(name) {
                Some(def) => Some(vec![
                    resources
                        .options
                        .shaders
                        .definitions_panel_title
                        .clone()
                        .set_uniform("u_text", ShaderUniform::String((2, name.clone())))
                        .set_uniform("u_outline_color", ShaderUniform::Color(def.color)),
                    resources
                        .options
                        .shaders
                        .definitions_panel_text
                        .clone()
                        .set_uniform(
                            "u_text",
                            ShaderUniform::String((1, def.description.clone())),
                        ),
                ]),
                None => None,
            })
            .flatten()
            .collect_vec()
    }

    fn get_status_names(
        statuses: &HashMap<legion::Entity, HashMap<String, i32>>,
        entity: &legion::Entity,
    ) -> Vec<(String, i32)> {
        statuses
            .get(entity)
            .and_then(|statuses| {
                Some(
                    statuses
                        .iter()
                        .map(|(name, charges)| (name.clone(), *charges))
                        .collect_vec(),
                )
            })
            .unwrap_or_else(|| vec![])
    }

    pub fn get_active_statuses_panel_effects(
        statuses: &HashMap<legion::Entity, HashMap<String, i32>>,
        resources: &Resources,
    ) -> Vec<VisualEffect> {
        let mut effects: Vec<VisualEffect> = default();
        if let Some(hovered) = resources.input.cur_hovered {
            let names = Self::get_status_names(statuses, &hovered);
            let name_shaders = Self::get_status_names_shaders(&names, resources);
            let definition_shaders = Self::get_definitions_shaders(&names, resources);
            if !name_shaders.is_empty() {
                let shader = resources.options.shaders.status_panel.clone();
                effects.push(VisualEffect::new(
                    0.0,
                    VisualEffectType::EntityExtraShaderConst {
                        entity: hovered,
                        shader,
                    },
                    1000,
                ));
                let shader = resources.options.shaders.definitions_panel.clone();
                effects.push(VisualEffect::new(
                    0.0,
                    VisualEffectType::EntityExtraShaderConst {
                        entity: hovered,
                        shader,
                    },
                    1001,
                ));
                for (ind, mut shader) in name_shaders.into_iter().enumerate() {
                    shader
                        .parameters
                        .uniforms
                        .0
                        .insert("u_index".to_string(), ShaderUniform::Int(ind as i32));
                    effects.push(VisualEffect::new(
                        0.0,
                        VisualEffectType::EntityExtraShaderConst {
                            entity: hovered,
                            shader,
                        },
                        1001,
                    ));
                }
                for (ind, mut shader) in definition_shaders.into_iter().enumerate() {
                    shader
                        .parameters
                        .uniforms
                        .0
                        .insert("u_index".to_string(), ShaderUniform::Int(ind as i32 / 2));
                    effects.push(VisualEffect::new(
                        0.0,
                        VisualEffectType::EntityExtraShaderConst {
                            entity: hovered,
                            shader,
                        },
                        1002,
                    ))
                }
            }
        }
        effects
    }
}
