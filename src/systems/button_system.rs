use super::*;

pub struct ButtonSystem;

impl ButtonSystem {
    pub fn create_button(
        text: Option<&str>,
        icon: Option<Image>,
        handler: Handler,
        entity: legion::Entity,
        options: &Options,
    ) -> Shader {
        let mut button = options.shaders.button.clone();
        button
            .input_handlers
            .extend([Self::button_handler, handler]);
        if let Some(text) = text {
            button.chain_after.push(
                options
                    .shaders
                    .button_text
                    .clone()
                    .set_uniform("u_color", ShaderUniform::Color(options.colors.text))
                    .set_uniform("u_text", ShaderUniform::String((1, text.to_string()))),
            );
        }
        if let Some(icon) = icon {
            button.chain_after.push(
                options
                    .shaders
                    .button_icon
                    .clone()
                    .set_uniform("u_texture", ShaderUniform::Texture(icon)),
            );
        }
        button
            .parameters
            .uniforms
            .insert_color_ref(&VarName::Color.uniform(), options.colors.button);
        button.entity = Some(entity);

        button
    }

    pub fn button_handler(
        event: InputEvent,
        _: legion::Entity,
        shader: &mut Shader,
        _: &mut legion::World,
        resources: &mut Resources,
    ) {
        match event {
            InputEvent::Hover => {
                shader
                    .parameters
                    .uniforms
                    .insert_color_ref(&VarName::Color.uniform(), resources.options.colors.hovered);
            }
            InputEvent::Press => {
                shader
                    .parameters
                    .uniforms
                    .insert_color_ref(&VarName::Color.uniform(), resources.options.colors.pressed);
            }
            _ => {}
        };
    }
}
