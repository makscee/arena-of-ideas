use super::*;

pub struct ButtonSystem;

impl ButtonSystem {
    pub fn create_button(
        text: Option<&str>,
        input_handler: Handler,
        update_handler: Option<Handler>,
        entity: legion::Entity,
        shader: Option<Shader>,
        hover_hints: Vec<(Rgba<f32>, String, String)>,
        options: &Options,
    ) -> Shader {
        let mut button = if let Some(shader) = shader {
            shader
        } else {
            options.shaders.button.clone()
        };
        button.input_handlers.push(input_handler);
        if let Some(update_handler) = update_handler {
            button.update_handlers.push(update_handler);
        }
        Self::add_button_handlers(&mut button);
        if let Some(text) = text {
            button.insert_string_ref("u_text".to_owned(), text.to_owned(), 1);
        }
        button.entity = Some(entity);
        button.hover_hints = hover_hints;

        button
    }

    fn button_input_handler(
        event: HandleEvent,
        _: legion::Entity,
        shader: &mut Shader,
        _: &mut legion::World,
        _: &mut Resources,
    ) {
        if shader.is_active() {
            match event {
                HandleEvent::Hover => {
                    shader.insert_float_ref("u_hovered".to_owned(), 1.0);
                }
                HandleEvent::Press => {
                    shader.insert_float_ref("u_pressed".to_owned(), 1.0);
                }
                _ => {}
            };
        }
    }

    fn button_update_handler(
        _: HandleEvent,
        _: legion::Entity,
        shader: &mut Shader,
        _: &mut legion::World,
        resources: &mut Resources,
    ) {
        if shader.is_active() {
            if let Some(pressed) = shader.parameters.uniforms.try_get_float("u_pressed") {
                if pressed == 1.0 {
                    shader.insert_color_local_ref(
                        "u_color".to_owned(),
                        resources.options.colors.pressed,
                    );
                }
            } else if let Some(hovered) = shader.parameters.uniforms.try_get_float("u_hovered") {
                if hovered == 1.0 {
                    shader.insert_color_local_ref(
                        "u_color".to_owned(),
                        resources.options.colors.hovered,
                    );
                }
            }
        } else {
            shader.insert_color_local_ref("u_color".to_owned(), resources.options.colors.inactive);
        }
    }

    pub fn add_button_handlers(shader: &mut Shader) {
        shader.set_active(true);
        shader.input_handlers.push(Self::button_input_handler);
        shader.update_handlers.push(Self::button_update_handler);
    }
}
