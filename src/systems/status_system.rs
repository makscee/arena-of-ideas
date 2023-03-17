use super::*;

pub struct StatusSystem {}

impl StatusSystem {
    fn get_status_names_shaders(
        entity: legion::Entity,
        resources: &Resources,
        node: &CassetteNode,
    ) -> Vec<Shader> {
        node.get_entity_statuses_names(&entity)
            .into_iter()
            .chain(vec!["Shield".to_string(), "Grow".to_string()])
            .filter_map(|name| match resources.definitions.get(&name) {
                Some(def) => Some(
                    resources
                        .options
                        .shaders
                        .status_panel_text
                        .clone()
                        .set_uniform("u_text", ShaderUniform::String((1, name.clone())))
                        .set_uniform("u_outline_color", ShaderUniform::Color(def.color)),
                ),
                None => None,
            })
            .collect_vec()
    }

    pub fn add_active_statuses_panel_to_node(node: &mut CassetteNode, resources: &Resources) {
        if let Some(hovered) = resources.input.cur_hovered {
            let name_shaders = Self::get_status_names_shaders(hovered, resources, node);
            if !name_shaders.is_empty() {
                node.clear_key("statuses");
                node.add_effect_by_key(
                    "statuses",
                    VisualEffect::new(
                        0.0,
                        VisualEffectType::EntityExtraShaderConst {
                            entity: hovered,
                            shader: resources.options.shaders.description_panel.clone(),
                        },
                        1000,
                    ),
                );
                for (ind, mut shader) in name_shaders.into_iter().enumerate() {
                    shader
                        .parameters
                        .uniforms
                        .0
                        .insert("u_index".to_string(), ShaderUniform::Int(ind as i32));
                    node.add_effect_by_key(
                        "statuses",
                        VisualEffect::new(
                            0.0,
                            VisualEffectType::EntityExtraShaderConst {
                                entity: hovered,
                                shader,
                            },
                            1001,
                        ),
                    );
                }
            }
        }
    }
}
