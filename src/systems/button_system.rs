use super::*;

pub struct ButtonSystem;

impl ButtonSystem {
    pub fn create_button(
        text: Option<&str>,
        icon: Option<Image>,
        input_handler: Handler,
        update_handler: Option<Handler>,
        entity: legion::Entity,
        options: &Options,
    ) -> Shader {
        let mut button = options.shaders.button.clone();
        Self::add_button_handlers(&mut button);
        button.input_handlers.push(input_handler);
        if let Some(update_handler) = update_handler {
            button.update_handlers.push(update_handler);
        }
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

    fn button_input_handler(
        event: HandleEvent,
        _: legion::Entity,
        shader: &mut Shader,
        _: &mut legion::World,
        resources: &mut Resources,
    ) {
        match event {
            HandleEvent::Hover => {
                shader.set_float_ref("u_hovered", 1.0);
            }
            HandleEvent::Press => {
                shader.set_float_ref("u_pressed", 1.0);
            }
            _ => {}
        };
    }

    fn button_update_handler(
        event: HandleEvent,
        _: legion::Entity,
        shader: &mut Shader,
        _: &mut legion::World,
        resources: &mut Resources,
    ) {
        if let Some(active) = shader.parameters.uniforms.try_get_float("u_active") {
            if active == 1.0 {
                shader.set_color_ref("u_color", resources.options.colors.button);
                if let Some(pressed) = shader.parameters.uniforms.try_get_float("u_pressed") {
                    if pressed == 1.0 {
                        shader.set_color_ref("u_color", resources.options.colors.pressed);
                    }
                } else if let Some(hovered) = shader.parameters.uniforms.try_get_float("u_hovered")
                {
                    if hovered == 1.0 {
                        shader.set_color_ref("u_color", resources.options.colors.hovered);
                    }
                }
            } else {
                shader.set_color_ref("u_color", resources.options.colors.inactive);
            }
        }
    }

    pub fn add_button_handlers(shader: &mut Shader) {
        shader.set_float_ref("u_active", 1.0);
        shader.input_handlers.push(Self::button_input_handler);
        shader.update_handlers.push(Self::button_update_handler);
    }
}
