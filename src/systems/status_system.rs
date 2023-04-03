use super::*;

pub struct StatusSystem {}

impl StatusSystem {
    fn get_status_names_shaders(
        statuses: &HashMap<String, i32>,
        resources: &Resources,
    ) -> Vec<Shader> {
        statuses
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
                                0,
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
    fn get_definitions_shaders(
        names: &HashSet<String>,
        statuses: &HashMap<String, i32>,
        resources: &Resources,
    ) -> Vec<Shader> {
        let names: HashSet<&String> = HashSet::from_iter(names.iter().chain(statuses.keys()));
        names
            .iter()
            .sorted()
            .filter_map(|name| match resources.definitions.get(name) {
                Some(def) => Some(vec![
                    resources
                        .options
                        .shaders
                        .definitions_panel_title
                        .clone()
                        .set_uniform("u_text", ShaderUniform::String((2, name.deref().clone())))
                        .set_uniform("u_outline_color", ShaderUniform::Color(def.color)),
                    resources
                        .options
                        .shaders
                        .definitions_panel_text
                        .clone()
                        .set_uniform(
                            "u_text",
                            ShaderUniform::String((0, def.description.clone())),
                        ),
                ]),
                None => None,
            })
            .flatten()
            .collect_vec()
    }

    pub fn get_active_statuses_panel_effects(
        node: &Node,
        resources: &Resources,
    ) -> Vec<VisualEffect> {
        let mut effects: Vec<VisualEffect> = default();
        if let Some(entity) = resources.input.cur_hovered {
            let empty = HashMap::default();
            let statuses = node.get_entity_statuses(&entity).unwrap_or(&empty);
            let name_shaders = Self::get_status_names_shaders(statuses, resources);
            let empty = HashSet::default();
            let names = node.get_entity_definitions(&entity).unwrap_or(&empty);
            let definition_shaders = Self::get_definitions_shaders(names, statuses, resources);
            if !name_shaders.is_empty() {
                let shader = resources.options.shaders.status_panel.clone();
                effects.push(VisualEffect::new(
                    0.0,
                    VisualEffectType::EntityExtraShaderConst { entity, shader },
                    1000,
                ));
                for (ind, mut shader) in name_shaders.into_iter().enumerate() {
                    shader
                        .parameters
                        .uniforms
                        .0
                        .insert("u_index".to_string(), ShaderUniform::Int(ind as i32));
                    effects.push(VisualEffect::new(
                        0.0,
                        VisualEffectType::EntityExtraShaderConst { entity, shader },
                        1001,
                    ));
                }
            }
            if !definition_shaders.is_empty() {
                let shader = resources.options.shaders.definitions_panel.clone();
                effects.push(VisualEffect::new(
                    0.0,
                    VisualEffectType::EntityExtraShaderConst { entity, shader },
                    1001,
                ));
                for (ind, mut shader) in definition_shaders.into_iter().enumerate() {
                    shader
                        .parameters
                        .uniforms
                        .0
                        .insert("u_index".to_string(), ShaderUniform::Int(ind as i32 / 2));
                    effects.push(VisualEffect::new(
                        0.0,
                        VisualEffectType::EntityExtraShaderConst { entity, shader },
                        1002,
                    ))
                }
            }
        }
        effects
    }
}
