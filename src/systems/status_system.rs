use super::*;

pub struct StatusSystem {}

impl StatusSystem {
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
