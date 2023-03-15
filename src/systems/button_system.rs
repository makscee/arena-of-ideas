use super::*;

pub struct ButtonSystem {}

impl ButtonSystem {
    pub fn create_button(
        world: &mut legion::World,
        world_entity: legion::Entity,
        resources: &mut Resources,
        icon: Image,
        color: Rgba<f32>,
        listener: fn(legion::Entity, &mut Resources, &mut legion::World, InputEvent),
        position: vec2<f32>,
        uniforms: &ShaderUniforms,
    ) -> legion::Entity {
        let entity = world.push((
            AreaComponent {
                r#type: AreaType::Rectangle {
                    size: vec2(1.0, 1.0),
                },
                position,
            },
            InputComponent {
                hovered: Some(default()),
                dragged: None,
                pressed: Some(default()),
                clicked: Some(default()),
            },
            resources
                .options
                .shaders
                .icon
                .clone()
                .set_uniform("u_texture", ShaderUniform::Texture(icon))
                .set_uniform(
                    "u_icon_color",
                    ShaderUniform::Color(Rgba::try_from(color).unwrap()),
                )
                .merge_uniforms(&uniforms, true),
        ));
        let mut entry = world.entry(entity).unwrap();
        entry.add_component(EntityComponent { entity });
        entry.add_component(Context {
            owner: entity,
            target: entity,
            parent: Some(world_entity),
            vars: default(),
        });
        resources.input.listeners.insert(entity, listener);
        entity
    }

    pub fn change_icon(entity: legion::Entity, world: &mut legion::World, icon: &Image) {
        Self::change_uniform(
            entity,
            world,
            "u_texture".to_string(),
            ShaderUniform::Texture(icon.clone()),
        );
    }

    pub fn change_icon_color(entity: legion::Entity, world: &mut legion::World, color: Rgba<f32>) {
        Self::change_uniform(
            entity,
            world,
            "u_icon_color".to_string(),
            ShaderUniform::Color(color),
        );
    }

    pub fn change_uniform(
        entity: legion::Entity,
        world: &mut legion::World,
        key: String,
        value: ShaderUniform,
    ) {
        let mut entry = world.entry(entity).unwrap();
        let uniforms = &mut entry
            .get_component_mut::<Shader>()
            .unwrap()
            .parameters
            .uniforms;
        uniforms.insert(key, value);
    }
}
