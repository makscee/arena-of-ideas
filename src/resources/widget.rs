use super::*;

pub enum Widget<'a> {
    Button {
        text: String,
        color: Option<Rgba<f32>>,
        input_handler: Handler,
        update_handler: Option<Handler>,
        pre_update_handler: Option<Handler>,
        options: &'a Options,
        uniforms: ShaderUniforms,
        shader: Option<ShaderChain>,
        hover_hints: Vec<(Rgba<f32>, String, String)>,
        entity: legion::Entity,
    },
}

impl<'a> Widget<'_> {
    pub fn generate_node(self) -> Node {
        match self {
            Self::Button {
                text,
                color,
                input_handler,
                update_handler,
                options,
                uniforms,
                shader,
                entity,
                hover_hints,
                pre_update_handler,
            } => {
                let button = ButtonSystem::create_button(
                    Some(&text),
                    color,
                    input_handler,
                    update_handler,
                    pre_update_handler,
                    entity,
                    shader,
                    hover_hints,
                    options,
                )
                .merge_uniforms(&uniforms, true);

                Node::new_panel_scaled(button)
            }
        }
    }
}
