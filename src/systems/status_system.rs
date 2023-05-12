use super::*;

pub struct StatusSystem {}

impl StatusSystem {
    fn get_status_names_shaders(
        statuses: &Vec<(String, i32)>,
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
        statuses: &Vec<(String, i32)>,
        resources: &Resources,
    ) -> Vec<Shader> {
        let names: HashSet<&String> =
            HashSet::from_iter(names.iter().chain(statuses.iter().map(|x| &x.0)));
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
    ) -> Vec<TimedEffect> {
        let mut effects: Vec<TimedEffect> = default();
        if let Some(entity) = resources.input_data.frame_data.1.get_hovered() {
            let empty = Vec::default();
            let statuses = node.get_entity_statuses(&entity).unwrap_or(&empty);
            let name_shaders = Self::get_status_names_shaders(statuses, resources);
            let empty = HashSet::default();
            let names = node.get_entity_definitions(&entity).unwrap_or(&empty);
            let definition_shaders = Self::get_definitions_shaders(names, statuses, resources);
            if !name_shaders.is_empty() {
                let shader = resources.options.shaders.status_panel.clone();
                effects.push(TimedEffect::new(
                    None,
                    Animation::EntityExtraShaderConst { entity, shader },
                    1000,
                ));
                for (ind, mut shader) in name_shaders.into_iter().enumerate() {
                    shader
                        .parameters
                        .uniforms
                        .insert_int_ref("u_index", ind as i32);
                    effects.push(TimedEffect::new(
                        None,
                        Animation::EntityExtraShaderConst { entity, shader },
                        1001,
                    ));
                }
            }
            if !definition_shaders.is_empty() {
                let shader = resources.options.shaders.definitions_panel.clone();
                effects.push(TimedEffect::new(
                    None,
                    Animation::EntityExtraShaderConst { entity, shader },
                    1001,
                ));
                for (ind, mut shader) in definition_shaders.into_iter().enumerate() {
                    shader
                        .parameters
                        .uniforms
                        .insert_int_ref("u_index", ind as i32 / 2);
                    effects.push(TimedEffect::new(
                        None,
                        Animation::EntityExtraShaderConst { entity, shader },
                        1002,
                    ))
                }
            }
        }
        effects
    }

    pub fn unpack_into_state(state: &mut ContextState, statuses: &Vec<(String, i32)>) {
        for (name, charges) in statuses.iter() {
            *state.statuses.entry(name.to_owned()).or_default() += *charges;
            state.t += 1;
            state.status_change_t.insert(name.to_owned(), state.t);
        }
    }

    pub fn pack_state_into_vec(state: &ContextState) -> Vec<(String, i32)> {
        Vec::from_iter(
            state
                .statuses
                .clone()
                .into_iter()
                .sorted_by_key(|(name, _)| state.status_change_t.get(name).unwrap_or(&0)),
        )
    }
}
