use super::*;

pub enum Widget<'a> {
    Button {
        text: String,
        input_handler: Handler,
        update_handler: Option<Handler>,
        options: &'a Options,
        uniforms: ShaderUniforms,
        shader: Option<Shader>,
        hover_hints: Vec<(Rgba<f32>, String, String)>,
        entity: legion::Entity,
    },
}

impl<'a> Widget<'_> {
    pub fn generate_node(self) -> Node {
        match self {
            Self::Button {
                text,
                input_handler,
                update_handler,
                options,
                uniforms,
                shader,
                entity,
                hover_hints,
            } => {
                let button = ButtonSystem::create_button(
                    Some(&text),
                    input_handler,
                    update_handler,
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
