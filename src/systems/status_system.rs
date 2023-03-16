use super::*;

pub struct StatusSystem {}

impl StatusSystem {
    fn get_status_names_shaders(entity: legion::Entity, resources: &Resources) -> Vec<Shader> {
        resources
            .status_pool
            .active_statuses
            .get(&entity)
            .iter()
            .map(|x| x.keys().collect_vec())
            .flatten()
            .map(|name| {
                resources
                    .options
                    .shaders
                    .text
                    .clone()
                    .set_uniform("u_text", ShaderUniform::String((1, name.clone())))
            })
            .collect_vec()
    }

    pub fn add_active_statuses_panel_to_node(node: &mut CassetteNode, resources: &Resources) {
        if let Some(hovered) = resources.input.cur_hovered {
            let statuses = node.get_entity_statuses_names(&hovered);
            if !statuses.is_empty() {
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
                )
            }
        }
    }
}
